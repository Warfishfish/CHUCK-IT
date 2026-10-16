#!/usr/bin/env python3
"""Actual Smoko UI, map, delivery and performance captures (no settings writes)."""
import argparse, os, subprocess
from pathlib import Path
# Metal window screenshots are black while macOS is locked. Preserve prior evidence.
state=subprocess.run(['ioreg','-n','Root','-d1'],capture_output=True,text=True,check=True).stdout
if '"CGSSessionScreenIsLocked"=Yes' in state:
 raise SystemExit('Unlock this Mac before capturing: locked-session Metal captures are black.')
root=Path(__file__).resolve().parents[2]
p=argparse.ArgumentParser();p.add_argument('--view',choices=['menu','map','delivery','stress','standard','menu-play']);a=p.parse_args()
out=root/'checkpoints/2026-10-15-smoko-mode/evidence';out.mkdir(parents=True,exist_ok=True)
views={
 'menu':['--menu','--preview-mode','smoko'],
 'menu-play':['--menu','--preview-mode','smoko','--autoplay','--cam','0,5,20,0,-14,78'],
 'map':['--mode','smoko','--cam','0,5,20,0,-14,78'],
 'delivery':['--mode','smoko','--motion-review','smoko'],
 'stress':['--mode','smoko','--bots','11','--cam','0,5,20,0,-10,78'],
 'standard':['--cheeky','--mode','heist','--bots','11','--cam','0,5,20,0,-10,78'],
}
for name,extra in views.items():
 if a.view and name!=a.view:continue
 target=out/f'after-{name}.png'
 r=subprocess.run([str(root/'rust/target/release/bbq_app'),'--shot',str(target),'--review','--big','--look','polished','--style','pop',*extra],cwd=root/'rust',env=dict(os.environ,BEVY_ASSET_ROOT=str(root/'rust/crates/bbq_app'),RUST_LOG='warn'),text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,timeout=240)
 target.with_suffix('.log').write_text(r.stdout)
 assert r.returncode==0 and 'ERROR' not in r.stdout, target.with_suffix('.log')
 if name=='delivery':
  assert all((out/f'after-delivery-{i:03}.png').exists() for i in range(75))
  assert 'SMOKO_REVIEW final_score=100 carries=0' in r.stdout,r.stdout[-1000:]
 else:assert target.exists() and target.with_suffix('.metrics.txt').exists()
 print('Saved',name,flush=True)
