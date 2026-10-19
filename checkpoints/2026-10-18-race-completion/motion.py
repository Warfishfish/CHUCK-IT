import os, subprocess, re
from pathlib import Path
folder=Path(__file__).resolve().parent
root=folder.parents[1]
if '"CGSSessionScreenIsLocked"=Yes' in subprocess.check_output(['ioreg','-n','Root','-d1'],text=True):
 raise SystemExit('Unlock the Mac before captures.')
out=folder/'motion';out.mkdir(exist_ok=True)
cmd=[str(root/os.environ.get('BBQ_REVIEW_BINARY','rust/target/release/bbq_app')),'--shot',str(out/'water.png'),'--review','--big','--look','polished','--style','pop','--mode','ffa','--bots','1','--motion-review','water-balloon','--bot','0','--bot-far','--bot-side']
p=subprocess.run(cmd,cwd=root/'rust',env=dict(os.environ,BEVY_ASSET_ROOT=str(root/'rust/crates/bbq_app'),RUST_LOG='warn'),stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,timeout=240)
(out/'capture.log').write_text(p.stdout)
assert p.returncode==0 and 'ERROR' not in p.stdout,p.stdout[-3000:]
assert (out/'water-104.png').exists()
samples=[(int(f),float(w),float(v)) for f,w,v in re.findall(r'WATER_REVIEW frame=(\d+) wet=([\d.]+) speed=([\d.]+)',p.stdout)]
assert any(w>0 and abs(v-4.96)<0.02 for f,w,v in samples), 'No measured wet slowdown in recording'
assert any(f>470 and w==0 and abs(v-6.2)<0.02 for f,w,v in samples), 'Normal speed did not return'
print('Verified actual collision, 4.96m/s wet movement, expiry and 6.2m/s recovery.')
