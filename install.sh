#!/bin/bash

echo "⚡ Installing Oxidroid..."

# 1. Install system dependencies
pkg update
pkg install termux-api wget -y

# 2. Download the latest release binary
wget https://github.com/Andrew-Velox/Oxidroid/releases/download/v0.1.3/oxidroid

# 3. Set permissions and move to bin
chmod +x oxidroid
mv oxidroid $PREFIX/bin/

echo "✅ Installation complete! Just type 'oxidroid' to start."