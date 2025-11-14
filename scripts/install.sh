#!/bin/bash
# HauntMasterOS Installer: SBC cluster for digital decor media.
# Licensed under MIT for personal use only, contact developer for more information on commercial licensing

set -e

# Logging
LOG_FILE="/var/log/hauntmaster-install.log"
exec > >(tee -a "$LOG_FILE") 2>&1
log() { echo "[INSTALL] $1"; }
warn() { echo "[WARN] $1"; }
error() { echo "[ERROR] $1"; exit 1; }

# Clone/update repo
INSTALL_DIR="/opt/hauntmaster"
if [ -d "$INSTALL_DIR" ]; then
    log "Updating repo..."
    cd "$INSTALL_DIR" && git pull origin main
else
    log "Cloning repo..."
    git clone https://github.com/Voodoohax/HauntmasterOS.git "$INSTALL_DIR"
fi
cd "$INSTALL_DIR"

# Dependencies
log "Installing system deps..."
apt update -qq
apt install -y curl git python3-pip network-manager hostapd dnsmasq iptables-persistent \
               avahi-daemon avahi-utils sqlite3 ffmpeg vlc python3-vlc udisks2 \
                || error "System deps failed."

# Python deps
pip3 install -r requirements.txt  # pyaml, pyudev, python-vlc, flask, zeroconf, requests

# Configs
log "Setting up configs..."
mkdir -p /etc/hauntmaster
cp -r configs/* /etc/hauntmaster/  # role.yaml template, etc.
chmod 644 /etc/hauntmaster/*

# Scripts & Services
cp scripts/*.py /opt/hauntmaster/scripts/
chmod +x /opt/hauntmaster/scripts/*.py

cp configs/haunt-boot.service /etc/systemd/system/
systemctl daemon-reload
systemctl enable haunt-boot.service

# Udev for USB wand
cp configs/udev-wand.rules /etc/udev/rules.d/99-haunt-wand.rules
udevadm control --reload-rules

# Firewall basics
iptables -A INPUT -p tcp --dport 8080 -s 192.168.0.0/16 -j ACCEPT
iptables -A INPUT -p tcp --dport 5000 -s 192.168.0.0/16 -j ACCEPT
netfilter-persistent save

log "Install complete. Reboot to start boot workflow."
