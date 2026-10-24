import os,subprocess,pathlib,shutil
root=pathlib.Path('/Users/marcus/Claude/australian-bbq');out=root/'checkpoints/2026-10-24-race-shirt/evidence';raw=pathlib.Path('/tmp/bbq-race-shirt-motion');raw.mkdir(exist_ok=True)
if '"CGSSessionScreenIsLocked"=Yes' in subprocess.check_output(['ioreg','-n','Root','-d1'],text=True):raise SystemExit('Mac locked: captures deferred')
env=dict(os.environ,BEVY_ASSET_ROOT=str(root/'rust/crates/bbq_app'),RUST_LOG='warn')
for name,extra in [('carry',['--mode','race','--motion-review','race-carry','--cam','7,2,-16,0,-5,65']),('tug',['--mode','race','--motion-review','race-tug','--cam','7,2,-16,0,-5,65']),('grass',['--mode','race','--motion-review','race-grass','--cam','2,4,-13,0,-18,65'])]:
 target=raw/f'{name}.png'
 r=subprocess.run([str(root/'rust/target/release/bbq_app'),'--shot',str(target),'--review','--big',*extra],cwd=root/'rust',env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,timeout=150)
 (out/f'{name}.log').write_text(r.stdout)
 assert r.returncode==0 and 'ERROR' not in r.stdout,(name,r.stdout[-1500:])
 assert (raw/f'{name}-074.png').exists(),name
 for n in [0,10,20,32,45]:shutil.copy2(raw/f'{name}-{n:03}.png',out/f'{name}-{n:03}.png')
 print('Captured',name,flush=True)
for mode in ['ffa','teams','heist','smoko','race']:
 r=subprocess.run([str(root/'rust/target/release/bbq_app'),'--shot',str(out/f'shirt-{mode}.png'),'--review','--big','--mode',mode,'--bot','0','--bot-back','--bot-far','--capture-frame','360'],cwd=root/'rust',env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,timeout=150)
 (out/f'shirt-{mode}.log').write_text(r.stdout)
 assert r.returncode==0 and 'ERROR' not in r.stdout,(mode,r.stdout[-1500:])
 print('Captured',mode,'shirt and mode smoke check',flush=True)
