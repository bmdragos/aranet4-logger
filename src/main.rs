mod ble;
mod db;
mod models;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

use ble::Aranet4;
use db::Database;

#[derive(Parser)]
#[command(name = "aranet4-logger")]
#[command(about = "Log Aranet4 CO2 sensor readings to SQLite")]
struct Args {
    /// Database file path
    #[arg(short, long, default_value = "~/aranet4.db")]
    database: String,

    /// Device name filter
    #[arg(short, long, default_value = "Aranet4")]
    name: String,

    /// Output reading as JSON to stdout
    #[arg(long)]
    json: bool,
}

fn expand_tilde(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        if let Some(home) = home::home_dir() {
            return home.join(&path[2..]);
        }
    }
    PathBuf::from(path)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let db_path = expand_tilde(&args.database);

    // Connect to Aranet4
    eprintln!("Scanning for {}...", args.name);
    let device = Aranet4::find_and_connect(&args.name).await?;

    // Read sensor data
    let reading = device.read().await?;
    device.disconnect().await?;

    // Output
    if args.json {
        println!("{}", serde_json::to_string_pretty(&reading)?);
    } else {
        eprintln!(
            "CO2: {} ppm | Temp: {:.1}°C | Humidity: {}% | Pressure: {:.1} hPa | Battery: {}%",
            reading.co2_ppm,
            reading.temperature_c,
            reading.humidity_percent,
            reading.pressure_hpa,
            reading.battery_percent
        );
    }

    // Save to database
    let db = Database::open(&db_path)?;
    db.insert(&reading)?;
    eprintln!("Saved to {}", db_path.display());

    Ok(())
}
