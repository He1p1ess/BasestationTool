use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const SERVICE: Uuid = Uuid::from_u128(0x00001523_1212_efde_1523_785feabcd124);
pub const POWER: Uuid = Uuid::from_u128(0x00001525_1212_efde_1523_785feabcd124);
pub const CHANNEL: Uuid = Uuid::from_u128(0x00001524_1212_efde_1523_785feabcd124);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum PowerCommand {
    Sleep = 0x00,
    Standby = 0x02,
    // Legacy EXE btnModeOn_Click / btnModeOnAll_Click pass 1, not read value 0x0b.
    On = 0x01,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Status {
    Sleep,
    Standby,
    /// Reported awake state, retaining the raw byte; not proof of laser/motor readiness.
    On(u8),
    Unknown(u8),
    #[default]
    Unavailable,
}
impl Status {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 1 {
            bail!("Expected one status byte, received {}", bytes.len());
        }
        Ok(match bytes[0] {
            0 => Self::Sleep,
            2 => Self::Standby,
            // Observed/reverse-engineered LHB read values, distinct from write commands.
            1 | 9 | 11 => Self::On(bytes[0]),
            b => Self::Unknown(b),
        })
    }
    pub fn matches(&self, command: PowerCommand) -> bool {
        matches!(
            (self, command),
            (Self::Sleep, PowerCommand::Sleep)
                | (Self::Standby, PowerCommand::Standby)
                | (Self::On(_), PowerCommand::On)
        )
    }
}
impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::On(b) => write!(f, "On (reported 0x{b:02X})"),
            Self::Unknown(b) => write!(f, "Unknown (0x{b:02X})"),
            s => write!(f, "{s:?}"),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Channel(u8);
impl Channel {
    pub fn new(value: u8) -> Result<Self> {
        if !(1..=16).contains(&value) {
            bail!("Channel must be 1–16");
        }
        Ok(Self(value))
    }
    pub fn byte(self) -> u8 {
        self.0
    }
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 1 {
            bail!("Expected one channel byte");
        }
        Self::new(bytes[0])
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn power_and_status() {
        for (cmd, byte, status) in [
            (PowerCommand::Sleep, 0, Status::Sleep),
            (PowerCommand::Standby, 2, Status::Standby),
            (PowerCommand::On, 1, Status::On(1)),
        ] {
            assert_eq!(cmd as u8, byte);
            assert_eq!(Status::parse(&[byte]).unwrap(), status);
        }
        for byte in [1, 9, 11] {
            assert_eq!(Status::parse(&[byte]).unwrap(), Status::On(byte));
            assert!(Status::On(byte).matches(PowerCommand::On));
            assert!(!Status::On(byte).matches(PowerCommand::Sleep));
        }
        assert_eq!(Status::parse(&[12]).unwrap(), Status::Unknown(12));
        assert_eq!(Status::parse(&[255]).unwrap(), Status::Unknown(255));
        assert!(Status::parse(&[]).is_err());
        assert!(Status::parse(&[0, 0]).is_err());
    }
    #[test]
    fn channels() {
        for i in [1, 5, 16] {
            assert_eq!(Channel::new(i).unwrap().byte(), i);
        }
        for i in [0, 17, 255] {
            assert!(Channel::new(i).is_err());
        }
    }
}
