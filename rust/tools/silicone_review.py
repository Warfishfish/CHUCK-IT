#!/usr/bin/env python3
"""Real game captures for continuous silicone models: held motion, gallery, drop and performance."""
import argparse,subprocess,os
from pathlib import Path
root=Path(__file__).resolve().parents[2]
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--label',required=True);p.add_argument('--binary',type=Path,default=root/'rust/target/release/bbq_app');p.add_argument('--baseline',action='store_true');p.add_argument('--encode',action='store_true');a=p.parse_args()
out=root/'checkpoints/2026-10-14-soft-silicone/evidence'
env=os.environ.copy();env.update(BEVY_ASSET_ROOT=str(root/'rust/crates/bbq_app'),RUST_LOG='warn')
if a.encode:
    from PIL import Image, ImageDraw
    for scene in ['held-rubber','drop']:
        frames=[]
        for i in range(75):
            with Image.open(out/f'{a.label}-{scene}-{i:03}.png') as source:
                frame=source.convert('RGB');frame.thumbnail((960,600));frames.append(frame)
        frames[0].save(out/f'{a.label}-{scene}.webp',save_all=True,append_images=frames[1:],duration=[67,67,66]*25,loop=0,quality=85,method=4)
        sheet=Image.new('RGB',(1440,960),'#14232c');draw=ImageDraw.Draw(sheet)
        for j,i in enumerate([0,10,25,32,40,46,55,65,74]):
            x=j%3*480;y=j//3*320;sheet.paste(frames[i].resize((480,300)),(x,y));draw.text((x+10,y+302),f'{i/15:.2f}s',fill='white')
        sheet.save(out/f'{a.label}-{scene}-contact.jpg')
        print('Encoded',scene)
    raise SystemExit(0)
scenes=['gallery','stress'] if a.baseline else ['held-rubber','gallery','drop','stress']
for scene in scenes:
    target=out/f'{a.label}-{scene}.png'
    extra={'gallery':['--gallery','--cam','0,30.35,5,0,0,48'], 'stress':['--cam','0,5,20,0,-10,78','--mode','heist','--bots','11'], 'held-rubber':['--motion-review','held-rubber'], 'drop':['--motion-review','drop','--cam','0,3.8,17,0,-10,60']}[scene]
    r=subprocess.run([str(a.binary.resolve()),'--shot',str(target),'--review','--big','--look','polished','--style','pop','--cheeky',*extra],cwd=root/'rust',env=env,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,timeout=240)
    target.with_suffix('.log').write_text(r.stdout)
    expected=[out/f'{a.label}-{scene}-{i:03}.png' for i in range(75)] if scene in ['held-rubber','drop'] else [target,target.with_suffix('.metrics.txt')]
    if r.returncode or 'ERROR' in r.stdout or not all(p.exists() for p in expected):raise SystemExit(f'Capture failed: {target.with_suffix(".log")}')
    print('Saved',a.label,scene,flush=True)
