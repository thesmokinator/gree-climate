#![deny(unsafe_code)]
#![allow(missing_docs)]

//! # gree-climate
//!
//! An idiomatic, fully async Rust library for controlling GREE air conditioners
//! over the local network. This crate implements the GREE local protocol used by
//! the Gree+ app and Home Assistant's greeclimate integration.
//!
//! ## Features
//!
//! - **Device discovery** via UDP broadcast on your LAN
//! - **Pairing / handshake** with automatic cipher detection (V1 and V2)
//! - **State reading** — power, mode, temperatures, fan, swing, and more
//! - **Command sending** — set power, mode, temperature, fan speed, swing, turbo, etc.
//! - **Clean, typed API** with enums instead of magic strings
//! - **Full async** support via Tokio
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use gree_climate::{discover, Client, Mode, FanSpeed};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), gree_climate::Error> {
//!     // Discover devices on the local network
//!     let devices = discover().await?;
//!
//!     // Connect to the first device found
//!     let device = devices.into_iter().next()
//!         .ok_or(gree_climate::Error::NoDevicesFound)?;
//!     let mut ac = Client::connect(device).await?;
//!
//!     // Bind (authenticate) with the device
//!     ac.bind().await?;
//!
//!     // Read the current state
//!     let state = ac.state();
//!     println!("Power: {}, Mode: {:?}, Temp: {}°C",
//!         state.power, state.mode, state.target_temperature);
//!
//!     // Send commands
//!     ac.set_power(true).await?;
//!     ac.set_mode(Mode::Cool).await?;
//!     ac.set_temperature(23).await?;
//!     ac.set_fan_speed(FanSpeed::Auto).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Protocol Details
//!
//! GREE devices communicate over UDP on port 7000 using JSON-encoded packets.
//! The packet payload is encrypted using AES (ECB for V1, GCM for V2) and
//! base64-encoded. Discovery uses an unencrypted broadcast.
//!
//! ## Cipher Versions
//!
//! - **V1**: AES-128-ECB with PKCS7 padding (older devices)
//! - **V2**: AES-128-GCM with AEAD tag (newer devices)
//!
//! The library auto-detects the correct cipher during binding.

pub mod client;
pub mod commands;
pub mod crypto;
pub mod device;
pub mod discovery;
pub mod error;
pub mod models;
pub mod packet;
pub mod protocol;
pub mod state;
pub(crate) mod utils;

pub use client::Client;
pub use device::DeviceInfo;
pub use discovery::discover;
pub use error::Error;
pub use models::{
    CipherKind, FanSpeed, Mode, SwingHorizontal, SwingVertical, TemperatureUnit,
};
pub use state::State;

/// Alias for `std::result::Result<T, gree_climate::Error>`.
pub type Result<T> = std::result::Result<T, Error>;
