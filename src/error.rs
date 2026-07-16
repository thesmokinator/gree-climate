//! Error types for the gree-climate crate.
//!
//! All fallible operations return [`enum@Error`], which covers every failure
//! mode the library can encounter.

use thiserror::Error;

/// Errors that can occur during GREE device communication.
#[derive(Error, Debug)]
pub enum Error {
    /// The device did not respond within the timeout period.
    #[error("device communication timed out")]
    Timeout,

    /// Received a packet that could not be parsed correctly.
    #[error("invalid packet received from device")]
    InvalidPacket,

    /// A packet expected the `pack` field but it was missing or empty.
    #[error("packet has no 'pack' field")]
    MissingPack,

    /// Encryption or decryption failed (wrong key, bad ciphertext, etc.).
    #[error("encryption/decryption failed: {0}")]
    Crypto(String),

    /// Device binding (authentication) failed.
    #[error("device binding failed: {0}")]
    Authentication(String),

    /// The device is unreachable or has disconnected.
    #[error("device is offline or unreachable")]
    DeviceOffline,

    /// No devices responded to a discovery scan.
    #[error("no devices found on the network")]
    NoDevicesFound,

    /// The client has not been bound to a device yet. Call `Client::bind()`.
    #[error("device is not bound; call Client::bind() first")]
    DeviceNotBound,

    /// An I/O error occurred on the UDP socket.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Failed to serialize or deserialize JSON.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// A value was out of the valid range for the given property.
    #[error("invalid value: {0}")]
    InvalidValue(String),
}
