use embassy_rp::gpio;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Ticker};
use gpio::{AnyPin, Output};
use {defmt_rtt as _, panic_probe as _};

pub type LedType = Mutex<ThreadModeRawMutex, Option<Output<'static>>>;

#[embassy_executor::task]
pub async fn toggle_led(led: &'static LedType, delay: Duration) {
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
pub async fn fast_blink(led: &'static LedType, delay: Duration) {
    let mut ticker = Ticker::every(delay);
    let mut led_unlocked = led.lock().await;
    // take for the whole duration of the fast ticking
    if let Some(pin_ref) = led_unlocked.as_mut() {
        for _ in 0..10 {
            (*pin_ref).toggle();
            ticker.next().await
        }
    }
}
