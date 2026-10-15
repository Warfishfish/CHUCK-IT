#!/usr/bin/env python3
"""Encode actual review PNG frames as looping WebP clips (requires Pillow)."""
import argparse
from pathlib import Path
from PIL import Image
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--label',required=True)
a=p.parse_args()
root=Path(__file__).resolve().parents[2]
out=root/'checkpoints/2026-10-14-item-physics/evidence'
for scene in ['drop','throw']:
    frames=[]
    for i in range(75):
        with Image.open(out/f'{a.label}-{scene}-{i:03}.png') as source:
            frame=source.convert('RGB')
            frame.thumbnail((960,600))
            frames.append(frame)
    frames[0].save(out/f'{a.label}-{scene}.webp',save_all=True,append_images=frames[1:],duration=[67,67,66]*25,loop=0,quality=85,method=4)
    print('Encoded',a.label,scene)
