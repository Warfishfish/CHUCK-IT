from pathlib import Path
import subprocess,os
c=Path(__file__).resolve().parent;r=c.parents[1]
assert '"CGSSessionScreenIsLocked"=Yes' not in subprocess.check_output(['ioreg','-n','Root','-d1'],text=True),'Unlock Mac for capture'
(c/'frames').mkdir(exist_ok=True)
p=subprocess.run([str(r/'rust/target/release/bbq_app'),'--shot',str(c/'frames'/'drift.png'),'--big','--mode','ffa','--bots','3', '--cam','0,2.8,13,0,-4,85','--cam-drift','--burst','60','--burst-step','2','--look','polished','--style','pop'],cwd=r/'rust',env=dict(os.environ,BEVY_ASSET_ROOT=str(r/'rust/crates/bbq_app'),RUST_LOG='warn'),capture_output=True,text=True,timeout=180)
(c/'capture.log').write_text(p.stdout+p.stderr)
assert p.returncode==0 and 'ERROR' not in p.stderr
assert len(list((c/'frames').glob('drift_[0-9]*.png')))==60
print('Captured 60 actual renderer frames of camera drift across grass, foliage and buildings.')
