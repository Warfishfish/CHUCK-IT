#!/usr/bin/env python3
"""Encode captured first-person motion. Requires Pillow; no generated images."""
import argparse
from pathlib import Path
from PIL import Image, ImageDraw
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--label',required=True);p.add_argument('--output-dir',type=Path);a=p.parse_args()
out=a.output_dir or Path(__file__).resolve().parents[2]/'checkpoints/2026-10-14-held-motion/evidence'
for name in ['held-noodle','held-rubber']:
    frames=[]
    for i in range(75):
        with Image.open(out/f'{a.label}-{name}-{i:03}.png') as source:
            frame=source.convert('RGB');frame.thumbnail((960,600));frames.append(frame)
    frames[0].save(out/f'{a.label}-{name}.webp',save_all=True,append_images=frames[1:],duration=[67,67,66]*25,loop=0,quality=85,method=4)
    sheet=Image.new('RGB',(1440,960),'#14232c');draw=ImageDraw.Draw(sheet)
    for j,i in enumerate([0,10,25,32,40,46,55,65,74]):
        x=j%3*480;y=j//3*320
        sheet.paste(frames[i].resize((480,300)),(x,y));draw.text((x+10,y+302),f'{i/15:.2f}s',fill='white')
    sheet.save(out/f'{a.label}-{name}-contact.jpg')
    print('Encoded',a.label,name)
