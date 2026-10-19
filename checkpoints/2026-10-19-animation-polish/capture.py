from pathlib import Path
import subprocess,os
c=Path(__file__).resolve().parent;r=c.parents[1]
assert '"CGSSessionScreenIsLocked"=Yes' not in subprocess.check_output(['ioreg','-n','Root','-d1'],text=True),'Unlock Mac for capture'
p=subprocess.run([str(r/'rust/target/release/bbq_app'),'--shot',str(c/'pose.png'),'--big','--mode','ffa','--bots','3','--pose','polish','--cam','5,2,-6,140,-3,65','--burst','60','--burst-step','2','--look','polished','--style','pop'],cwd=r/'rust',env=dict(os.environ,BEVY_ASSET_ROOT=str(r/'rust/crates/bbq_app'),RUST_LOG='warn'),capture_output=True,text=True,timeout=180)
(c/'capture.log').write_text(p.stdout+p.stderr)
assert p.returncode==0 and 'ERROR' not in p.stderr
assert len(list(c.glob('pose_[0-9]*.png')))==60
print('Captured 60 actual renderer frames of controlled animation preview.')
