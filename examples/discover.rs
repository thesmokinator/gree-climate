use gree_climate::discover;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    println!("Scanning for GREE devices on the local network...");
    println!("(This may take a few seconds)\n");

    let devices = discover().await?;

    println!("Found {} device(s):\n", devices.len());
    for device in &devices {
        println!("  {}", device);
        println!("    IP:      {}", device.ip);
        println!("    Port:    {}", device.port);
        println!("    MAC:     {}", device.mac);
        println!("    Model:   {:?}", device.model);
        println!("    Version: {:?}", device.version);
        println!();
    }

    Ok(())
}
