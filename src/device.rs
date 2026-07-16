//! Device information and identification.
//!
//! [`DeviceInfo`] is the primary type in this module — it holds everything
//! needed to connect to and identify a GREE device on the network.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Information about a discovered GREE device.
///
/// Contains everything needed to connect and identify a device.
/// `DeviceInfo` is returned by [`crate::discover`] and fed to
/// [`crate::Client::connect`].
///
/// Two `DeviceInfo` instances are considered equal if they have the same
/// MAC address, name, brand, model, and version — IP and port may differ.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// IPv4 address of the device.
    pub ip: String,
    /// UDP port (typically 7000).
    pub port: u16,
    /// MAC address (e.g., `"aabbcc112233"`).
    pub mac: String,
    /// Human-readable name reported by the device.
    pub name: String,
    /// Brand name, if reported.
    #[serde(default)]
    pub brand: Option<String>,
    /// Model identifier, if reported.
    #[serde(default)]
    pub model: Option<String>,
    /// Firmware version, if reported.
    #[serde(default)]
    pub version: Option<String>,
    /// Encryption key obtained during binding (persisted between sessions).
    #[serde(skip)]
    pub key: Option<String>,
}

impl DeviceInfo {
    /// Create a new `DeviceInfo` with the minimum required fields.
    ///
    /// If `name` is empty, the MAC address (with colons stripped) is used.
    pub fn new(ip: String, port: u16, mac: String, name: String) -> Self {
        let mac_stripped = mac.replace(':', "");
        let resolved_name = if name.is_empty() { mac_stripped } else { name };
        Self {
            ip,
            port,
            mac,
            name: resolved_name,
            brand: None,
            model: None,
            version: None,
            key: None,
        }
    }

    /// Returns the device's address as a `SocketAddr`.
    pub fn socket_addr(&self) -> std::net::SocketAddr {
        crate::protocol::socket_addr(&self.ip, self.port)
    }
}

impl fmt::Display for DeviceInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Device: {} @ {}:{} (mac: {})",
            self.name, self.ip, self.port, self.mac
        )
    }
}

impl PartialEq for DeviceInfo {
    fn eq(&self, other: &Self) -> bool {
        self.mac == other.mac
            && self.name == other.name
            && self.brand == other.brand
            && self.model == other.model
            && self.version == other.version
    }
}

impl Eq for DeviceInfo {}
