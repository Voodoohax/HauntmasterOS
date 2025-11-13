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

# Start API
cd /home/$USER/hauntmaster-phase1/phase1/backend
cargo run --release &

# Start UI in DEV MODE (proxy + hot reload)
cd /home/$USER/hauntmaster-phase1/phase1/ui
npm run dev -- --host 0.0.0.0 --port 8080
echo "Open: http://$(hostname -I | awk '{print $1}'):8080"
