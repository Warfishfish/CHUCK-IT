#!/usr/bin/env python3
"""Capture implemented race foundation and a matched Heist regression baseline."""
import argparse, os, subprocess
from pathlib import Path
root=Path(__file__).resolve().parents[2]
p=argparse.ArgumentParser();p.add_argument('--view',choices=['baseline','menu','racer','sideline','motion','results','standard']);a=p.parse_args()
state=subprocess.check_output(['ioreg','-n','Root','-d1'],text=True)
if '"CGSSessionScreenIsLocked"=Yes' in state:raise SystemExit('Unlock this Mac before capturing.')
out=root/'checkpoints/2026-10-16-race-foundation/evidence';out.mkdir(parents=True,exist_ok=True)
views={
 'baseline':['--cheeky','--mode','heist','--bots','11','--cam','0,5,20,0,-10,78'],
 'menu':['--menu','--preview-mode','race'],
 'racer':['--mode','race','--capture-frame','360'],
 'sideline':['--mode','race','--sideline','--capture-frame','360','--cam','0,35,64,0,-27,78'],
 'motion':['--mode','race','--sideline','--motion-review','race'],
 'results':['--mode','race','--sideline','--capture-frame','9000'],
 'standard':['--cheeky','--mode','heist','--bots','11','--cam','0,5,20,0,-10,78'],
}
for name,extra in views.items():
 if a.view and a.view!=name:continue
 binary=Path('/tmp/bbq-race-m1-before') if name=='baseline' else root/'rust/target/release/bbq_app'
 target=out/f'{name}.png'
 r=subprocess.run([str(binary),'--shot',str(target),'--review','--big','--look','polished','--style','pop',*extra],cwd=root/'rust',env=dict(os.environ,BEVY_ASSET_ROOT=str(root/'rust/crates/bbq_app'),RUST_LOG='warn'),text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,timeout=240)
 target.with_suffix('.log').write_text(r.stdout)
 assert r.returncode==0 and 'ERROR' not in r.stdout,r.stdout[-1500:]
 if name=='motion':assert all((out/f'motion-{i:03}.png').exists() for i in range(75))
 else:assert target.exists() and target.with_suffix('.metrics.txt').exists()
 print('Saved',name,flush=True)
