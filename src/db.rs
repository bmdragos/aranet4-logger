use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;

use crate::models::Reading;

pub struct Stats {
    pub count: u64,
    pub avg_co2: f64,
    pub min_co2: u16,
    pub max_co2: u16,
    pub avg_temp: f64,
    pub avg_humidity: f64,
    pub first_reading: Option<String>,
    pub last_reading: Option<String>,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS readings (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                co2_ppm INTEGER NOT NULL,
                temperature_c REAL NOT NULL,
                humidity_percent INTEGER NOT NULL,
                pressure_hpa REAL NOT NULL,
                battery_percent INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_readings_timestamp
            ON readings(timestamp);
            ",
        )?;
        Ok(())
    }

    pub fn insert(&self, reading: &Reading) -> Result<()> {
        self.conn.execute(
            "INSERT INTO readings (timestamp, co2_ppm, temperature_c, humidity_percent, pressure_hpa, battery_percent)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                reading.timestamp.to_rfc3339(),
                reading.co2_ppm,
                reading.temperature_c,
                reading.humidity_percent,
                reading.pressure_hpa,
                reading.battery_percent,
            ),
        )?;
        Ok(())
    }

    pub fn last_reading(&self) -> Result<Option<(u16, u8, u8)>> {
        let mut stmt = self.conn.prepare(
            "SELECT co2_ppm, humidity_percent, battery_percent FROM readings ORDER BY id DESC LIMIT 1",
        )?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            Ok(Some((row.get(0)?, row.get(1)?, row.get(2)?)))
        } else {
            Ok(None)
        }
    }

    pub fn stats(&self) -> Result<Option<Stats>> {
        let mut stmt = self.conn.prepare(
            "SELECT
                COUNT(*),
                AVG(co2_ppm),
                MIN(co2_ppm),
                MAX(co2_ppm),
                AVG(temperature_c),
                AVG(humidity_percent),
                MIN(timestamp),
                MAX(timestamp)
             FROM readings",
        )?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            let count: u64 = row.get(0)?;
            if count == 0 {
                return Ok(None);
            }
            Ok(Some(Stats {
                count,
                avg_co2: row.get(1)?,
                min_co2: row.get(2)?,
                max_co2: row.get(3)?,
                avg_temp: row.get(4)?,
                avg_humidity: row.get(5)?,
                first_reading: row.get(6)?,
                last_reading: row.get(7)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn export_csv<W: std::io::Write>(&self, writer: &mut W) -> Result<u64> {
        writeln!(
            writer,
            "timestamp,co2_ppm,temperature_c,humidity_percent,pressure_hpa,battery_percent"
        )?;
        let mut stmt = self.conn.prepare(
            "SELECT timestamp, co2_ppm, temperature_c, humidity_percent, pressure_hpa, battery_percent
             FROM readings ORDER BY timestamp",
        )?;
        let mut rows = stmt.query([])?;
        let mut count = 0u64;
        while let Some(row) = rows.next()? {
            let ts: String = row.get(0)?;
            let co2: u16 = row.get(1)?;
            let temp: f64 = row.get(2)?;
            let humidity: u8 = row.get(3)?;
            let pressure: f64 = row.get(4)?;
            let battery: u8 = row.get(5)?;
            writeln!(
                writer,
                "{},{},{:.2},{},{:.1},{}",
                ts, co2, temp, humidity, pressure, battery
            )?;
            count += 1;
        }
        Ok(count)
    }
}
