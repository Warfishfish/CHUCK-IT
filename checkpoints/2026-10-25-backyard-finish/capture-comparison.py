import pathlib,subprocess,os
r=pathlib.Path('/Users/marcus/Claude/australian-bbq');out=r/'checkpoints/2026-10-25-backyard-finish/evidence'
if '"CGSSessionScreenIsLocked"=Yes' in subprocess.check_output(['ioreg','-n','Root','-d1'],text=True):raise SystemExit('Mac locked')
env=dict(os.environ,BEVY_ASSET_ROOT=str(r/'rust/crates/bbq_app'),RUST_LOG='warn')
for phase in ['before','after']:
 for name,cam in [('gameplay','-16,3,16,-25,-6,70'),('close','-20,2,-6,40,-5,65')]:
  binary='/tmp/bbq-before-backyard-finish' if phase=='before' else str(r/'rust/target/release/bbq_app')
  args=[binary,'--shot',str(out/(phase+'-'+name+'.png')),'--review','--big','--look','polished','--style','pop','--cam',cam,'--capture-frame','360']
  if phase=='after':args+=['--backyard-finish']
  p=subprocess.run(args,cwd=r/'rust',env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,timeout=180)
  (out/(phase+'-'+name+'.log')).write_text(p.stdout);print(phase,name,p.returncode,flush=True);assert p.returncode==0 and 'ERROR' not in p.stdout
p=subprocess.run([str(r/'rust/target/release/bbq_app'),'--shot',str(out/'moving.png'),'--review','--big','--look','polished','--style','pop','--backyard-finish','--motion-review','charge-release','--cam','-4,2,18,-15,-6,65','--cam-drift'],cwd=r/'rust',env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,timeout=180)
(out/'moving.log').write_text(p.stdout);print('moving',p.returncode,flush=True);assert p.returncode==0 and 'ERROR' not in p.stdout
