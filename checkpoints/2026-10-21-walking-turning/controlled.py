from pathlib import Path
import subprocess,os,sys,json
c=Path(__file__).resolve().parent;r=c.parents[1];phase='controlled'
assert '"CGSSessionScreenIsLocked"=Yes' not in subprocess.check_output(['ioreg','-n','Root','-d1'],text=True),'Unlock Mac'
folder=c/phase;folder.mkdir(exist_ok=True)
base=r
p=subprocess.run([str(base/'rust/target/release/bbq_app'),'--shot',str(folder/'walk.png'),'--big','--mode','ffa','--bots','3','--cam','0,2.6,-8,180,-4,85','--pose','locomotion','--burst','200','--burst-step','3','--look','polished','--style','pop'],cwd=r/'rust',env=dict(os.environ,BEVY_ASSET_ROOT=str(r/'rust/crates/bbq_app'),RUST_LOG='warn'),capture_output=True,text=True,timeout=240)
(folder/'capture.log').write_text(p.stdout+p.stderr)
assert p.returncode==0 and 'ERROR' not in p.stderr
assert len(list(folder.glob('walk_[0-9]*.png')))==200
print(phase,'200 controlled animation frames captured',flush=True)
