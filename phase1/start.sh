#!/bin/bash
echo "🎃 Starting HauntMaster..."

# Start backend
cd /home/$USER/hauntmaster-phase1/phase1/backend
cargo run --release &

# Serve UI
cd /home/$USER/hauntmaster-phase1/phase1/ui/dist
python3 -m http.server 80
