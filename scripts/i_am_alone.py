#!/usr/bin/env python3
# i_am_alone.py: Offline mode - play resting, monitor controller.

import subprocess
import yaml
import time
from vlc import Instance, MediaPlayer

CONFIG_DIR = '/etc/hauntmaster'
ROLE_FILE = os.path.join(CONFIG_DIR, 'role.yaml')
MEDIA_DIR = '/opt/hauntmaster/media'

def is_reachable():
    with open(ROLE_FILE, 'r') as f:
        role = yaml.safe_load(f)
    controller_ip = role.get('controller_ip', 'controller.local')
    result = subprocess.run(['ping', '-c', '1', controller_ip], capture_output=True)
    return result.returncode == 0

if __name__ == '__main__':
    # Play resting media
    resting_file = os.path.join(MEDIA_DIR, 'resting.mp4')
    if os.path.exists(resting_file):
        instance = Instance('--loop')
        player = instance.media_player_new()
        media = instance.media_new(resting_file)
        player.set_media(media)
        player.play()
    # Monitor loop
    while True:
        if is_reachable():
            print("Controller found - rebooting to normal mode.")
            subprocess.run(['reboot'])
        time.sleep(30)
    print("Isolation mode: Resting active, monitoring.")
