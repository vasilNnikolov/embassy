#![no_std]
#![no_main]

use core::cell::RefCell;
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::{gpio, PeripheralRef};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::mutex::Mutex;
use embassy_time::Duration;
use embassy_time::{Instant, Ticker, Timer};
use gpio::{AnyPin, Level, Output};
use serde::{Deserialize, Serialize};
use serde_json_core;
use {defmt_rtt as _, panic_probe as _};

type ErrType = ();
type LedType = Mutex<ThreadModeRawMutex, Option<PeripheralRef<'static, Output<'static, AnyPin>>>>;
static LED: LedType = Mutex::new(None);
// size of queue for RF chip in bytes
const RF_CHANNEL_SIZE: usize = 256;

type ChannelType = Mutex<ThreadModeRawMutex, RefCell<Channel<ThreadModeRawMutex, u8, RF_CHANNEL_SIZE>>>;

static RF_CHANNEL: ChannelType = Mutex::new(RefCell::new(Channel::new()));

// TODO implement a global command queue that all command input streams push to

#[derive(Serialize, Deserialize)]
enum Command {
    Config(Config),
    TransmitByte(u8),
}

impl Command {
    pub fn from_bytes(bytes: &[u8]) -> Result<Command, ErrType> {
        if bytes.len() == 0 {
            return Err(());
        }
        match serde_json_core::from_slice::<Command>(bytes) {
            Ok((cmd, _)) => Ok(cmd),
            Err(_) => Err(()),
        }
    }
}

trait CommandInput {
    async fn get_command(&mut self) -> Command;
}

#[derive(Serialize, Deserialize)]
struct Config {
    /// Transmit freqency, Hz
    tx_freq: u64,
    /// Receive freqency, Hz
    rx_freq: u64,
}

/// dummy Input source that sends random data every second
struct RandomInput;

impl CommandInput for RandomInput {
    async fn get_command(&mut self) -> Command {
        Timer::after_secs(1).await;
        let time_since_start = Instant::now().as_micros();
        if time_since_start % 10 == 0 {
            return Command::Config(Config {
                tx_freq: time_since_start,
                rx_freq: u64::MAX - time_since_start,
            });
        } else {
            return Command::TransmitByte(5 as u8);
        }
    }
}

struct NoInput;

impl CommandInput for NoInput {
    async fn get_command(&mut self) -> Command {
        loop {
            Timer::after_secs(2).await;
        }
        return Command::Config(Config { tx_freq: 1, rx_freq: 2 });
    }
}

#[embassy_executor::task]
async fn fast_blink(led: &'static LedType, delay: Duration) {
    let mut ticker = Ticker::every(delay);
    let mut led_unlocked = led.lock().await;
    // take for the whole duration of the fast ticking
    if let Some(pin_ref) = led_unlocked.as_mut() {
        for _ in 0..50 {
            (*pin_ref).toggle();
            ticker.next().await
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // set the content of the global LED reference to the real LED pin
    let led = Output::new(AnyPin::from(p.PIN_25), Level::High);
    {
        *(LED.lock().await) = Some(PeripheralRef::new(led));
    }
    unwrap!(spawner.spawn(working_indicator(&LED, Duration::from_millis(500))));
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

#[embassy_executor::task]
async fn working_indicator(led: &'static LedType, delay: Duration) {
    let mut ticker = Ticker::every(delay);
    loop {
        {
            let mut led_unlocked = led.lock().await;
            if let Some(pin_ref) = led_unlocked.as_mut() {
                pin_ref.toggle();
            }
        }
        ticker.next().await
    }
}
