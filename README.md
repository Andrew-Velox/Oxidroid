# 🚀 TmxMon

![Demo](./assets/demo.gif)

A fast, modern TUI dashboard for Termux — rewritten in Rust using `ratatui`.

## Features
- **Overview** — CPU, Memory, Storage, Battery gauges + Network speed + Device info
- **CPU** — Overall + per-core gauges, model, frequency stats
- **Memory** — RAM & Swap gauges with detailed breakdown
- **Storage** — Disk usage + built-in interactive file explorer
- **Battery** — Charge level, status, health, temperature, current (via `termux-battery-status`)
- **Network** — Live upload/download speed, IP, total transferred
- **Processes** — Top 20 processes by CPU usage
- **Settings** — Adjustable refresh rate & battery capacity

## Install & Build

```bash
pkg install rust
pkg install git
git clone https://github.com/Andrew-Velox/TmxMon.git
cd TmxMon
cargo run --release
```



| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate tabs |
| `Tab` | Next tab |
| `Enter` | Open file explorer (Storage tab) |
| `Esc` | Close file explorer |
| `←` / `→` | Adjust setting value (Settings tab) |
| `r` | Reset settings to defaults |
| `q` | Quit |

## Dependencies (auto-fetched by Cargo)
- `ratatui` — TUI framework
- `crossterm` — Terminal control
- `sysinfo` — System info
- `chrono` — Date/time
- `anyhow` — Error handling

## Notes
- Battery info requires `termux-battery-status` (from `termux-api` package)
- Device info requires `getprop` (standard on Android/Termux)
