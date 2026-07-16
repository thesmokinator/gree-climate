//! Typed representations of GREE device state enums.
//!
//! Every operational mode and setting is represented as a strongly-typed
//! enum rather than a magic integer or string. Each type implements
//! `TryFrom<u8>` for safe conversion from raw device values.

use serde_repr::{Deserialize_repr, Serialize_repr};

/// Operating mode of the air conditioner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Mode {
    /// Automatic mode — device chooses heating or cooling.
    Auto = 0,
    /// Cooling mode.
    Cool = 1,
    /// Dehumidification / dry mode.
    Dry = 2,
    /// Fan-only mode (no heating or cooling).
    Fan = 3,
    /// Heating mode.
    Heat = 4,
}

impl TryFrom<u8> for Mode {
    type Error = crate::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Auto),
            1 => Ok(Self::Cool),
            2 => Ok(Self::Dry),
            3 => Ok(Self::Fan),
            4 => Ok(Self::Heat),
            v => Err(crate::Error::InvalidValue(format!(
                "invalid mode value: {v}"
            ))),
        }
    }
}

/// Fan speed levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum FanSpeed {
    /// Automatic fan speed.
    Auto = 0,
    /// Low speed.
    Low = 1,
    /// Medium-low speed.
    MediumLow = 2,
    /// Medium speed.
    Medium = 3,
    /// Medium-high speed.
    MediumHigh = 4,
    /// High / maximum speed.
    High = 5,
}

impl TryFrom<u8> for FanSpeed {
    type Error = crate::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Auto),
            1 => Ok(Self::Low),
            2 => Ok(Self::MediumLow),
            3 => Ok(Self::Medium),
            4 => Ok(Self::MediumHigh),
            5 => Ok(Self::High),
            v => Err(crate::Error::InvalidValue(format!(
                "invalid fan speed value: {v}"
            ))),
        }
    }
}

/// Temperature measurement unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum TemperatureUnit {
    /// Celsius.
    Celsius = 0,
    /// Fahrenheit.
    Fahrenheit = 1,
}

impl TryFrom<u8> for TemperatureUnit {
    type Error = crate::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Celsius),
            1 => Ok(Self::Fahrenheit),
            v => Err(crate::Error::InvalidValue(format!(
                "invalid temperature unit value: {v}"
            ))),
        }
    }
}

/// Vertical swing / louver position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum SwingVertical {
    /// Default position (off).
    Default = 0,
    /// Full swing (continuous up-down movement).
    FullSwing = 1,
    /// Fixed at uppermost position.
    FixedUpper = 2,
    /// Fixed at upper-middle position.
    FixedUpperMiddle = 3,
    /// Fixed at middle position.
    FixedMiddle = 4,
    /// Fixed at lower-middle position.
    FixedLowerMiddle = 5,
    /// Fixed at lowest position.
    FixedLower = 6,
    /// Swing in upper range.
    SwingUpper = 7,
    /// Swing in upper-middle range.
    SwingUpperMiddle = 8,
    /// Swing in middle range.
    SwingMiddle = 9,
    /// Swing in lower-middle range.
    SwingLowerMiddle = 10,
    /// Swing in lower range.
    SwingLower = 11,
}

impl TryFrom<u8> for SwingVertical {
    type Error = crate::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Default),
            1 => Ok(Self::FullSwing),
            2 => Ok(Self::FixedUpper),
            3 => Ok(Self::FixedUpperMiddle),
            4 => Ok(Self::FixedMiddle),
            5 => Ok(Self::FixedLowerMiddle),
            6 => Ok(Self::FixedLower),
            7 => Ok(Self::SwingUpper),
            8 => Ok(Self::SwingUpperMiddle),
            9 => Ok(Self::SwingMiddle),
            10 => Ok(Self::SwingLowerMiddle),
            11 => Ok(Self::SwingLower),
            v => Err(crate::Error::InvalidValue(format!(
                "invalid vertical swing value: {v}"
            ))),
        }
    }
}

/// Horizontal swing / louver position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum SwingHorizontal {
    /// Default position (off).
    Default = 0,
    /// Full swing (continuous left-right movement).
    FullSwing = 1,
    /// Fixed left.
    Left = 2,
    /// Fixed left-center.
    LeftCenter = 3,
    /// Fixed center.
    Center = 4,
    /// Fixed right-center.
    RightCenter = 5,
    /// Fixed right.
    Right = 6,
}

impl TryFrom<u8> for SwingHorizontal {
    type Error = crate::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Default),
            1 => Ok(Self::FullSwing),
            2 => Ok(Self::Left),
            3 => Ok(Self::LeftCenter),
            4 => Ok(Self::Center),
            5 => Ok(Self::RightCenter),
            6 => Ok(Self::Right),
            v => Err(crate::Error::InvalidValue(format!(
                "invalid horizontal swing value: {v}"
            ))),
        }
    }
}

/// Cipher version used for device communication.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CipherKind {
    /// AES-128-ECB cipher (older devices).
    V1,
    /// AES-128-GCM cipher (newer devices).
    V2,
}
