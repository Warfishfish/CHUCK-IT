from pathlib import Path
from PIL import Image
import shutil
c=Path(__file__).resolve().parent
frames=[]
for p in sorted((c/'motion').glob('water-[0-9][0-9][0-9].png')):
 im=Image.open(p).convert('RGB')
 assert max(im.getextrema()[0])>10, f'Black frame: {p}'
 frames.append(im.resize((960,600),Image.Resampling.LANCZOS))
assert len(frames)==105,len(frames)
frames[0].save(c/'water-balloon.webp',save_all=True,append_images=frames[1:],duration=[67,67,66]*35,loop=0,quality=85)
for name,i in [('wet',30),('dry',80),('splash',11)]:shutil.copy2(c/'motion'/f'water-{i:03}.png',c/'motion'/f'{name}.png')
print('Encoded 7 seconds of actual gameplay; every source frame checked for black output.')
