import subprocess,os,time,pathlib
root=pathlib.Path('/Users/marcus/Claude/australian-bbq');out=root/'checkpoints/2026-10-24-race-shirt/evidence'
if '"CGSSessionScreenIsLocked"=Yes' in subprocess.check_output(['ioreg','-n','Root','-d1'],text=True):raise SystemExit('Mac locked')
env=dict(os.environ,BEVY_ASSET_ROOT=str(root/'rust/crates/bbq_app'),RUST_LOG='warn')
base=[str(root/'rust/target/release/bbq_app'),'--menu','--preview-mode','race','--server','ws://localhost:3000','--net-debug','--net-race-social']
handles=[];procs=[]
try:
 for role,args in [('host',['--net-host','shirt24','--net-play','12','--name','HostBack']),('guest',['--net-join','shirt24','--net-ready','--name','GuestBack'])]:
  f=(out/f'network-{role}.log').open('w');handles.append(f)
  procs.append(subprocess.Popen(base+args,cwd=root/'rust',env=env,stdout=f,stderr=subprocess.STDOUT))
  time.sleep(1.5)
 time.sleep(28)
finally:
 for p in procs:
  p.terminate()
 for p in procs:
  try:p.wait(8)
  except subprocess.TimeoutExpired:p.kill();p.wait()
 for f in handles:f.close()
for role in ['host','guest']:
 text=(out/f'network-{role}.log').read_text()
 print(role, '\n'.join(line for line in text.splitlines() if 'NET-RACE' in line or 'asked:' in line)[-6500:])
 assert 'ERROR' not in text
