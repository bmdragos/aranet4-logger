mod ble;
mod db;
mod models;

use anyhow::Result;
use chrono::DateTime;
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

    /// Export all readings to CSV and exit
    #[arg(long)]
    export: bool,

    /// Show statistics and exit
    #[arg(long)]
    stats: bool,
}

fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/")
        && let Some(home) = home::home_dir()
    {
        return home.join(rest);
    }
    PathBuf::from(path)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let db_path = expand_tilde(&args.database);

    // Export mode - no BLE needed
    if args.export {
        let db = Database::open(&db_path)?;
        let mut stdout = std::io::stdout();
        let count = db.export_csv(&mut stdout)?;
        eprintln!("Exported {} readings", count);
        return Ok(());
    }

    // Stats mode - no BLE needed
    if args.stats {
        let db = Database::open(&db_path)?;
        match db.stats()? {
            Some(stats) => {
                println!("Readings: {}", stats.count);
                println!(
                    "CO2:      {:.0} ppm avg (min: {}, max: {})",
                    stats.avg_co2, stats.min_co2, stats.max_co2
                );
                println!("Temp:     {:.1}°C avg", stats.avg_temp);
                println!("Humidity: {:.0}% avg", stats.avg_humidity);
                if let (Some(first), Some(last)) = (stats.first_reading, stats.last_reading) {
                    println!("Period:   {} to {}", first, last);
                }
            }
            None => {
                println!("No readings yet");
            }
        }
        return Ok(());
    }

    // Normal mode - connect to Aranet4 and log
    eprintln!("Scanning for {}...", args.name);
    let device = Aranet4::find_and_connect(&args.name).await?;

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

    // Check for stale data (same measurement we already have)
    let db = Database::open(&db_path)?;
    if let Some(last_ts) = db.last_timestamp()?
        && let Ok(last_time) = DateTime::parse_from_rfc3339(&last_ts)
    {
        let diff = reading.timestamp.signed_duration_since(last_time.to_utc());
        if diff.abs().num_seconds() < 60 {
            eprintln!("Skipped (stale - same measurement as last reading)");
            return Ok(());
        }
    }

    db.insert(&reading)?;
    eprintln!("Saved to {}", db_path.display());

    Ok(())
}
