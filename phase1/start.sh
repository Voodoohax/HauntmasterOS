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

# Start backend API (port 3000)
cd /home/$USER/hauntmaster/backend
cargo run --release &
BACKEND_PID=$!

# Wait for API
sleep 5

# Start Legacy UI (port 80)
cd /home/$USER/hauntmaster/ui
npm run dev -- --host 0.0.0.0 --port 80 &
UI_PID=$!

# Start Composer (port 81)
cd /home/$USER/hauntmaster/composer
npm run dev -- --host 0.0.0.0 --port 81 &
COMPOSER_PID=$!

echo ""
echo "🎃   HauntMaster OS - LIVE   🎃"
echo "┌──────────────────────────────┐"
echo "│ API + Files      :3000       │"
echo "│ Legacy UI        :80         │"
echo "│ Scene Composer   :81         │"
echo "└──────────────────────────────┘"
echo ""
echo "Open Composer:    http://$(hostname -I | awk '{print $1}'):81"
echo "Open Legacy UI:   http://$(hostname -I | awk '{print $1}'):80"
echo "API Docs:         http://$(hostname -I | awk '{print $1}'):3000/api/media"
echo ""
