#!/bin/bash
set -e

echo "🎃 Installing HauntMaster Phase 1 (Ubuntu/MPV Mode)..."

# Update system
sudo apt update && sudo apt upgrade -y

# Install dependencies (MPV for Ubuntu, OMX for Pi)
if [ -f /proc/device-tree/model ] && [[ "$(cat /proc/device-tree/model)" == *"Raspberry"* ]]; then
  sudo apt install -y omxplayer ffmpeg curl git build-essential libssl-dev pkg-config
else
  sudo apt install -y mpv ffmpeg curl git build-essential libssl-dev pkg-config
fi

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

# Install Node.js (for Vue build)
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs

# Clone repo
cd /home/$USER
git clone https://github.com/hauntmaster/phase1.git hauntmaster-phase1
cd hauntmaster-phase1

# Build UI
cd ui
npm install
npm run build
cd ..

# Build backend
cd backend
cargo build --release
cd ..

# Create media dirs
mkdir -p media thumbs

# Make start script executable
chmod +x start.sh

echo "✅ Installed! Run: ./start.sh"
echo "🌐 Open: http://$(hostname -I | awk '{print $1}')"
