import pathlib,subprocess,os
r=pathlib.Path('/Users/marcus/Claude/australian-bbq');out=r/'checkpoints/2026-10-25-animation-graphics/evidence'
if '"CGSSessionScreenIsLocked"=Yes' in subprocess.check_output(['ioreg','-n','Root','-d1'],text=True):raise SystemExit('Mac locked; captures deferred')
env=dict(os.environ,BEVY_ASSET_ROOT=str(r/'rust/crates/bbq_app'),RUST_LOG='warn')
for name,binary,args in [('before-matched','/tmp/bbq-before-animation-graphics',[]),('after-matched',str(r/'rust/target/release/bbq_app'),[]),('teams',str(r/'rust/target/release/bbq_app'),['--mode','teams']),('heist',str(r/'rust/target/release/bbq_app'),['--mode','heist']),('smoko',str(r/'rust/target/release/bbq_app'),['--mode','smoko']),('race',str(r/'rust/target/release/bbq_app'),['--mode','race'])]:
 p=subprocess.run([binary,'--shot',str(out/(name+'.png')),'--review','--big','--look','polished','--style','pop','--capture-frame','360',*args],cwd=r/'rust',env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,timeout=180)
 (out/(name+'.log')).write_text(p.stdout);assert p.returncode==0 and 'ERROR' not in p.stdout,p.stdout[-1500:];print(name,'verified',flush=True)
