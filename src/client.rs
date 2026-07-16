//! High-level client for connecting to, authenticating with, and
//! controlling a single GREE air conditioner.
//!
//! The main entry point is [`Client`].

use std::collections::HashMap;
use std::time::Duration;

use serde_json::Value;
use tokio::net::UdpSocket;
use tracing::{debug, info};

use crate::commands::Commands;
use crate::crypto::Cipher;
use crate::device::DeviceInfo;
use crate::error::Error;
use crate::models::{FanSpeed, Mode, SwingHorizontal, SwingVertical};
use crate::packet::Packet;
use crate::protocol;
use crate::state::State;

const BIND_TIMEOUT: Duration = Duration::from_secs(10);
const REFRESH_TIMEOUT: Duration = Duration::from_secs(10);
const COMMAND_TIMEOUT: Duration = Duration::from_secs(10);

/// A connected and (optionally) bound GREE device client.
///
/// `Client` provides the high-level API for interacting with a single
/// air conditioner. After connecting via [`Client::connect`], you must
/// call [`Client::bind`] to authenticate before reading or writing state.
///
/// # Example
///
/// ```rust,no_run
/// # use gree_climate::{Client, DeviceInfo};
/// # async fn example() -> Result<(), gree_climate::Error> {
/// let info = DeviceInfo::new("192.168.1.100".into(), 7000,
///     "aabbcc112233".into(), "Living Room".into());
/// let mut ac = Client::connect(info).await?;
/// ac.bind().await?;
/// ac.refresh().await?;
/// println!("{:?}", ac.state());
/// # Ok(()) }
/// ```
#[derive(Debug)]
pub struct Client {
    pub(crate) device_info: DeviceInfo,
    pub(crate) state: State,
    pub(crate) cipher: Option<Cipher>,
    pub(crate) socket: Option<UdpSocket>,
    pub(crate) hid: Option<String>,
    pub(crate) bound: bool,
}

impl Client {
    /// Create a new client and connect to the device's UDP endpoint.
    ///
    /// This only establishes the socket connection; you still need to call
    /// [`bind`](Self::bind) to authenticate.
    pub async fn connect(device_info: DeviceInfo) -> Result<Self, Error> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        let dev_info = device_info.clone();

