use anyhow::{Result, anyhow};
use btleplug::api::{Central, Characteristic, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral};
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

use crate::models::Reading;

const ARANET4_SERVICE: Uuid = Uuid::from_u128(0x0000FCE0_0000_1000_8000_00805f9b34fb);
const SENSOR_CHARACTERISTIC: Uuid = Uuid::from_u128(0xF0CD3001_95DA_4F4B_9AC8_AA55D312AF0C);

pub struct Aranet4 {
    peripheral: Peripheral,
    sensor_char: Characteristic,
}

impl Aranet4 {
    pub async fn find_and_connect(name_filter: &str) -> Result<Self> {
        let manager = Manager::new().await?;
        let adapters = manager.adapters().await?;
        let adapter = adapters
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("No Bluetooth adapters found"))?;

        let device = find_device(&adapter, name_filter).await?;

        device.connect().await?;
        device.discover_services().await?;

        let sensor_char = find_characteristic(&device, ARANET4_SERVICE, SENSOR_CHARACTERISTIC)
            .ok_or_else(|| anyhow!("Sensor characteristic not found"))?;

        Ok(Self {
            peripheral: device,
            sensor_char,
        })
    }

    pub async fn read(&self) -> Result<Reading> {
        let data = self.peripheral.read(&self.sensor_char).await?;
        Reading::from_bytes(&data).ok_or_else(|| anyhow!("Failed to parse sensor data"))
    }

    pub async fn disconnect(&self) -> Result<()> {
        self.peripheral.disconnect().await?;
        Ok(())
    }
}

async fn find_device(adapter: &Adapter, name_filter: &str) -> Result<Peripheral> {
    adapter.start_scan(ScanFilter::default()).await?;
    sleep(Duration::from_secs(5)).await;
    adapter.stop_scan().await?;

    let peripherals = adapter.peripherals().await?;
    let name_filter_lower = name_filter.to_lowercase();

    for p in peripherals {
        if let Some(props) = p.properties().await? {
            if let Some(name) = props.local_name {
                if name.to_lowercase().contains(&name_filter_lower) {
                    return Ok(p);
                }
            }
        }
    }

    Err(anyhow!("No device found matching '{}'", name_filter))
}

fn find_characteristic(
    peripheral: &Peripheral,
    service_uuid: Uuid,
    char_uuid: Uuid,
) -> Option<Characteristic> {
    for service in peripheral.services() {
        if service.uuid == service_uuid {
            for char in service.characteristics {
                if char.uuid == char_uuid {
                    return Some(char);
                }
            }
        }
    }
    None
}
