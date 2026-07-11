use std::time::Duration;

use tracing::{debug, info};

use crate::commands::Commands;
use crate::crypto::{Cipher, CipherV1};
use crate::device::DeviceInfo;
use crate::error::Error;
use crate::packet::{Packet, GREE_PORT};
use crate::protocol;

const DEFAULT_SCAN_TIMEOUT_SECS: u64 = 5;

pub async fn discover() -> Result<Vec<DeviceInfo>, Error> {
    discover_with_timeout(Duration::from_secs(DEFAULT_SCAN_TIMEOUT_SECS)).await
}

pub async fn discover_with_timeout(
    timeout: Duration,
) -> Result<Vec<DeviceInfo>, Error> {
    info!("Discovering GREE devices...");
    let socket = protocol::create_broadcast_socket().await?;
    let scan_packet = Commands::scan_packet();

    protocol::broadcast(&socket, &scan_packet, GREE_PORT).await?;

    let mut buffer = vec![0u8; 4096];
    let mut devices: Vec<DeviceInfo> = Vec::new();
    let cipher = Cipher::V1(CipherV1::new());

    loop {
        match protocol::recv_from(&socket, timeout, &mut buffer).await {
            Ok((size, addr)) => {
                let raw_packet = match Packet::from_bytes(&buffer[..size]) {
                    Ok(p) => p,
                    Err(e) => {
                        debug!("Failed to parse packet from {addr}: {e}");
                        continue;
                    }
                };

                let pack = match raw_packet.decrypt_pack(&cipher) {
                    Ok(p) => p,
                    Err(e) => {
                        debug!("Failed to decrypt/parse pack from {addr}: {e}");
                        continue;
                    }
                };

                let mac = pack["mac"]
                    .as_str()
                    .or_else(|| pack["cid"].as_str())
                    .unwrap_or("")
                    .to_string();

                if mac.is_empty() {
                    debug!("Skipping packet from {addr} with no mac/cid");
                    continue;
                }

                let name = pack["name"].as_str().unwrap_or("").to_string();
                let brand = pack["brand"].as_str().map(|s| s.to_string());
                let model = pack["model"].as_str().map(|s| s.to_string());
                let version = pack["ver"].as_str().map(|s| s.to_string());

                let device = DeviceInfo {
                    ip: addr.ip().to_string(),
                    port: addr.port(),
                    mac: mac.clone(),
                    name: if name.is_empty() {
                        mac.replace(':', "")
                    } else {
                        name
                    },
                    brand,
                    model,
                    version,
                    key: None,
                };

                if !devices.iter().any(|d| d.mac == device.mac) {
                    info!("Found device: {device}");
                    devices.push(device);
                }
            }
            Err(Error::Timeout) => break,
            Err(e) => return Err(e),
        }
    }

    if devices.is_empty() {
        return Err(Error::NoDevicesFound);
    }

    Ok(devices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_info_display() {
        let info = DeviceInfo::new(
            "192.168.1.100".into(),
            7000,
            "aabbcc112233".into(),
            "Living Room".into(),
        );
        let display = format!("{info}");
        assert!(display.contains("Living Room"));
        assert!(display.contains("192.168.1.100"));
    }

    #[test]
    fn test_device_info_deduplication() {
        let mut devices = Vec::new();
        let d1 = DeviceInfo::new(
            "192.168.1.100".into(),
            7000,
            "aabbcc112233".into(),
            "Room A".into(),
        );
        let d2 = DeviceInfo::new(
            "192.168.1.100".into(),
            7000,
            "aabbcc112233".into(),
            "Room A".into(),
        );
        devices.push(d1);
        assert!(devices.iter().any(|d| d.mac == d2.mac));
    }

    #[test]
    fn test_device_info_new_with_empty_name() {
        let info = DeviceInfo::new(
            "10.0.0.1".into(),
            7000,
            "aa:bb:cc:11:22:33".into(),
            "".into(),
        );
        assert_eq!(info.name, "aabbcc112233");
    }
}
