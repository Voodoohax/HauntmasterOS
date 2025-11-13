#!/bin/bash
echo "Starting HauntMaster..."

# Source Rust
if ! command -v cargo &> /dev/null && [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

# Start API
cd /home/$USER/hauntmaster-phase1/phase1/backend
cargo run --release &

# Start UI in DEV MODE (proxy + hot reload)
cd /home/$USER/hauntmaster-phase1/phase1/ui
npm run dev -- --host 0.0.0.0 --port 8080
echo "Open: http://$(hostname -I | awk '{print $1}'):8080"
