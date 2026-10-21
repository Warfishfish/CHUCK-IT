from pathlib import Path
from PIL import Image
import re
c=Path(__file__).resolve().parent
for name,pattern,duration in [('balloon','balloon-[0-9][0-9][0-9].png',67),('magpie','magpie-[0-9][0-9][0-9].png',67),('throw','throw_[0-9]*.png',33)]:
 files=sorted(c.glob(pattern),key=lambda p:int(re.search(r"(\d+)$",p.stem).group(1)));assert files,name
 frames=[]
 for f in files:
  with Image.open(f) as im:frames.append(im.convert('RGB').resize((960,600),Image.Resampling.LANCZOS))
 frames[0].save(c/(name+'.webp'),save_all=True,append_images=frames[1:],duration=duration,loop=0,quality=83,method=4)
 print(name,len(frames))
