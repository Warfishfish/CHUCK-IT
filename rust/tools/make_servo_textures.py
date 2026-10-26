"""Deterministic, tileable low-contrast surfaces; Pillow only, no external assets."""
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont
import random, math
out=Path(__file__).resolve().parents[1]/'crates/bbq_app/assets/textures'
import runpy
runpy.run_path(str(Path(__file__).with_name('make_servo_ground.py')))
im=Image.new('RGBA',(1024,128),(57,88,79,255));d=ImageDraw.Draw(im)
font=ImageFont.truetype(str(out.parent/'fonts/BowlbyOne-Regular.ttf'),58)
d.rounded_rectangle((3,3,1020,124),radius=10,outline=(229,213,171),width=4)
d.text((512,61),'BUSH SERVO',font=font,anchor='mm',fill=(244,227,186))
im.save(out/'sign_servo.png')
