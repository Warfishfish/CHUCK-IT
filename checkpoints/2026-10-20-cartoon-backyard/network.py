import os,subprocess,time,socket
from pathlib import Path
c=Path(__file__).resolve().parent;r=c.parents[1]
if '"CGSSessionScreenIsLocked"=Yes' in subprocess.check_output(['ioreg','-n','Root','-d1'],text=True):raise SystemExit('Unlock the Mac before captures.')
with socket.socket() as s:s.bind(('127.0.0.1',0));port=s.getsockname()[1]
out=c/'network';out.mkdir(exist_ok=True)
(out/'appearance-packets.jsonl').write_text('')
processes=[];files=[]
try:
 log=open(out/'relay.log','w');files.append(log);relay=subprocess.Popen(['node','--require',str(c/'relay-observe.cjs'),'server.js'],cwd=r,env=dict(os.environ,PORT=str(port)),stdout=log,stderr=subprocess.STDOUT);processes.append(relay);time.sleep(1)
 for name,flags in [('host',['--net-host','raceverify','--net-play','8','--net-give','teddy','--leg-length','0','--face-length','0']),('guest',['--net-join','raceverify','--net-throw','--leg-length','100','--face-length','100'])]:
  log=open(out/(name+'.log'),'w');files.append(log)
  cmd=[str(r/'rust/target/release/bbq_app'),'--shot',str(out/(name+'.png')),'--review','--big','--look','polished','--style','pop','--menu','--preview-mode','ffa','--server',f'localhost:{port}','--net-debug','--capture-frame','2400',*flags]
  processes.append(subprocess.Popen(cmd,cwd=r/'rust',env=dict(os.environ,BEVY_ASSET_ROOT=str(r/'rust/crates/bbq_app'),RUST_LOG='warn'),stdout=log,stderr=subprocess.STDOUT));time.sleep(1)
 for p in processes[1:]:assert p.wait(timeout=180)==0
finally:
 for p in processes:
  if p.poll() is None:p.terminate()
 for p in processes:
  try:p.wait(timeout=5)
  except subprocess.TimeoutExpired:p.kill()
 for f in files:f.close()
for name in ['host','guest']:
 text=(out/(name+'.log')).read_text();assert 'ERROR' not in text and (out/(name+'.png')).exists(),text[-2000:]
assert 'mirror=true' in (out/'guest.log').read_text()
assert 'phase=Play' in (out/'guest.log').read_text()
assert 'NET-TEST guest let go of a throw' in (out/'guest.log').read_text()
import base64,json
rosters=[]
for line in (out/'appearance-packets.jsonl').read_text().splitlines():
 b=base64.b64decode(json.loads(line)['d'])
 if b[0] not in (2,4):continue
 p=5 if b[0]==2 else 1
 count=b[p];p+=1;lengths=[]
 for _ in range(count):
  p+=4;name_length=b[p];p+=1+name_length
  p+=2 # character and flags
  lengths.append(b[p+14]);p+=16
 rosters.append(lengths)
assert any(0 in lengths and 100 in lengths for lengths in rosters),rosters
(out/'verification.txt').write_text('Two local release instances connected through the actual relay and entered play.\nHost-accepted roster sent both 0% and 100% face lengths. Guest performed a throw.\nAll 101 face lengths separately pass appearance serialization tests. WAN conditions were not tested.\n')
print('Real relay play, guest throw and host-accepted 0%/100% appearance replication verified.')
