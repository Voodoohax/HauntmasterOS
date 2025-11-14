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

# Start Legacy UI (on :80)
cd /home/$USER/hauntmaster/ui
npm run dev -- --host 0.0.0.0 --port 80 &

# Start Composer (on :81)
cd /home/$USER/hauntmaster/composer
npm run dev -- --host 0.0.0.0 --port 81
