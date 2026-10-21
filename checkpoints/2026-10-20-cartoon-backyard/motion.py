from pathlib import Path
import subprocess,os
c=Path(__file__).resolve().parent;r=c.parents[1]
assert '\"CGSSessionScreenIsLocked\"=Yes' not in subprocess.check_output(['ioreg','-n','Root','-d1'],text=True),'Unlock Mac'
for name,mode,args in [('balloon','held-balloon',[]),('magpie','magpie',['--cam','0,2.6,18,0,-3,80'])]:
 for f in c.glob(name+'-[0-9][0-9][0-9].png'):f.unlink()
 p=subprocess.run([str(r/'rust/target/release/bbq_app'),'--shot',str(c/(name+'.png')),'--review','--big','--look','polished','--style','pop','--mode','ffa','--bots','1','--motion-review',mode,*args],cwd=r/'rust',env=dict(os.environ,BEVY_ASSET_ROOT=str(r/'rust/crates/bbq_app'),RUST_LOG='warn'),capture_output=True,text=True,timeout=240)
 (c/(name+'.log')).write_text(p.stdout+p.stderr)
 assert p.returncode==0 and 'ERROR' not in p.stderr,p.stderr[-2000:]
 assert len(list(c.glob(name+'-[0-9][0-9][0-9].png')))==75
 if name=='magpie':assert 'score=-10' in p.stdout,p.stdout
 print(name,'motion captured',flush=True)
