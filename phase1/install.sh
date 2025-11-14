#!/bin/bash
set -e

echo "🎃   Installing HauntMaster OS — Alpha   🎃"
echo " VLC + FFmpeg + Scene Composer + Auto-Start"

# === 1. SYSTEM UPDATE ===
sudo apt update && sudo apt full-upgrade -y

# === 2. DEPENDENCIES ===
sudo apt install -y \
    ffmpeg \
    vlc vlc-bin vlc-plugin-base \
    webp \
    curl \
    git \
    build-essential \
    libssl-dev \
    pkg-config \
    nodejs npm

# === 3. RUST ===
if ! command -v cargo &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# === 4. CLONE REPO ===
cd /home/$USER
if [ -d "hauntmaster" ]; then
    echo "Updating existing repo..."
    cd hauntmaster
    git pull
else
    echo "Cloning HauntMasterOS..."
    git clone https://github.com/Voodoohax/HauntmasterOS.git hauntmaster
    cd hauntmaster
fi

# === 5. BUILD (BACKEND) ===
echo "Building Backend..."
cd /backend
cargo build --release
cd ../..

# === 6. BUILD (COMPOSER UI) ===
echo "Building Composer..."
cd composer
npm install
npm run build
cd ../..

# === 7. CREATE DIRS & PERMS ===
mkdir -p media thumbs
chmod 777 media thumbs

# === 8. DOWNLOAD LATEST FILES ===
echo "Syncing latest main.rs + scene.rs + App.vue + Canvas.vue..."
curl -o backend/src/main.rs https://raw.githubusercontent.com/Voodoohax/HauntmasterOS/main/backend/src/main.rs
curl -o backend/src/scene.rs https://raw.githubusercontent.com/Voodoohax/HauntmasterOS/main/backend/src/scene.rs
curl -o composer/src/App.vue https://raw.githubusercontent.com/Voodoohax/HauntmasterOS/main/composer/src/App.vue
curl -o composer/src/Canvas.vue https://raw.githubusercontent.com/Voodoohax/HauntmasterOS/main/composer/src/Canvas.vue

# === 9. REBUILD BACKEND AFTER SYNC ===
cd backend
cargo build --release
cd ..

# === 10. AUTO-START SERVICE ===
echo "Creating systemd service..."
sudo tee /etc/systemd/system/hauntmaster.service > /dev/null <<EOF
[Unit]
Description=HauntMaster OS
After=network.target

[Service]
Type=simple
User=$USER
WorkingDirectory=/home/$USER/hauntmaster
ExecStart=/home/$USER/hauntmaster/backend/target/release/hauntmaster-backend
ExecStartPost=/bin/sleep 5 && /usr/bin/xdg-open http://localhost:80
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable hauntmaster.service

# === 11. COMPOSER START SCRIPT ===
echo "Creating composer start script..."
cat > start.sh <<'EOF'
#!/bin/bash
echo "🎃 Starting HauntMaster..."

# Start backend
cd /home/$USER/hauntmaster/backend
cargo run --release &

# Start composer UI
cd /home/$USER/hauntmaster/composer
npm run dev -- --host 0.0.0.0 --port 80
EOF
chmod +x start-composer.sh

# === 12. FINAL INSTRUCTIONS ===
echo ""
echo "INSTALL COMPLETE!"
echo ""
echo "AUTO-START ON BOOT:"
echo "   sudo systemctl start hauntmaster.service"
echo ""
echo "OPEN IN BROWSER:"
echo "   http://$(hostname -I | awk '{print $1}'):80"
./start.sh"
