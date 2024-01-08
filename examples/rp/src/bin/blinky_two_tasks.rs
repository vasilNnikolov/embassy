#![no_std]
#![no_main]
/// This example demonstrates how to access a given pin from more than one embassy task
/// The on-board LED is toggled by two tasks with slightly different periods, leading to the
/// apparent duty cycle of the LED increasing, then decreasing, linearly. The phenomenon is similar
/// to interference and the 'beats' you can hear if you play two frequencies close to one another
/// [Link explaining it](https://www.physicsclassroom.com/class/sound/Lesson-3/Interference-and-Beats)
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::{gpio, PeripheralRef};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Duration;
use embassy_time::{Ticker, Timer};
use gpio::{AnyPin, Level, Output};
use {defmt_rtt as _, panic_probe as _};

type LedType = Mutex<ThreadModeRawMutex, Option<PeripheralRef<'static, Output<'static, AnyPin>>>>;
static LED: LedType = Mutex::new(None);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    // set the content of the global LED reference to the real LED pin
    let led = Output::new(AnyPin::from(p.PIN_25), Level::High);
    // inner scope is so that once the mutex is written to, the MutexGuard is dropped, thus the
    // Mutex is released
    {
        *(LED.lock().await) = Some(PeripheralRef::new(led));
    }
    let dt = 100 * 1_000_000;
    let k = 1.003;

    unwrap!(spawner.spawn(toggle(&LED, Duration::from_nanos(dt))));
    unwrap!(spawner.spawn(toggle_slightly_slower(
        &LED,
        Duration::from_nanos((dt as f64 * k) as u64)
    )));
}

async fn toggle_led(led: &'static LedType, delay: Duration) {
    let mut ticker = Ticker::every(delay);
    loop {
        {
            let mut led_unlocked = led.lock().await;
            if let Some(pin_ref) = led_unlocked.as_mut() {
                pin_ref.toggle();
            }
        }
        ticker.next().await;
    }
}
#[embassy_executor::task]
async fn toggle(led: &'static LedType, delay: Duration) {
    toggle_led(led, delay).await
}

#[embassy_executor::task]
async fn toggle_slightly_slower(led: &'static LedType, delay: Duration) {
    toggle_led(led, delay).await
}
