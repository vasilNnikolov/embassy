use super::CommandInput;
use crate::{Command, Config};
use embassy_time::{Instant, Timer};
struct NoInput;

#[allow(unreachable_code)]
impl CommandInput for NoInput {
    async fn get_command(&mut self) -> Command {
        loop {
            Timer::after_secs(2).await;
        }
        return Command::Config(Config { tx_freq: 1, rx_freq: 2 });
    }
}
