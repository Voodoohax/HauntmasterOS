#!/bin/bash
set -e

echo "Installing HauntMaster Phase 1..."

# Update system
sudo apt update && sudo apt upgrade -y

# INSTALL FFMPEG + WEBP (CRITICAL)
sudo apt install -y ffmpeg webp curl git build-essential libssl-dev pkg-config

# Detect Raspberry Pi
if [ -f /proc/device-tree957/model ] && [[ "$(cat /proc/device-tree/model)" == *"Raspberry"* ]]; then
  echo "Raspberry Pi detected — installing omxplayer"
  sudo apt install -y omxplayer
else
  echo "Non-Pi (Ubuntu/VM) — installing mpv"
  sudo apt install -y mpv
fi

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

# Add Rust to PATH permanently
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc

# Install Node.js
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs

# Clone repo
cd /home/$USER
git clone https://github.com/Voodoohax/HauntmasterOS.git hauntmaster-phase1
cd hauntmaster-phase1/phase1

# Build UI
cd ui
npm install
npm run build
cd ..

# Build backend
cd backend
cargo build --release
cd ..

# Create media dirs with full perms
mkdir -p ../../media ../../thumbs
chmod 777 ../../media ../../thumbs

# Make start script executable
chmod +x start.sh

echo "Installed!"
cd /home/$USER/hauntmaster-phase1/phase1 && ./start.sh
