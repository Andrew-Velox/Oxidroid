#!/data/data/com.termux/files/usr/bin/bash
# Termux Monitor - Install Script
set -e

echo "🚀 Installing Termux Monitor..."

# Install Rust if not present
if ! command -v cargo &>/dev/null; then
    echo "📦 Installing Rust toolchain..."
    pkg install rust -y
fi

# Build
echo "🔨 Building (this may take a few minutes)..."
cargo build --release

# Copy binary
cp target/release/termux-monitor "$PREFIX/bin/termux-monitor"
chmod +x "$PREFIX/bin/termux-monitor"

echo "✅ Done! Run with: termux-monitor"
