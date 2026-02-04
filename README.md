# aranet4-logger

A simple CLI tool to log [Aranet4](https://aranet.com/products/aranet4/) CO2 sensor readings to a SQLite database via Bluetooth LE.

## Features

- Connects to Aranet4 via Bluetooth LE
- Logs CO2, temperature, humidity, pressure, and battery level
- Stores readings in SQLite for easy querying and analysis
- Single-shot design - schedule with cron/launchd/systemd for continuous logging
- JSON output mode for piping to other tools

## Installation

### From crates.io

```bash
cargo install aranet4-logger
```

### From source

```bash
git clone https://github.com/bmdragos/aranet4-logger.git
cd aranet4-logger
cargo build --release
```

The binary will be at `target/release/aranet4-logger`.

### Pre-built binaries

Download from [GitHub Releases](https://github.com/bmdragos/aranet4-logger/releases).

## Usage

```bash
# Log a single reading (scans for any Aranet4 device)
aranet4-logger

# Specify database location
aranet4-logger --database /path/to/readings.db

# Filter by device name (useful if you have multiple Aranet4s)
aranet4-logger --name "Aranet4 1D016"

# Output as JSON (doesn't save to database)
aranet4-logger --json
```

### Options

| Flag | Description | Default |
|------|-------------|---------|
| `-d, --database <PATH>` | SQLite database path | `~/aranet4.db` |
| `-n, --name <NAME>` | Device name filter | `Aranet4` |
| `--json` | Output reading as JSON to stdout | - |

## Scheduled Logging

### macOS (launchd)

Create `~/Library/LaunchAgents/com.aranet4.logger.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.aranet4.logger</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/aranet4-logger</string>
    </array>
    <key>StartInterval</key>
    <integer>300</integer>
    <key>StandardErrorPath</key>
    <string>/tmp/aranet4-logger.err</string>
</dict>
</plist>
```

Then load it:

```bash
launchctl load ~/Library/LaunchAgents/com.aranet4.logger.plist
```

### Linux (systemd)

Create `~/.config/systemd/user/aranet4-logger.service`:

```ini
[Unit]
Description=Aranet4 CO2 Logger

[Service]
Type=oneshot
ExecStart=/usr/local/bin/aranet4-logger
```

Create `~/.config/systemd/user/aranet4-logger.timer`:

```ini
[Unit]
Description=Log Aranet4 readings every 5 minutes

[Timer]
OnBootSec=1min
OnUnitActiveSec=5min

[Install]
WantedBy=timers.target
```

Then enable it:

```bash
systemctl --user enable --now aranet4-logger.timer
```

## Platform Setup

### macOS

Bluetooth should work out of the box. You may need to grant Bluetooth permissions to your terminal app in System Preferences > Privacy & Security > Bluetooth.

### Linux

You need BlueZ installed and your user must have permissions to access Bluetooth:

```bash
# Install BlueZ (Debian/Ubuntu)
sudo apt install bluez

# Add your user to the bluetooth group
sudo usermod -aG bluetooth $USER

# Log out and back in for group changes to take effect
```

### Windows

Bluetooth LE support requires Windows 10 or later. Run from an administrator terminal if you encounter permission issues.

## Querying the Data

The SQLite database has a simple schema:

```sql
SELECT * FROM readings ORDER BY timestamp DESC LIMIT 10;
```

Example queries:

```bash
# Last 24 hours average CO2
sqlite3 ~/aranet4.db "SELECT AVG(co2_ppm) FROM readings WHERE timestamp > datetime('now', '-24 hours');"

# Hourly averages
sqlite3 ~/aranet4.db "SELECT strftime('%Y-%m-%d %H:00', timestamp) as hour, AVG(co2_ppm) as avg_co2 FROM readings GROUP BY hour ORDER BY hour DESC LIMIT 24;"

# Export to CSV
sqlite3 -header -csv ~/aranet4.db "SELECT * FROM readings;" > readings.csv
```

## Database Schema

```sql
CREATE TABLE readings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    co2_ppm INTEGER NOT NULL,
    temperature_c REAL NOT NULL,
    humidity_percent INTEGER NOT NULL,
    pressure_hpa REAL NOT NULL,
    battery_percent INTEGER NOT NULL
);
```

## License

MIT
