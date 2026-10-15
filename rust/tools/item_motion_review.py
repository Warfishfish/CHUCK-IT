#!/usr/bin/env python3
"""Deterministic in-game drop/throw clips. Encode PNG frames separately; do not benchmark capture runs."""
import argparse,os,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--label',required=True);p.add_argument('--binary',type=Path,default=ROOT/'rust/target/release/bbq_app');p.add_argument('--scene',choices=['drop','throw','stress']);a=p.parse_args()
out=ROOT/'checkpoints/2026-10-14-item-physics/evidence';out.mkdir(parents=True,exist_ok=True)
env=os.environ.copy();env.update(BEVY_ASSET_ROOT=str(ROOT/'rust/crates/bbq_app'),RUST_LOG='warn')
for name in ['drop','throw','stress']:
 if a.scene and a.scene!=name:continue
 target=out/f'{a.label}-{name}.png'
 extra=['--motion-review',name,'--cam','0,3.8,17,0,-10,60'] if name!='stress' else ['--cam','0,5,20,0,-10,78','--mode','heist','--bots','11']
 cmd=[str(a.binary.resolve()),'--shot',str(target),'--review','--big','--look','polished','--style','pop','--cheeky',*extra]
 print('Capturing',a.label,name,flush=True)
 result=subprocess.run(cmd,cwd=ROOT/'rust',env=env,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,timeout=240)
 target.with_suffix('.log').write_text(result.stdout)
 expected=[out/f'{a.label}-{name}-{i:03}.png' for i in range(75)] if name!='stress' else [target.with_suffix('.metrics.txt')]
 if result.returncode or 'ERROR' in result.stdout or not all(p.exists() for p in expected):raise SystemExit(f'Capture failed: {target.with_suffix(".log")}')
 print('Saved',name,flush=True)
