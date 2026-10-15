#!/usr/bin/env python3
"""Capture staged first-person idle/walk/run/turn/stop input, plus separate performance."""
import argparse,os,subprocess
from pathlib import Path
root=Path(__file__).resolve().parents[2]
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--label',required=True);p.add_argument('--binary',type=Path,default=root/'rust/target/release/bbq_app');p.add_argument('--output-dir',type=Path);p.add_argument('--scene',choices=['held-noodle','held-rubber','stress']);a=p.parse_args()
out=(a.output_dir or root/'checkpoints/2026-10-14-held-motion/evidence').resolve()
out.mkdir(parents=True,exist_ok=True)
env=os.environ.copy();env.update(BEVY_ASSET_ROOT=str(root/'rust/crates/bbq_app'),RUST_LOG='warn')
for name in ['held-noodle','held-rubber','stress']:
    if a.scene and a.scene != name: continue
    target=out/f'{a.label}-{name}.png'
    extra=['--motion-review',name] if name!='stress' else ['--cam','0,5,20,0,-10,78','--mode','heist','--bots','11']
    r=subprocess.run([str(a.binary.resolve()),'--shot',str(target),'--review','--big','--look','polished','--style','pop','--cheeky',*extra],cwd=root/'rust',env=env,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,timeout=240)
    target.with_suffix('.log').write_text(r.stdout)
    expected=[out/f'{a.label}-{name}-{i:03}.png' for i in range(75)] if name!='stress' else [target.with_suffix('.metrics.txt')]
    if r.returncode or 'ERROR' in r.stdout or not all(p.exists() for p in expected): raise SystemExit(f'Capture failed: {target.with_suffix(".log")}')
    print('Saved',a.label,name,flush=True)
