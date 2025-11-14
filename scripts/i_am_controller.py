#!/usr/bin/env python3
# i_am_controller.py: Start controller services.

import subprocess
import yaml

CONFIG_DIR = '/etc/hauntmaster'
ROLE_FILE = os.path.join(CONFIG_DIR, 'role.yaml')

if __name__ == '__main__':
    with open(ROLE_FILE, 'r') as f:
        role = yaml.safe_load(f)
    # Start net host if not set
    if 'net_ssid' not in role:
        # Stub: Launch net setup UI
        subprocess.Popen(['python3', '/opt/hauntmaster/src/net_setup.py'])
    # Start main services
    subprocess.run(['systemctl', 'start', 'haunt-controller'])
    subprocess.run(['systemctl', 'start', 'haunt-media-server'])
    print("Controller mode active.")
