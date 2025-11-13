#!/bin/bash
echo "Starting HauntMaster..."

# === 1. ENSURE RUST IS IN PATH ===
if ! command -v cargo &> /dev/null; then
    echo "cargo not found — sourcing Rust env..."
    if [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
    else
        echo "Rust not installed! Run install.sh first."
        exit 1
    fi
fi

# === 2. START BACKEND (Rust API on :3000) ===
echo "Starting API on :3000..."
cd /home/$USER/hauntmaster-phase1/phase1/backend
cargo run --release &

# === 3. SERVE UI ON PORT 8080 (NO SUDO NEEDED) ===
echo "Serving UI on :8080..."
cd /home/$USER/hauntmaster-phase1/phase1/ui/dist
python3 -m http.server 8080
