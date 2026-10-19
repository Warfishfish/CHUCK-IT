from pathlib import Path
import subprocess,os
c=Path(__file__).resolve().parent;r=c.parents[1]
assert '"CGSSessionScreenIsLocked"=Yes' not in subprocess.check_output(['ioreg','-n','Root','-d1'],text=True),'Unlock Mac'
p=subprocess.run([str(r/'rust/target/release/bbq_app'),'--shot',str(c/'climb.png'),'--review','--big','--look','polished','--style','pop','--mode','ffa','--bots','1','--cam','-21,2.6,-7,35,-5,65','--motion-review','height'],cwd=r/'rust',env=dict(os.environ,BEVY_ASSET_ROOT=str(r/'rust/crates/bbq_app'),RUST_LOG='warn'),capture_output=True,text=True,timeout=240)
(c/'motion.log').write_text(p.stdout+p.stderr)
assert p.returncode==0 and 'ERROR' not in p.stderr
assert 'y=2.300 grounded=true' in p.stdout,p.stdout[-1500:]
assert len(list(c.glob('climb-[0-9][0-9][0-9].png')))==75
print('Actual climbing replay reached and stood on the upper box.')
