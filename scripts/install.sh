#!/bin/bash
# HauntMasterOS Installer: Choose your chain—Controller or Node? 🎃🔮
# Full repo for flex-frights; 720p cap for ancient iron. 
# Not licensed for commercial use, contact dev for more details.

set -e

# Console curses (unchanged)
RED='\033[0;31m' GREEN='\033[0;32m' YELLOW='\033[1;33m' NC='\033[0m'
log() { echo -e "${GREEN}[HAUNTMASTER]${NC} $1" | tee -a /var/log/hauntmaster-install.log; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1" | tee -a /var/log/hauntmaster-install.log; }
error() { echo -e "${RED}[ERROR]${NC} $1" | tee -a /var/log/hauntmaster-install.log; exit 1; }

# Clone the full crypt (always—flex fuel!)
INSTALL_DIR="/opt/hauntmaster"
if [ -d "$INSTALL_DIR" ]; then
    log "Refreshing the repo relics..."
    cd "$INSTALL_DIR" && git pull origin main
else
    log "Summoning full HauntMasterOS arsenal..."
    git clone https://github.com/Voodoohax/HauntmasterOS.git "$INSTALL_DIR"
fi
cd "$INSTALL_DIR"

# Dependencies: Apt & pip the potions
log "Brewing dependencies..."
apt update -qq
apt install -y curl git python3-pip network-manager hostapd dnsmasq iptables-persistent \
               python3-flask python3-jinja2 python3-requests python3-pytest sqlite3 \
               ffmpeg || error "Potions failed—stir your sources.list?"
pip3 install -r requirements.txt  # Flask, etc.—full for both roles

# Perf whisper: Sniff & nag on older models
MODEL=$(grep "^Model" /proc/cpuinfo | awk '{print $3}' | head -1)
if [[ "$MODEL" == *"Pi 3"* || "$MODEL" == *"Pi 2"* || "$MODEL" == *"Pi 1"* ]]; then
    warn "Elder Pi detected ($MODEL): Capped at 720p for spectral smoothness. Crave 1080p? Summon a Pi 4 or 5!"
    echo "720p-max" > /etc/hauntmaster/perf_cap.txt
fi

# Config crypt: Full etch
log "Carving configs..."
mkdir -p /etc/hauntmaster
cp -r configs/* /etc/hauntmaster/  # Services, themes, role stub

# Role crossroads: Summon selector on first boot
ROLE_FILE="/etc/hauntmaster/role.txt"
if [ ! -f "$ROLE_FILE" ]; then
    log "No role etched—booting fate forge (SSID + setup wizard)..."
    # Oneshot service: Runs selector.py, which beacons SSID & serves choice UI
    cp scripts/role_selector.py /opt/hauntmaster/bin/
    cat > /etc/systemd/system/haunt-setup-oneshot.service << EOF
[Unit]
Description=Haunt Role Selector
ConditionPathExists=!$ROLE_FILE

[Service]
Type=oneshot
ExecStart=/usr/bin/python3 /opt/hauntmaster/bin/role_selector.py
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
EOF
    systemctl daemon-reload && systemctl enable haunt-setup-oneshot.service
else
    ROLE=$(cat "$ROLE_FILE")
    log "Role recalled: $ROLE. Igniting services..."
fi

# Dynamic services: Post-choice (stubbed—selector handles)
if [ -f "$ROLE_FILE" ]; then
    if [ "$ROLE" = "controller" ]; then
        # Controller: AP + full UI wizard stub
        cp src/flask_app.py /opt/hauntmaster/bin/
        cp configs/systemd-haunt-controller.service /etc/systemd/system/
        systemctl daemon-reload && systemctl enable --now haunt-controller.service
        log "Controller crowned! Setup wizard awaits @ http://10.42.0.1:8080/network"
    elif [ "$ROLE" = "node" ]; then
        # Node: Adoption daemon
        cp scripts/node_adopt_nm.py /opt/hauntmaster/bin/
        cp configs/systemd-haunt-node.service /etc/systemd/system/
        systemctl daemon-reload && systemctl enable --now haunt-node.service
        log "Node bowed! Adoption beacon ready."
    fi
fi

# Wards & whispers
iptables -A INPUT -p tcp --dport 8080 -s 192.168.0.0/16 -j ACCEPT
netfilter-persistent save

if [ "$1" != "--pro" ]; then
    warn "Personal possession: Free for fiends. Pro for profit?"
fi

log "🌑 HauntMasterOS v0.2: Roles unbound! Reboot to crossroads. Docs: /docs/ROLE-CHOICE.md"