        Ok(Self {
            device_info: dev_info,
            state: State::default(),
            cipher: None,
            socket: Some(socket),
            hid: None,
            bound: false,
        })
    }

    /// Returns a reference to the device information.
    pub fn device_info(&self) -> &DeviceInfo {
        &self.device_info
    }

    /// Bind (authenticate) with the device, auto-detecting the cipher version.
    ///
    /// If the `DeviceInfo` already contains a key (from a previous session),
    /// use [`bind_with_key`](Self::bind_with_key) instead to skip the binding
    /// handshake.
    pub async fn bind(&mut self) -> Result<(), Error> {
        self.bind_with_key(None, None).await
    }

    /// Bind using an existing device key and optional cipher kind.
    ///
    /// If `key` is `None` and `device_info.key` has a stored key, that key
    /// will be used. If no key is available, the full binding handshake
    /// is performed.
    pub async fn bind_with_key(
        &mut self,
        key: Option<&str>,
        cipher_kind: Option<crate::models::CipherKind>,
    ) -> Result<(), Error> {
        let key = match (key, &self.device_info.key) {
            (Some(k), _) => Some(k.to_string()),
            (None, Some(k)) => Some(k.clone()),
            (None, None) => None,
        };

        if let Some(k) = key {
            let cipher = match cipher_kind.unwrap_or(crate::models::CipherKind::V1) {
                crate::models::CipherKind::V1 => Cipher::v1_with_key(&k),
                crate::models::CipherKind::V2 => Cipher::v2_with_key(&k),
            };
            self.cipher = Some(cipher);
            self.bound = true;
            self.refresh().await?;
            return Ok(());
        }

        let sock = self.socket.as_ref().ok_or(Error::DeviceOffline)?;
        let bind_packet = Commands::bind_packet(&self.device_info.mac);

        for cipher_try in [Cipher::v1(), Cipher::v2()] {
            debug!(
                "Trying bind with {:?} cipher",
                std::mem::discriminant(&cipher_try)
            );
            let mut try_packet = bind_packet.clone();

            if let Cipher::V1(ref cv1) = cipher_try {
                try_packet.encrypt_pack(&Cipher::V1(cv1.clone()))?;
            } else if let Cipher::V2(ref cv2) = cipher_try {
                try_packet.encrypt_pack(&Cipher::V2(cv2.clone()))?;
            }

            let mut buf = vec![0u8; 4096];
            protocol::send_to(sock, self.device_info.socket_addr(), &try_packet).await?;

            match protocol::recv_from(sock, BIND_TIMEOUT, &mut buf).await {
                Ok((size, _)) => {
                    let response = Packet::from_bytes(&buf[..size])?;

                    if let Ok(true) = response.is_bind_ok(&cipher_try) {
                        let key = response.get_bind_key(&cipher_try)?;
                        info!("Device bound with key: {key}");
                        let mut cipher = cipher_try;
                        cipher.set_key(&key);
                        self.device_info.key = Some(key);
                        self.cipher = Some(cipher);
                        self.bound = true;
                        self.refresh().await?;
                        return Ok(());
                    }
                }
                Err(Error::Timeout) => {
                    debug!("Bind timeout, trying next cipher...");
                    continue;
                }
                Err(e) => return Err(e),
            }
        }

        Err(Error::Authentication(
            "Failed to bind with any cipher".into(),
        ))
    }

    /// Request the current state from the device and update the internal cache.
    ///
    /// Call this to synchronize the client's view of the state with the
    /// physical device. This is automatically called after a successful bind.
    pub async fn refresh(&mut self) -> Result<(), Error> {
        self.ensure_bound()?;
        let sock = self.socket.as_ref().ok_or(Error::DeviceOffline)?;
        let cipher = self.cipher.as_ref().ok_or(Error::DeviceNotBound)?;

        let include_hid = self.hid.is_none();
        let status = Commands::status_packet(&self.device_info.mac, include_hid);
        let mut packet = status.clone();
        packet.encrypt_pack(cipher)?;

        let mut buf = vec![0u8; 4096];
        protocol::send_to(sock, self.device_info.socket_addr(), &packet).await?;

        let (size, _) = protocol::recv_from(sock, REFRESH_TIMEOUT, &mut buf).await?;
        let response = Packet::from_bytes(&buf[..size])?;

        let inner = response.decrypt_pack(cipher)?;
        let inner_type = inner["t"].as_str().unwrap_or("");

        let properties = match inner_type {
            "dat" => {
                let cols: Vec<String> = inner["cols"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();
                let values: Vec<Value> = inner["dat"].as_array().cloned().unwrap_or_default();
                cols.into_iter()
                    .zip(values)
                    .collect::<HashMap<String, Value>>()
            }
            "res" => {
                let opts: Vec<String> = inner["opt"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();
                let values: Vec<Value> = inner["val"]
                    .as_array()
                    .or_else(|| inner["p"].as_array())
                    .cloned()
                    .unwrap_or_default();
                opts.into_iter()
                    .zip(values)
                    .collect::<HashMap<String, Value>>()
            }
            _ => {
                return Err(Error::InvalidPacket);
            }
        };

        if let Some(hid) = properties.get("hid").and_then(|v| v.as_str()) {
            self.hid = Some(hid.to_string());
        }

        self.state
            .merge_from_properties(&properties, self.hid.as_deref());
        debug!("State refreshed: {:?}", self.state);

        Ok(())
    }

    /// Returns a reference to the cached device state.
    ///
    /// Call [`refresh`](Self::refresh) first to fetch the latest state
    /// from the device.
    pub fn state(&self) -> &State {
        &self.state
    }

    /// Turn the device on or off.
    pub async fn set_power(&mut self, on: bool) -> Result<(), Error> {
        self.ensure_bound()?;
        let packet = Commands::set_power(&self.device_info.mac, on);
        self.send_command(packet).await
    }

    /// Set the operating mode (Auto, Cool, Dry, Fan, Heat).
    pub async fn set_mode(&mut self, mode: Mode) -> Result<(), Error> {
        self.ensure_bound()?;
        let packet = Commands::set_mode(&self.device_info.mac, mode);
        self.send_command(packet).await
    }

    /// Set the target temperature.
    ///
    /// Valid range: 16–30°C (Celsius) or 61–86°F (Fahrenheit).
    pub async fn set_temperature(&mut self, temp: u8) -> Result<(), Error> {
        self.ensure_bound()?;
        let packet = Commands::set_temperature(&self.device_info.mac, temp);
        self.send_command(packet).await
    }

    /// Set the fan speed.
    pub async fn set_fan_speed(&mut self, speed: FanSpeed) -> Result<(), Error> {
        self.ensure_bound()?;
        let packet = Commands::set_fan_speed(&self.device_info.mac, speed);
        self.send_command(packet).await
    }

    /// Set the vertical swing position.
    pub async fn set_swing_vertical(&mut self, swing: SwingVertical) -> Result<(), Error> {
        self.ensure_bound()?;
        let packet = Commands::set_swing_vertical(&self.device_info.mac, swing);
        self.send_command(packet).await
    }

    /// Set the horizontal swing position.
    pub async fn set_swing_horizontal(&mut self, swing: SwingHorizontal) -> Result<(), Error> {
        self.ensure_bound()?;
        let packet = Commands::set_swing_horizontal(&self.device_info.mac, swing);
        self.send_command(packet).await
    }

    /// Enable or disable turbo mode.
    pub async fn set_turbo(&mut self, on: bool) -> Result<(), Error> {
        self.ensure_bound()?;
        let packet = Commands::set_turbo(&self.device_info.mac, on);
        self.send_command(packet).await
    }

    /// Enable or disable quiet mode.
    pub async fn set_quiet(&mut self, on: bool) -> Result<(), Error> {
        self.ensure_bound()?;
        let packet = Commands::set_quiet(&self.device_info.mac, on);
        self.send_command(packet).await
    }

    /// Enable or disable sleep mode.
    pub async fn set_sleep(&mut self, on: bool) -> Result<(), Error> {
        self.ensure_bound()?;
        let packet = Commands::set_sleep(&self.device_info.mac, on);
        self.send_command(packet).await
    }

    /// Enable or disable steady heat (8°C maintenance mode).
    pub async fn set_steady_heat(&mut self, on: bool) -> Result<(), Error> {
        self.ensure_bound()?;
        let packet = Commands::set_steady_heat(&self.device_info.mac, on);
        self.send_command(packet).await
    }

    /// Enable or disable power save mode.
    pub async fn set_power_save(&mut self, on: bool) -> Result<(), Error> {
        self.ensure_bound()?;
        let packet = Commands::set_power_save(&self.device_info.mac, on);
        self.send_command(packet).await
    }

    /// Enable or disable fresh air circulation.
    pub async fn set_fresh_air(&mut self, on: bool) -> Result<(), Error> {
        self.ensure_bound()?;
        let packet = Commands::set_fresh_air(&self.device_info.mac, on);
        self.send_command(packet).await
    }

    /// Enable or disable X-Fan (coil drying after cooling/drying).
    pub async fn set_xfan(&mut self, on: bool) -> Result<(), Error> {
        self.ensure_bound()?;
        let packet = Commands::set_xfan(&self.device_info.mac, on);
        self.send_command(packet).await
    }

    /// Enable or disable the anion (ionizer/ozone generator).
    pub async fn set_anion(&mut self, on: bool) -> Result<(), Error> {
        self.ensure_bound()?;
        let packet = Commands::set_anion(&self.device_info.mac, on);
        self.send_command(packet).await
    }

    /// Enable or disable the unit's front panel light.
    pub async fn set_light(&mut self, on: bool) -> Result<(), Error> {
        self.ensure_bound()?;
        let packet = Commands::set_light(&self.device_info.mac, on);
        self.send_command(packet).await
    }

    /// Send a raw command to the device with custom property key/value pairs.
    ///
    /// Useful for experimental or unsupported properties.
    pub async fn send_raw(&mut self, props: HashMap<&str, Value>) -> Result<(), Error> {
        self.ensure_bound()?;
        let opts: Vec<&str> = props.keys().copied().collect();
        let values: Vec<Value> = props.values().cloned().collect();
        let packet = Commands::cmd_packet(&self.device_info.mac, opts, values);
        self.send_command(packet).await
    }

    async fn send_command(&mut self, mut packet: Packet) -> Result<(), Error> {
        let sock = self.socket.as_ref().ok_or(Error::DeviceOffline)?;
        let cipher = self.cipher.as_ref().ok_or(Error::DeviceNotBound)?;
        packet.encrypt_pack(cipher)?;

        protocol::send_to(sock, self.device_info.socket_addr(), &packet).await?;

        let mut buf = vec![0u8; 4096];
        let (_, _) = protocol::recv_from(sock, COMMAND_TIMEOUT, &mut buf).await?;

        Ok(())
    }

    fn ensure_bound(&self) -> Result<(), Error> {
        if !self.bound {
            return Err(Error::DeviceNotBound);
        }
        Ok(())
    }
}
