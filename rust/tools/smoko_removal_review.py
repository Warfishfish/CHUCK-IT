#!/usr/bin/env python3
"""Capture the former Smoko areas and matched performance, one game process at a time."""
import argparse
import os
from pathlib import Path
import subprocess

root = Path(__file__).resolve().parents[2]
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--label', required=True)
p.add_argument('--binary', type=Path, default=root/'rust/target/release/bbq_app')
a = p.parse_args()
out = root/'checkpoints/2026-10-15-smoko-removal/evidence'
out.mkdir(parents=True, exist_ok=True)
views = {
    'standard': ['--cam', '-16,2.2,-3,90,-8,85'],
    'heist': ['--cam', '0,3.5,12,70,-12,80', '--mode', 'heist'],
    'stress': ['--cam', '0,5,20,0,-10,78', '--mode', 'heist', '--bots', '11'],
}
env = dict(os.environ, BEVY_ASSET_ROOT=str(root/'rust/crates/bbq_app'), RUST_LOG='warn')
for name, extra in views.items():
    target = out/f'{a.label}-{name}.png'
    r = subprocess.run([str(a.binary.resolve()), '--shot', str(target), '--review', '--big',
                        '--look', 'polished', '--style', 'pop', '--cheeky', *extra],
                       cwd=root/'rust', env=env, text=True, stdout=subprocess.PIPE,
                       stderr=subprocess.STDOUT, timeout=240)
    target.with_suffix('.log').write_text(r.stdout)
    if r.returncode or 'ERROR' in r.stdout or not target.exists() or not target.with_suffix('.metrics.txt').exists():
        raise SystemExit(f'Capture failed: {target}')
    print(name, target.with_suffix('.metrics.txt').read_text(), flush=True)
