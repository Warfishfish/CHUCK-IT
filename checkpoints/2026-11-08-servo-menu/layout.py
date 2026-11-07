"""Measured site diagrams in shared game coordinates, drawn with Pillow."""
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

im=Image.new('RGB',(1400,670),'#f1e6cf')
d=ImageDraw.Draw(im)
font_path='/System/Library/Fonts/Supplemental/Arial.ttf'
font=ImageFont.truetype(font_path,18)
title=ImageFont.truetype(font_path,25)
for col,(name,offset) in enumerate([('A - north-west',-6),('B - north-east',17)]):
 left=col*700+35
 def point(x,z): return (left+(x+35)*9,70+(z+26)*9)
 def rect(x,z,w,depth,colour,outline=None):
  d.rectangle([point(x,z),point(x+w,z+depth)],fill=colour,outline=outline,width=2)
 d.text((left,16),name,font=title,fill='#302b20')
 rect(-33,-24,66,48,'#deb080','#65462c')
 d.ellipse([point(-26,6-16.9),point(26,6+16.9)],outline='#746855',width=35)
 d.ellipse([point(-18,6-11.7),point(18,6+11.7)],fill='#deb080')
 rect(-23.75+offset,-23.5,35.415,9,None,'#b32920')
 rect(-19.5+offset,-23,23,7,'#a04f36')
 for x in [7.6,10.8]:rect(x+offset-.8,-17.5,1.6,1.7,'#d52727')
 rect(-4.65,5.4,9.3,9.2,'#3d8b7a')
 d.text(point(-3.6,8),'SMOKO',font=font,fill='white')
 for x in [-24,12]:
  px,pz=point(x,-19.5);d.ellipse((px-6,pz-6,px+6,pz+6),fill='white',outline='black')
 rect(-9.45,-18.6,1.5,.8,'#30261e')
 rect(-6.85,-18.4,1.7,.8,'#30261e')
 rect(-1.8,-21.95,3.6,.9,'#405d86')
 for x in [-14,14]:
  d.line([point(x,1),point(x*.32,9)],fill='#287d3c',width=4)
 d.text((left,555),'White dots - existing spawns',font=font,fill='#302b20')
 d.text((left,581),'Brown/blue blocks - BBQ, meat table and bar',font=font,fill='#302b20')
 d.text((left,607),'Green lines - carrying approaches to keep clear',font=font,fill='#302b20')
d.text((35,642),'Both candidates need spawn or supply adjustments before permanent integration.',font=font,fill='#923026')
im.save(Path(__file__).parent/'evidence/sites.png')
