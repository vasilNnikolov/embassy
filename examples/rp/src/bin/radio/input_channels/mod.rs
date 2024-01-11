use crate::command::Command;

pub mod no_input;
pub mod random_input;

/// Trait for each source of commands
pub trait CommandInput {
    async fn get_command(&mut self) -> Command;
}
