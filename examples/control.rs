use std::io::{self, Write};

use gree_climate::{discover, Client, FanSpeed, Mode, SwingVertical};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = std::env::args().collect();

    let device = if let Some(pos) = args.iter().position(|a| a == "--ip") {
        let ip = args.get(pos + 1).expect("--ip requires an IP address");
        println!("Resolving device at {ip}...");
        let devices = discover().await?;
        devices
            .into_iter()
            .find(|d| d.ip == *ip)
            .ok_or_else(|| format!("No device found at {ip}"))?
    } else if let Some(pos) = args.iter().position(|a| a == "--index") {
        let idx: usize = args
            .get(pos + 1)
            .expect("--index requires a number")
            .parse()?;
        let devices = discover().await?;
        devices
            .into_iter()
            .nth(idx)
            .ok_or_else(|| format!("Device index {idx} out of range"))?
    } else {
        let devices = discover().await?;
        println!("\nFound {} device(s):\n", devices.len());
        for (i, d) in devices.iter().enumerate() {
            println!("  [{i}] {d}");
        }
        println!();
        let idx = pick_device(devices.len())?;
        devices.into_iter().nth(idx).unwrap()
    };

    println!("Connecting to {device}...\n");
    let mut ac = Client::connect(device.clone()).await?;

    println!("Binding to device...");
    ac.bind().await?;

    println!("Reading current state...");
    ac.refresh().await?;

    let state = ac.state();
    println!("\nCurrent device state:");
    println!("  Power:     {:?}", state.power);
    println!("  Mode:      {:?}", state.mode);
    println!("  Target:    {}°C", state.target_temperature);
    println!("  Current:   {:?}°C", state.current_temperature);
    println!("  Fan speed: {:?}", state.fan_speed);
    println!("  Vert swing: {:?}", state.swing_vertical);
    println!("  Horiz swing: {:?}", state.swing_horizontal);
    println!("  Turbo:     {:?}", state.turbo);
    println!("  Quiet:     {:?}", state.quiet);
    println!("  Sleep:     {:?}", state.sleep);
    println!("  Firmware:  {:?}", state.firmware_version);
    println!();

    println!("Sending commands to the device...\n");

    println!("  set_power(true)");
    ac.set_power(true).await?;

    println!("  set_mode(Cool)");
    ac.set_mode(Mode::Cool).await?;

    println!("  set_temperature(23)");
    ac.set_temperature(23).await?;

    println!("  set_fan_speed(Auto)");
    ac.set_fan_speed(FanSpeed::Auto).await?;

    println!("  set_swing_vertical(FullSwing)");
    ac.set_swing_vertical(SwingVertical::FullSwing).await?;

    println!("  set_turbo(false)");
    ac.set_turbo(false).await?;

    println!("\nAll commands sent successfully!");
    Ok(())
}

fn pick_device(count: usize) -> Result<usize, Box<dyn std::error::Error>> {
    loop {
        print!("Select device [0-{}]: ", count - 1);
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let trimmed = input.trim();
        if let Ok(idx) = trimmed.parse::<usize>() {
            if idx < count {
                return Ok(idx);
            }
        }
        eprintln!("Invalid selection. Enter a number between 0 and {}.", count - 1);
    }
}
