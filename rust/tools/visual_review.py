#!/usr/bin/env python3
"""Capture fixed views and frame timing, one GPU process at a time. No builds or settings writes.

python3 tools/visual_review.py --label before --binary /tmp/bbq-m1-baseline
python3 tools/visual_review.py --label after
"""
import argparse
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--label', required=True)
parser.add_argument('--binary', type=Path, default=ROOT / 'rust/target/release/bbq_app')
parser.add_argument('--view', choices=['yard', 'bbq', 'shade', 'stress', 'stress-no-fx'])
args = parser.parse_args()
out = ROOT / 'checkpoints/2026-10-13-graphics-m1/evidence'
out.mkdir(parents=True, exist_ok=True)
views = {
    'yard': ['--cam', '0,5,20,0,-10,78'],
    'bbq': ['--cam', '-8,2.8,-9,0,-8,65'],
    'shade': ['--cam', '12,2.2,-6,-45,-5,75'],
    'stress': ['--cam', '0,5,20,0,-10,78', '--mode', 'heist', '--bots', '11', '--cheeky'],
    'stress-no-fx': ['--cam', '0,5,20,0,-10,78', '--mode', 'heist', '--bots', '11', '--cheeky', '--fx', 'off'],
}
env = os.environ.copy()
env['BEVY_ASSET_ROOT'] = str(ROOT / 'rust/crates/bbq_app')
env['RUST_LOG'] = 'warn'
for name, extra in views.items():
    if args.view and name != args.view:
        continue
    target = out / f'{args.label}-{name}.png'
    command = [str(args.binary.resolve()), '--shot', str(target), '--review', '--big', '--look', 'polished', '--style', 'pop', *extra]
    print(f'Capturing {args.label}: {name}', flush=True)
    result = subprocess.run(command, cwd=ROOT / 'rust', env=env, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=180)
    target.with_suffix('.log').write_text(result.stdout)
    if result.returncode or not target.exists() or not target.with_suffix('.metrics.txt').exists() or 'ERROR' in result.stdout:
        raise SystemExit(f'Capture failed; inspect {target.with_suffix(".log")}')
    print(target.with_suffix('.metrics.txt').read_text(), flush=True)
