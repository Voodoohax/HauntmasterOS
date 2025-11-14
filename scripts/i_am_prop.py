#!/usr/bin/env python3
# i_am_prop.py: Start prop playback and heartbeat.

import subprocess
import yaml
import time
from omxplayer.player import OMXPlayer  # pip: python-omx

CONFIG_DIR = '/etc/hauntmaster'
ROLE_FILE = os.path.join(CONFIG_DIR, 'role.yaml')
MEDIA_DIR = '/opt/hauntmaster/media'

if __name__ == '__main__':
    with open(ROLE_FILE, 'r') as f:
        role = yaml.safe_load(f)
    # Start GPIO listener for interactives
    subprocess.Popen(['python3', '/opt/hauntmaster/scripts/gpio_listener.py'])
    # Heartbeat to controller
    def heartbeat():
        while True:
            subprocess.run(['curl', '-X', 'POST', f"http://{role['controller_ip']}:5000/api/heartbeat", '-d', '{"status": "alive"}'])
            time.sleep(30)
    import threading
    threading.Thread(target=heartbeat, daemon=True).start()
    # Default to resting media
    resting_file = os.path.join(MEDIA_DIR, 'resting.mp4')  # From last push
    if os.path.exists(resting_file):
        player = OMXPlayer(resting_file, loop=True)
    print("Prop mode active.")
