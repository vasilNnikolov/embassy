#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio;
use embassy_rp::spi;
use embassy_rp::spi::Spi;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::{Channel, Sender};
use embassy_sync::mutex::Mutex;
use embassy_time::Duration;
use embassy_time::Timer;
use gpio::{AnyPin, Level, Output};
use {defmt_rtt as _, panic_probe as _};

mod command;
use command::{Command, Config};

mod input_channels;
use input_channels::{no_input::NoInput, random_input::RandomInput, CommandInput};

mod led_utilities;

static LED: led_utilities::LedType = Mutex::new(None);

const COMMAND_CHANNEL_SIZE: usize = 64;

static COMMAND_CHANNEL: Channel<ThreadModeRawMutex, Command, COMMAND_CHANNEL_SIZE> = Channel::new();
type CommandChannelSender = Sender<'static, ThreadModeRawMutex, Command, COMMAND_CHANNEL_SIZE>;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let led = Output::new(AnyPin::from(p.PIN_25), Level::High);
    {
        *(LED.lock().await) = Some(led);
    }
    // SPI code

    let miso = p.PIN_12;
    let mosi = p.PIN_11;
    let clk = p.PIN_10;
    let touch_cs = p.PIN_16;

    // create SPI
    let mut config = spi::Config::default();
    config.frequency = 2_000_000;
    let mut spi = Spi::new_blocking(p.SPI1, clk, mosi, miso, config);

    // Configure CS
    let mut cs = Output::new(touch_cs, Level::Low);

    // set the content of the global LED reference to the real LED pin
    unwrap!(spawner.spawn(led_utilities::toggle_led(&LED, Duration::from_millis(500))));

    // spawn different input streams
    unwrap!(spawner.spawn(random_input(COMMAND_CHANNEL.sender())));
    unwrap!(spawner.spawn(no_input(COMMAND_CHANNEL.sender())));

    // handle commands from different input streams
    loop {
        match COMMAND_CHANNEL.receive().await {
            Command::Config(_c) => match spawner.spawn(led_utilities::fast_blink(&LED, Duration::from_millis(50))) {
                Ok(_) => (),
                Err(_) => (),
            },
            Command::TransmitByte(b) => {
                cs.set_low();
                spi.blocking_write(&[b, b, b, b]);
                cs.set_high();
            }
        }
    }
}

async fn forward_command_to_channel(mut stream: impl CommandInput, channel_sender: CommandChannelSender) -> ! {
    loop {
        let command = stream.get_command().await;
        channel_sender.send(command).await;
    }
}

#[embassy_executor::task]
async fn random_input(channel_sender: CommandChannelSender) {
    let input_source = RandomInput {};
    forward_command_to_channel(input_source, channel_sender).await;
}

#[embassy_executor::task]
async fn no_input(channel_sender: CommandChannelSender) {
    let input_source = NoInput {};
    forward_command_to_channel(input_source, channel_sender).await;
}
