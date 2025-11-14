#!/usr/bin/env python3
# USB Wand Monitor: Plug the phylactery, pledge the Pi!
import json
import subprocess
import time
import os
from pyudev import Context, Monitor  # pip: pyudev
import inotify.adapters  # pip: inotify (for file watch)

CONFIG_DIR = '/etc/hauntmaster'
ROLE_FILE = f'{CONFIG_DIR}/role.txt'
JOIN_FILE = 'join.us'
WAND_MOUNT = '/media/wand'  # Auto-mount point

def parse_wand(join_path):
    """Decipher the wand's writ."""
    with open(join_path, 'r') as f:
        data = json.load(f)
    return data.get('ssid'), data.get('passphrase'), data.get('controller_ip'), data.get('token'), data.get('media_paths', [])

def join_via_wand(ssid, passphrase, controller_ip, token):
    """Wand-wrought joining—nmcli + register."""
    # Connect net
    subprocess.run(['nmcli', 'dev', 'wifi', 'connect', ssid, 'password', passphrase], check=True)
    time.sleep(5)  # Net settle
    
    # Register (HTTP to controller)
    mac = get_mac()  # From prior
    payload = {'mac': mac, 'role': 'node', 'token': token}
    resp = requests.post(f'http://{controller_ip}:5000/api/props/wand-register', json=payload)
    if resp.status_code == 201:
        with open(ROLE_FILE, 'w') as f: f.write('node')
        print("🪄 Wand worked! Node nestled—respining services.")
        subprocess.run(['systemctl', 'restart', 'haunt-node.service'], check=False)  # Or dynamic
        return True
    return False

def clone_media(media_paths, local_dir='/opt/hauntmaster/media'):
    """Slurp wand's media bounty."""
    os.makedirs(local_dir, exist_ok=True)
    for path in media_paths:
        wand_file = os.path.join(WAND_MOUNT, path)
        if os.path.exists(wand_file):
            subprocess.run(['rsync', '-av', wand_file, local_dir], check=True)
    print("📼 Media mirrored—feast local!")

def monitor_usb():
    """Udev + inotify vigil."""
    context = Context()
    monitor = Monitor.from_netlink(context)
    monitor.filter_by(subsystem='block', action='add')
    
    for device in iter(monitor.poll, None):
        if 'ID_FS_LABEL' in device and device.get('ID_FS_LABEL', '') == 'HAUNTWAND':  # Label USB
            mount_point = f'/media/{os.uname().nodename}'  # Or fixed
            subprocess.run(['mount', device.device_node, WAND_MOUNT], check=False)
            
            join_path = os.path.join(WAND_MOUNT, JOIN_FILE)
            if os.path.exists(join_path):
                ssid, passphrase, ctrl_ip, token, media = parse_wand(join_path)
                if join_via_wand(ssid, passphrase, ctrl_ip, token):
                    clone_media(media)
                    subprocess.run(['umount', WAND_MOUNT], check=False)  # Eject
                    print("💀 Wand waved—node awakened!")
                    return  # One-and-done per boot

if __name__ == '__main__':
    # Role guard: Skip if already node
    if os.path.exists(ROLE_FILE) and open(ROLE_FILE).read().strip() == 'node':
        print("👻 Already bowed—no wand needed.")
        exit(0)
    monitor_usb()
