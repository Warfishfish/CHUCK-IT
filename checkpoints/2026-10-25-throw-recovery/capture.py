import pathlib,subprocess,os
r=pathlib.Path('/Users/marcus/Claude/australian-bbq');out=r/'checkpoints/2026-10-25-throw-recovery/evidence';out.mkdir(exist_ok=True)
if '"CGSSessionScreenIsLocked"=Yes' in subprocess.check_output(['ioreg','-n','Root','-d1'],text=True):raise SystemExit('Mac locked')
env=dict(os.environ,BEVY_ASSET_ROOT=str(r/'rust/crates/bbq_app'),RUST_LOG='warn')
for name,args in [('charge-release',['--cam','0,2,18,0,-5,40']),('charge-camera',[])]:
 p=subprocess.run([str(r/'rust/target/debug/bbq_app'),'--shot',str(out/(name+'.png')),'--review','--big','--look','polished','--style','pop','--motion-review',name,*args],cwd=r/'rust',env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,timeout=180)
 (out/(name+'.log')).write_text(p.stdout);print(name,p.returncode,flush=True);assert p.returncode==0 and 'ERROR' not in p.stdout
