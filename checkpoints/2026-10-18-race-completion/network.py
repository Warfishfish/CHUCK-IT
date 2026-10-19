import os,subprocess,time,socket
from pathlib import Path
c=Path(__file__).resolve().parent;r=c.parents[1]
if '"CGSSessionScreenIsLocked"=Yes' in subprocess.check_output(['ioreg','-n','Root','-d1'],text=True):raise SystemExit('Unlock the Mac before captures.')
with socket.socket() as s:s.bind(('127.0.0.1',0));port=s.getsockname()[1]
out=c/'network';out.mkdir(exist_ok=True)
processes=[];files=[]
try:
 log=open(out/'relay.log','w');files.append(log);relay=subprocess.Popen(['node','server.js'],cwd=r,env=dict(os.environ,PORT=str(port)),stdout=log,stderr=subprocess.STDOUT);processes.append(relay);time.sleep(1)
 for name,flags in [('host',['--net-host','raceverify','--net-play','8']),('guest',['--net-join','raceverify','--sideline','--net-throw'])]:
  log=open(out/(name+'.log'),'w');files.append(log)
  cmd=[str(r/'rust/target/release/bbq_app'),'--shot',str(out/(name+'.png')),'--review','--big','--look','polished','--style','pop','--menu','--preview-mode','race','--server',f'localhost:{port}','--net-debug','--capture-frame','2400',*flags]
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
print('Two local game instances connected through the real relay and entered play.')
