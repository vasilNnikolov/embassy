#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::{gpio, PeripheralRef};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Duration;
use embassy_time::Timer;
use gpio::{AnyPin, Level, Output};
use {defmt_rtt as _, panic_probe as _};

type LedType = Mutex<ThreadModeRawMutex, Option<PeripheralRef<'static, Output<'static, AnyPin>>>>;
static LED: LedType = Mutex::new(None);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    // set the content of the global LED reference to the real LED pin
    let led = Output::new(AnyPin::from(p.PIN_25), Level::High);
    {
        *(LED.lock().await) = Some(PeripheralRef::new(led));
    }
    let dt = 100 * 1000;
    let k = 1.015;

    unwrap!(spawner.spawn(toggle(&LED, Duration::from_micros(dt))));
    unwrap!(spawner.spawn(toggle2(&LED, Duration::from_micros((dt as f64 * k) as u64))));
}

async fn toggle_led(led: &'static LedType, delay: Duration) {
    loop {
        {
            let mut led_unlocked = led.lock().await;
            if let Some(pin_ref) = led_unlocked.as_mut() {
                pin_ref.toggle();
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
