#!/usr/bin/env python3
"""M2 actual game captures. Run sequentially, after builds finish; no settings writes."""
import argparse
import os
from pathlib import Path
import subprocess
ROOT = Path(__file__).resolve().parents[2]
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--label', required=True)
p.add_argument('--binary', type=Path, default=ROOT/'rust/target/release/bbq_app')
p.add_argument('--view')
a = p.parse_args()
out = ROOT/'checkpoints/2026-10-14-graphics-m2/evidence'
out.mkdir(parents=True, exist_ok=True)
base = ['--bot','0','--bot-head','--char','classic','--mouth','smile','--brows','flat','--hair','none','--hat','none']
views = {m: base+['--expression', m] for m in ['normal','grit','ouch','sip','cheer']}
for char in ['pear','egg','gumdrop']:
 views[char] = ['--bot','0','--bot-wide' if char=='egg' else '--bot-far','--char',char,'--mouth','grin','--brows','arched','--hair','tuft','--hat','none','--expression','cheer']
views['profile'] = base+['--bot-side','--expression','cheer']
views['none'] = ['--bot','0','--bot-head','--char','classic','--mouth','none','--brows','none','--hair','none','--hat','none','--expression','grit']
views['stress'] = ['--cam','0,5,20,0,-10,78','--mode','heist','--bots','11','--cheeky']
views['current'] = base+['--expression','normal','--style','current']
if a.view and a.view not in views: p.error('Unknown view')
env = os.environ.copy()
env.update(BEVY_ASSET_ROOT=str(ROOT/'rust/crates/bbq_app'), RUST_LOG='warn')
for name, extra in views.items():
 if a.view and name != a.view: continue
 target = out/f'{a.label}-{name}.png'
 style = 'current' if name=='current' else 'pop'
 command = [str(a.binary.resolve()),'--shot',str(target),'--review','--big','--look','polished','--style',style,*extra]
 print(f'Capturing {a.label}: {name}',flush=True)
 result = subprocess.run(command,cwd=ROOT/'rust',env=env,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,timeout=180)
 target.with_suffix('.log').write_text(result.stdout)
 if result.returncode or not target.exists() or not target.with_suffix('.metrics.txt').exists() or 'ERROR' in result.stdout:
  raise SystemExit(f'Capture failed: {target.with_suffix(".log")}')
 print(target.with_suffix('.metrics.txt').read_text().splitlines()[0],flush=True)
