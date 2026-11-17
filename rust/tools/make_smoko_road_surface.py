"""Bake the current 16.5–19 m circular road in Blender MCP's Python runtime.
Original procedural art, no external assets; exact 58m world-space UVs used by shapes.rs.
"""
import math, struct, zlib
from pathlib import Path
import numpy as np
ROOT=Path(__file__).resolve().parents[1]
n=1024
y,x=np.mgrid[0:n,0:n];xx=(x/(n-1)-.5)*58;zz=(y/(n-1)-.5)*58
r=np.hypot(xx,zz);a=np.arctan2(zz,xx)
edge=.18*np.sin(a*17)+.11*np.sin(a*39+1.2)
inside=np.minimum(r-(16.5+edge),(19+edge*1.4)-r)
# Broad waves, no per-pixel white speckling. Two worn wheel bands share the actual lane.
broad=np.sin(xx*.73+np.sin(zz*.28))*.45+np.sin(zz*.58+xx*.19)*.30
shoulder=np.clip(1-inside/(.40+.10*broad),0,1)
wheel=np.exp(-((r-17.20)/.21)**2)+np.exp(-((r-18.25)/.23)**2)
dust=np.clip(.10+shoulder*.84+broad*.055,0,1)
asphalt=np.array([89,85,77])[None,None,:]+(broad*7-wheel*5)[:,:,None]
earth=np.array([189,113,65])[None,None,:]+broad[:,:,None]*[9,7,4]
rgb=np.clip(asphalt*(1-dust[:,:,None])+earth*dust[:,:,None],0,255).astype(np.uint8)
def chunk(tag,data):return struct.pack('>I',len(data))+tag+data+struct.pack('>I',zlib.crc32(tag+data)&0xffffffff)
raw=b''.join(b'\0'+rgb[j].tobytes() for j in range(n))
png=b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',n,n,8,2,0,0,0))+chunk(b'IDAT',zlib.compress(raw,9))+chunk(b'IEND',b'')
path=ROOT/'crates/bbq_app/assets/textures/servo_road.png';path.write_bytes(png)
print('BAKED current circular road',path,len(png),'bytes')
