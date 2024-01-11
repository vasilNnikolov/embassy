use super::CommandInput;
use crate::{Command, Config};
use embassy_time::{Instant, Timer};
/// dummy Input source that sends random data every second
pub struct RandomInput;

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
