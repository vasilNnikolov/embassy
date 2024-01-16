#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio;
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
            Command::TransmitByte(_b) => Timer::after_nanos(2).await,
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

// #[embassy_executor::main]
async fn main_2(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let led = Output::new(AnyPin::from(p.PIN_25), Level::High);
    {
        *(LED.lock().await) = Some(led);
    }

    // set the content of the global LED reference to the real LED pin
    unwrap!(spawner.spawn(led_utilities::toggle_led(&LED, Duration::from_millis(500))));

    let mut random_input_stream = RandomInput {};

    loop {
        match random_input_stream.get_command().await {
            Command::Config(_c) => match spawner.spawn(led_utilities::fast_blink(&LED, Duration::from_millis(50))) {
                Ok(_) => (),
                Err(_) => (),
            },
            Command::TransmitByte(_b) => Timer::after_nanos(2).await,
        }
    }
}
