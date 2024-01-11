use serde::{Deserialize, Serialize};
use serde_json_core;
pub type CommandDecodeErr = ();

#[derive(Serialize, Deserialize)]
pub enum Command {
    Config(Config),
    TransmitByte(u8),
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    /// Transmit freqency, Hz
    pub tx_freq: u64,
    /// Receive freqency, Hz
    pub rx_freq: u64,
}

impl Command {
    pub fn from_bytes(bytes: &[u8]) -> Result<Command, CommandDecodeErr> {
        if bytes.len() == 0 {
            return Err(());
        }
        match serde_json_core::from_slice::<Command>(bytes) {
            Ok((cmd, _)) => Ok(cmd),
            Err(_) => Err(()),
        }
    }
}
