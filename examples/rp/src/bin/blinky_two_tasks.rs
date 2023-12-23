#![no_std]
#![no_main]
use core::cell::RefCell;

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio;
use embassy_sync::blocking_mutex::raw::*;
use embassy_sync::mutex::Mutex;
use embassy_time::Duration;
use embassy_time::Timer;
use gpio::{AnyPin, Level, Output};
use {defmt_rtt as _, panic_probe as _};

type LedType = Mutex<ThreadModeRawMutex, RefCell<Option<Output<'static, AnyPin>>>>;
static LED: LedType = Mutex::new(RefCell::new(None));

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let led = Output::new(AnyPin::from(p.PIN_25), Level::High);
    {
        *(LED.lock().await) = RefCell::new(Some(led));
    }
    let dt = 100 * 1000;
    let k = 1.005;

    unwrap!(spawner.spawn(toggle(&LED, Duration::from_micros(dt))));
    unwrap!(spawner.spawn(toggle2(&LED, Duration::from_micros((dt as f32 * k) as u64))));
}

async fn toggle_led(led: &'static LedType, delay: Duration) {
    loop {
        {
            let mut led_unlocked = led.lock().await;
            match led_unlocked.get_mut() {
                &mut Some(ref mut x) => x.toggle(),
                _ => {}
            }
        }
        Timer::after(delay).await;
    }
}

#[embassy_executor::task]
async fn toggle(led: &'static LedType, delay: Duration) {
    toggle_led(led, delay).await
}

#[embassy_executor::task]
async fn toggle2(led: &'static LedType, delay: Duration) {
    toggle_led(led, delay).await
}
