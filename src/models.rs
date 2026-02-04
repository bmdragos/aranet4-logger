use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Reading {
    pub timestamp: DateTime<Utc>,
    pub co2_ppm: u16,
    pub temperature_c: f32,
    pub humidity_percent: u8,
    pub pressure_hpa: f32,
    pub battery_percent: u8,
}

impl Reading {
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 8 {
            return None;
        }

        let co2_ppm = u16::from_le_bytes([data[0], data[1]]);
        let temp_raw = u16::from_le_bytes([data[2], data[3]]);
        let pressure_raw = u16::from_le_bytes([data[4], data[5]]);
        let humidity_percent = data[6];
        let battery_percent = data[7];

        Some(Self {
            timestamp: Utc::now(),
            co2_ppm,
            temperature_c: temp_raw as f32 * 0.05,
            humidity_percent,
            pressure_hpa: pressure_raw as f32 * 0.1,
            battery_percent,
        })
    }
}
