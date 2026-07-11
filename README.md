# gree-climate

[![Crates.io](https://img.shields.io/crates/v/gree-climate)](https://crates.io/crates/gree-climate)
[![Docs.rs](https://docs.rs/gree-climate/badge.svg)](https://docs.rs/gree-climate)

An idiomatic, fully async Rust library for controlling GREE air conditioners over the local network. Implements the GREE local protocol used by the Gree+ app and Home Assistant's `greeclimate` integration.

## Features

- **Device discovery** via UDP broadcast
- **Pairing & authentication** with automatic cipher detection (V1 AES-ECB / V2 AES-GCM)
- **Read state** — power, mode, temperatures, fan speed, swing, turbo, and more
- **Send commands** — typed API with enums, no magic strings
- **Cross-platform** — Linux, macOS, Windows
- **Pure async** via Tokio

## Quick Start

```rust
use gree_climate::{discover, Client, Mode, FanSpeed};

#[tokio::main]
async fn main() -> Result<(), gree_climate::Error> {
    // Discover devices on the LAN
    let devices = discover().await?;
    let device = devices.into_iter().next()
        .ok_or(gree_climate::Error::NoDevicesFound)?;

    // Connect and authenticate
    let mut ac = Client::connect(device).await?;
    ac.bind().await?;

    // Read current state
    let state = ac.state();
    println!("Power: {}, Temp: {}°C", state.power, state.target_temperature);

    // Send commands
    ac.set_power(true).await?;
    ac.set_mode(Mode::Cool).await?;
    ac.set_temperature(23).await?;
    ac.set_fan_speed(FanSpeed::Auto).await?;

    Ok(())
}
```

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
gree-climate = "0.1"
```

## Examples

```bash
# Discover devices on the network
cargo run --example discover

# Control a device (interactive or by index/IP)
cargo run --example control
cargo run --example control -- --index 0
cargo run --example control -- --ip 192.168.1.100
```

## Protocol

GREE devices communicate over UDP port 7000 using JSON-encoded packets. The inner payload is encrypted with AES (ECB for V1, GCM for V2) and base64-encoded. This library handles all encryption, serialization, and transport automatically.
