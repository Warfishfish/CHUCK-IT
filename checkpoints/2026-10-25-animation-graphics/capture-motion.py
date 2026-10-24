import pathlib,subprocess,os
root=pathlib.Path('/Users/marcus/Claude/australian-bbq');out=root/'checkpoints/2026-10-25-animation-graphics/evidence'
if '"CGSSessionScreenIsLocked"=Yes' in subprocess.check_output(['ioreg','-n','Root','-d1'],text=True):raise SystemExit('Mac locked; captures deferred')
env=dict(os.environ,BEVY_ASSET_ROOT=str(root/'rust/crates/bbq_app'),RUST_LOG='warn')
for name,args in [('charge-fall',['--motion-review','charge-fall','--cam','0,2,18,0,-5,40']),('magpie-flock',['--motion-review','magpie-flock','--mode','teams','--cam','0,4,20,0,-12,75'])]:
 r=subprocess.run([str(root/'rust/target/debug/bbq_app'),'--shot',str(out/(name+'.png')),'--review','--big','--look','polished','--style','pop',*args],cwd=root/'rust',env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,timeout=180)
 (out/(name+'.log')).write_text(r.stdout);print(name,r.returncode,flush=True);assert r.returncode==0 and 'ERROR' not in r.stdout,r.stdout[-2000:]
