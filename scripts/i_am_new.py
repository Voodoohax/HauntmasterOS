#!/usr/bin/env python3
# i_am_new.py: Handle unconfigured device - USB join or OOBE beacon.

import os
import json
import subprocess
import time
import yaml
from pathlib import Path

CONFIG_DIR = '/etc/hauntmaster'
ROLE_FILE = os.path.join(CONFIG_DIR, 'role.yaml')
WAND_FILE = 'join.us'
WAND_MOUNT = '/media/usb'  # Assume auto-mount; use udev for prod

def check_magic_wand():
    if os.path.exists(WAND_MOUNT) and os.path.exists(os.path.join(WAND_MOUNT, WAND_FILE)):
        with open(os.path.join(WAND_MOUNT, WAND_FILE), 'r') as f:
            data = json.load(f)
        # Join net
        subprocess.run(['nmcli', 'dev', 'wifi', 'connect', data['ssid'], 'password', data['passphrase']])
        # Etch role
        role_data = {'role': 'prop', 'controller_ip': data['controller_ip']}
        with open(ROLE_FILE, 'w') as f:
            yaml.dump(role_data, f)
        # Clone media if present
        if 'media' in data:
            subprocess.run(['rsync', '-av', f"{WAND_MOUNT}/media/", '/opt/hauntmaster/media/'])
        subprocess.run(['umount', WAND_MOUNT])
        print("Joined via magic wand.")
        return True
    return False

def start_beacon_and_oobe():
    # Start adoption SSID (NM hotspot)
    mac = subprocess.check_output(['ip', 'link', 'show', 'wlan0']).decode()
    ssid = f"HauntAdopt-{mac.split()[-1][-6:]}"
    subprocess.run(['nmcli', 'dev', 'wifi', 'hotspot', 'ifname', 'wlan0', 'ssid', ssid, 'password', ''])
    # Launch OOBE Flask UI (stub: Serve role choice form at 10.42.0.1:8080)
    subprocess.Popen(['python3', '/opt/hauntmaster/src/oobe_ui.py'])
    print(f"OOBE beacon started: {ssid}")

if __name__ == '__main__':
    os.makedirs(CONFIG_DIR, exist_ok=True)
    if check_magic_wand():
        # Reboot to route via who_am_i
        subprocess.run(['reboot'])
    else:
        start_beacon_and_oobe()
        # Monitor USB loop (every 10s)
        while True:
            time.sleep(10)
            check_magic_wand()
