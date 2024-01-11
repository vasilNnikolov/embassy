#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio;
use embassy_sync::mutex::Mutex;
use embassy_time::Duration;
use embassy_time::Timer;
use gpio::{AnyPin, Level, Output};
use {defmt_rtt as _, panic_probe as _};

mod command;
use command::{Command, Config};

mod input_channels;
use input_channels::{random_input::RandomInput, CommandInput};

mod led_utilities;

use led_utilities::{fast_blink, toggle_led, LedType};

static LED: LedType = Mutex::new(None);

// TODO implement a global command queue that all command input streams push to

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let led = Output::new(AnyPin::from(p.PIN_25), Level::High);
    {
        *(LED.lock().await) = Some(led);
    }

    // set the content of the global LED reference to the real LED pin
    unwrap!(spawner.spawn(toggle_led(&LED, Duration::from_millis(500))));
    let mut random_input_stream = RandomInput {};

    loop {
        match random_input_stream.get_command().await {
            Command::Config(_c) => match spawner.spawn(fast_blink(&LED, Duration::from_millis(50))) {
                Ok(_) => (),
                Err(_) => (),
            },
            Command::TransmitByte(_b) => Timer::after_nanos(2).await,
        }
    }
}
