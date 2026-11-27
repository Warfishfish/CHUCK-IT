"""Bake the Bush Servo dust and worn asphalt maps with NumPy."""
from pathlib import Path
import struct
import zlib
import numpy as np

output = Path(__file__).resolve().parents[1] / 'crates/bbq_app/assets/textures'
rng = np.random.default_rng(27036)

def smooth(a, b, x):
    t = np.clip((x-a)/(b-a), 0, 1)
    return t*t*(3-2*t)

def noise(n, grid):
    y, x = np.mgrid[:n, :n] * (grid/n)
    ix, iy = x.astype(int), y.astype(int)
    dx, dy = smooth(0, 1, x-ix), smooth(0, 1, y-iy)
    a = rng.uniform(-1, 1, (grid, grid))
    u = a[iy, ix]*(1-dx) + a[iy, (ix+1)%grid]*dx
    v = a[(iy+1)%grid, ix]*(1-dx) + a[(iy+1)%grid, (ix+1)%grid]*dx
    return u*(1-dy) + v*dy

def blend(a, b, weight):
    weight = weight[..., None]
    return np.asarray(a)*(1-weight) + np.asarray(b)*weight

def output_png(name, rgb):
    rgb = np.clip(np.rint(rgb), 0, 255).astype(np.uint8)
    h, w, _ = rgb.shape
    def chunk(tag, data):
        return struct.pack('>I',len(data))+tag+data+struct.pack('>I',zlib.crc32(tag+data)&0xffffffff)
    pixels = b''.join(b'\0'+row.tobytes() for row in rgb)
    (output/name).write_bytes(b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR', struct.pack('>IIBBBBB',w,h,8,2,0,0,0))+chunk(b'IDAT',zlib.compress(pixels,9))+chunk(b'IEND',b''))

n = 512
broad, mids, grain = [noise(n, scale) for scale in (4, 11, 68)]
chalk = smooth(-.55, .74, broad + .27*mids)
soil = blend([151, 103, 72], [178, 137, 99], chalk)
soil += (mids*5 + grain*3)[..., None]*[1, .88, .75]
output_png('servo_dirt.png', soil)

n = 1024
z, x=np.mgrid[0:n,0:n]/(n-1)*58-29
rad, angle=np.hypot(x,z), np.arctan2(z,x)
movement=.18*np.sin(angle*17)+.11*np.sin(angle*39+1.2)
inner,outer=16.5+movement,19+movement*1.4
depth=np.minimum(rad-inner, outer-rad)
patch, broken, detail = [noise(n, k) for k in (19, 75, 220)]
tracks=np.exp(-((rad-17.16)/.26)**2)+np.exp(-((rad-18.29)/.23)**2)
repairs=smooth(.35,.58, patch+.13*broken)*smooth(.04,.37,depth)
asphalt=np.asarray([64,63,62])+(patch*9+broken*5+detail*4-tracks*7-repairs*8)[...,None]
edge_width=.20+.13*np.sin(7*angle+.5)+.13*patch+.10*broken
washed_edge=1-smooth(-.06,.3,depth-edge_width)
silt=smooth(.35,.87, patch + .47*broken) * .34
dust=np.maximum(washed_edge*.93, silt)
earth=blend([151,103,72],[178,137,99],smooth(-.66,.78,patch+.47*broken))
output_png('servo_road.png',blend(asphalt,earth,dust))
