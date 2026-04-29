# ⚡TmxMon

![Demo](./assets/demo.gif)

A blazingly fast, modern TUI system monitor originally built for Termux — rewritten in Rust using `ratatui`. 

While optimized for Android hardware environments, TmxMon features graceful native fallbacks, making it a fully cross-platform monitoring dashboard for Windows and Linux desktops as well.

## Features
- **Overview** — CPU, Memory, Storage, Battery gauges + Network speed + Device info
- **CPU** — Overall + per-core gauges, model, and live frequency stats
- **Memory** — RAM & Swap gauges with detailed usage breakdown
- **Storage** — Disk usage + built-in interactive file explorer
- **Battery** — Charge level, status, health, temperature, and current 
- **Network** — Live upload/download speed, IP, and total data transferred
- **Processes** — Top 20 processes sorted by live CPU usage
- **Settings** — Adjustable refresh rate & battery capacity configs


## Android / Termux Installation

Due to Android's strict security sandboxing, TmxMon requires a companion app to bridge the terminal with your phone's hardware sensors (like the battery).

### Step 1: Where to Download the Apps
⚠️ **Important:** Do NOT download Termux from the Google Play Store (it is deprecated and broken).

**Option A: F-Droid (Recommended)**
1. Download **Termux**: [F-Droid Link](https://f-droid.org/packages/com.termux/)
2. Download **Termux:API**: [F-Droid Link](https://f-droid.org/packages/com.termux.api/)


### Step 2: Install Terminal Packages & Run
Open the Termux:API app once after installation (it may show a blank screen; this is normal). Then, return to Termux and run the following commands:

```bash
# Install required dependencies
pkg update
pkg install rust git termux-api
git clone [https://github.com/Andrew-Velox/TmxMon.git](https://github.com/Andrew-Velox/TmxMon.git)
cd TmxMon
cargo run --release
```

## Keybindings

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate tabs |
| `Tab` | Next tab |
| `Enter` | Open file explorer (Storage tab) |
| `Esc` | Close file explorer |
| `←` / `→` | Adjust setting value (Settings tab) |
| `r` | Reset settings to defaults |
| `q` | Quit |

## Dependencies
- `ratatui` — TUI framework
- `crossterm` — Terminal control
- `sysinfo` — Core system info gathering
- `chrono` — Date/time formatting
- `anyhow` — Error handling

## Notes
- **Android:** Battery info requires `termux-battery-status` (install via the `termux-api` package). Device info requires standard Android `getprop`.
- **Desktop:** On Windows and Linux, TmxMon automatically bypasses Termux dependencies and uses native WMI/sysfs to read hardware and battery data.