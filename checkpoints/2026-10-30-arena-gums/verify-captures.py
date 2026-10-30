import pathlib,subprocess,os
r=pathlib.Path(__file__).resolve().parents[2];out=pathlib.Path(__file__).parent/'evidence'
if '"CGSSessionScreenIsLocked"=Yes' in subprocess.check_output(['ioreg','-n','Root','-d1'],text=True):raise SystemExit('Mac locked')
env=dict(os.environ,BEVY_ASSET_ROOT=str(r/'rust/crates/bbq_app'),RUST_LOG='warn')
binary=str(r/'rust/target/release/bbq_app')
for mode in ['ffa']:
 args=[binary,'--mode',mode,'--shot',str(out/(mode+'.png')),'--big','--look','polished','--style','pop']
 p=subprocess.run(args,cwd=r/'rust',env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,timeout=120)
 (out/(mode+'.log')).write_text(p.stdout);print(mode,p.returncode,flush=True);assert p.returncode==0 and 'ERROR' not in p.stdout
for name,extra in [('moving',['--motion-review','smoko','--cam','-18,4,17,-21,5,65','--cam-drift'])]:
 p=subprocess.run([binary,'--mode','smoko','--shot',str(out/(name+'.png')),'--review','--big','--look','polished','--style','pop',*extra],cwd=r/'rust',env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,timeout=180)
 (out/(name+'.log')).write_text(p.stdout);print(name,p.returncode,flush=True);assert p.returncode==0 and 'ERROR' not in p.stdout
