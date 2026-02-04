use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;

use crate::models::Reading;

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
}
