# Oxidroid

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Platform](https://img.shields.io/badge/platform-Android%20%7C%20Linux%20%7C%20Windows-lightgrey)
![Rust](https://img.shields.io/badge/rust-stable-orange.svg)
![Release](https://img.shields.io/github/v/release/Andrew-Velox/Oxidroid)

**Oxidroid** is a fast, polished Terminal User Interface (TUI) system monitor built in Rust. Originally created for Android via Termux, it now runs smoothly on Linux and Windows too.

![Oxidroid UI](assets/demo.gif)

## Highlights

* **Zero-Configuration Native Execution:** Custom-built binaries for Termux (Bionic libc), Linux (musl static), and Windows. No missing library errors.
* **Modern Aesthetic:** Clean, responsive layout with a sharp, anime-inspired vibe.
* **High Performance:** Rust-powered efficiency for low overhead and responsive updates.
* **Cross-Platform:** Monitor your phone, your desktop, or your server with the exact same tool.

---

## Installation

Oxidroid is distributed as a single, standalone binary. Download the correct file for your OS, then run it using the appropriate commands below.

### Android (Termux)
Install Termux and Termux:API from F-Droid before proceeding:

- Download **Termux**: [F-Droid Link](https://f-droid.org/packages/com.termux/)
- Download **Termux:API**: [F-Droid Link](https://f-droid.org/packages/com.termux.api/)

Install dependencies:
```bash
pkg install wget termux-api -y
```

Download the binary:
```bash
wget -O $PREFIX/bin/oxidroid https://github.com/Andrew-Velox/Oxidroid/releases/download/v0.1.8/oxidroid-android-aarch64
```

Run it:
```bash
chmod +x $PREFIX/bin/oxidroid
hash -r
oxidroid
```

### Linux (Desktop / Server)
Download the binary:
```bash
wget https://github.com/Andrew-Velox/Oxidroid/releases/download/v0.1.8/oxidroid-linux-x86_64
```

Run it:
```bash
chmod +x oxidroid-linux-x86_64
./oxidroid-linux-x86_64
```

### Windows
Download the binary:
```powershell
Invoke-WebRequest -Uri "https://github.com/Andrew-Velox/Oxidroid/releases/download/v0.1.8/oxidroid-windows.exe" -OutFile "oxidroid.exe"
```

Run it:
```powershell
.\oxidroid.exe
```
*(Note: Windows SmartScreen may flag the `.exe` since it is a new open-source binary. Click "More info" -> "Run anyway").*

---

## Usage

Once installed, run:
```bash
oxidroid
```

## Building from Source

If you prefer to compile the project yourself, ensure you have [Rust](https://www.rust-lang.org/tools/install) installed, then run:
```bash
git clone https://github.com/Andrew-Velox/Oxidroid.git
cd Oxidroid
cargo build --release
```
The compiled binary will be located in `target/release/oxidroid`.

---

## Contributing

Contributions, issues, and feature requests are always welcome!

1. **Fork** the project.
2. **Create** your feature branch: `git checkout -b feature/AmazingFeature`
3. **Commit** your changes: `git commit -m 'Add some AmazingFeature'`
4. **Push** to the branch: `git push origin feature/AmazingFeature`
5. **Open** a Pull Request.

---


