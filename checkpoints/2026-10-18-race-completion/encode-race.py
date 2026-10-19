from pathlib import Path
from PIL import Image
c=Path(__file__).resolve().parent
frames=[]
for i in range(75):
 p=c/'after'/f'race-motion-{i:03}.png'
 im=Image.open(p).convert('RGB');assert max(v[1] for v in im.getextrema())>25,p
 im.thumbnail((960,600));frames.append(im)
frames[0].save(c/'race.webp',save_all=True,append_images=frames[1:],duration=[67,67,66]*25,loop=0,quality=85)
print('Encoded five seconds of actual race gameplay.')
