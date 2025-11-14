#!/usr/bin/env python3
# Role Selector: The boot crossroads—beacon & choose!
from flask import Flask, request, render_template_string
# ... (summon_hotspot, get_mac from node_adopt_nm.py—import here)

app = Flask(__name__)
ROLE_FILE = '/etc/hauntmaster/role.txt'

HTML_CHOICE = '''
<!DOCTYPE html>
<html><head><title>👻 Choose Your Haunt Role!</title>
<style>body{background:#111;color:#f90;text-align:center;padding:50px;} button{background:#900;color:#fff;padding:15px;border:none;cursor:pointer;} button:hover{background:#c00;} input[type=radio]{margin:10px;}</style></head>
<body><h1>HauntMaster: Crown or Kneel?</h1>
<p>Full powers await, pick & possess!</p>
<form method="POST"><label><input type="radio" name="role" value="controller" checked> 👑 Controller (Boss the Coven)</label><br>
<label><input type="radio" name="role" value="node"> 🧟 Node (Join the Horde)</label><br><button>Seal the Fate!</button></form>
{% if message %}<p style="color:{{'green' if success else 'red'}};">{{message}}</p>{% endif %}</body></html>
'''

@app.route('/setup', methods=['GET', 'POST'])
def choose_role():
    if request.method == 'POST':
        role = request.form['role']
        with open(ROLE_FILE, 'w') as f: f.write(role)
        success = True
        message = f"{role.title()} etched! Rebooting services..."
        # Trigger role services (subprocess.call(['systemctl', 'start', f'haunt-{role}']))
        return redirect(url_for('done'))  # Stub /done: "Fate forged!"
    return render_template_string(HTML_CHOICE, message=None)

if __name__ == '__main__':
    # Beacon SSID always on first boot
    mac_suffix = get_mac()
    summon_hotspot(SSID_TEMPLATE.format(mac_suffix))
    app.run(host=ADOPT_IP, port=8080)
