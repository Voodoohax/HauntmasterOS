#!/usr/bin/env python3
# who_am_i.py: Boot-time role detector and router.

import os
import subprocess
import sys
import yaml
from pathlib import Path

CONFIG_DIR = '/etc/hauntmaster'
ROLE_FILE = os.path.join(CONFIG_DIR, 'role.yaml')
SCRIPTS_DIR = '/opt/hauntmaster/scripts'

def load_role():
    if not os.path.exists(ROLE_FILE):
        return None
    with open(ROLE_FILE, 'r') as f:
        return yaml.safe_load(f)

def route_to_script(script_name):
    script_path = os.path.join(SCRIPTS_DIR, f"{script_name}.py")
    if not os.path.exists(script_path):
        print(f"Error: Script {script_path} not found.", file=sys.stderr)
        sys.exit(1)
    subprocess.run([sys.executable, script_path])

if __name__ == '__main__':
    role = load_role()
    if role is None:
        # No role: New device
        route_to_script('i_am_new')
    elif role.get('role') == 'controller':
        # Start controller
        route_to_script('i_am_controller')
    elif role.get('role') == 'prop':
        # Check controller for prop
        if is_controller_reachable(role):
            route_to_script('i_am_prop')
        else:
            route_to_script('i_am_alone')
    else:
        print(f"Unknown role: {role}", file=sys.stderr)
        route_to_script('i_am_new')

def is_controller_reachable(role):
    # Stub: Ping controller_ip from role.yaml via mDNS or direct ping
    controller_ip = role.get('controller_ip', 'controller.local')
    try:
        result = subprocess.run(['ping', '-c', '1', '-W', '2', controller_ip], capture_output=True)
        return result.returncode == 0
    except:
        return False
