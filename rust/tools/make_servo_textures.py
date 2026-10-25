"""Deterministic, tileable low-contrast surfaces; Pillow only, no external assets."""
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont
import random, math
out=Path(__file__).resolve().parents[1]/'crates/bbq_app/assets/textures'
rng=random.Random(5172)
for name,base in [('servo_dirt',213),('servo_road',125)]:
 im=Image.new('RGBA',(512,512));px=im.load()
 for y in range(512):
  for x in range(512):
   # Periodic broad weathering + restrained grain; mipmaps remove distant sparkle.
   broad=4*math.sin(x*math.tau/512)+3*math.cos(y*math.tau/512)+2*math.sin((x+y)*math.tau/128)
   v=int(max(0,min(255,base+broad+rng.uniform(-5,5))))
   px[x,y]=(v,v,v,255)
 im.save(out/(name+'.png'))
im=Image.new('RGBA',(1024,128),(57,88,79,255));d=ImageDraw.Draw(im)
font=ImageFont.truetype(str(out.parent/'fonts/BowlbyOne-Regular.ttf'),58)
d.rounded_rectangle((3,3,1020,124),radius=10,outline=(229,213,171),width=4)
d.text((512,61),'BUSH SERVO',font=font,anchor='mm',fill=(244,227,186))
im.save(out/'sign_servo.png')
