"""Deterministic Bush Servo ground maps (Pillow + NumPy, no downloaded assets).
1024px road spans 58m before Z squash; 512px dirt repeats every 8.125m.
Roughness stays in the existing material. Opaque geometry preserves ground shadows.
"""
from pathlib import Path
from PIL import Image, ImageDraw
import numpy as np
import random, math
OUT=Path(__file__).resolve().parents[1]/'crates/bbq_app/assets/textures'
SEED=260126
# Warm dust encroaches unevenly; preserve dark readable racing/combat surface.
SHOULDER_WIDTH=1.65
PATCH_STRENGTH=.24
AGGREGATE_STRENGTH=2.4
DIRT=(189,113,65)
ROAD=(89,85,77)
rng=np.random.default_rng(SEED)
def noise(n,grid):
 a=rng.integers(0,256,(grid,grid),dtype=np.uint8)
 tiled=np.tile(a,(3,3))
 im=Image.fromarray(tiled).resize((n*3,n*3),Image.Resampling.BICUBIC)
 return (np.asarray(im)[n:2*n,n:2*n].astype(float)-127.5)/127.5
n=512
large=noise(n,6);medium=noise(n,28);fine=rng.normal(0,1,(n,n))
variation=10*large+6*medium+1.7*fine
rgb=np.clip(np.array(DIRT)[None,None,:]+variation[:,:,None]*np.array([1,.8,.55]),0,255).astype('uint8')
im=Image.fromarray(rgb).convert('RGBA');d=ImageDraw.Draw(im)
r=random.Random(SEED)
# Tiny embedded grit, subdued enough to mip away rather than sparkle.
for _ in range(1200):
 x,y=r.randrange(n),r.randrange(n);v=r.choice([-16,-10,9,14]);c=tuple(max(0,min(255,a+v)) for a in DIRT)
 d.ellipse((x,y,x+r.choice([1,1,2]),y+1),fill=(*c,255))
im.save(OUT/'servo_dirt.png')
n=1024
coarse=noise(n,9);mid=noise(n,38);grain=rng.normal(0,1,(n,n))
y,x=np.mgrid[0:n,0:n];xx=(x/(n-1)-.5)*58;zz=(y/(n-1)-.5)*58
radius=np.hypot(xx,zz);a=np.arctan2(zz,xx)
jag=.18*np.sin(a*17)+.11*np.sin(a*39+1.2)+.08*mid
inner=18+jag;outer=26+jag*1.4
inside=np.minimum(radius-inner,outer-radius)
shoulder=np.clip(1-inside/(SHOULDER_WIDTH+.45*coarse),0,1)
patch=np.clip((coarse+.16*mid-.08)*2.4,0,1)
dust=np.clip(.07+PATCH_STRENGTH*patch+shoulder*(.85+.15*mid),0,1)
ruts=np.exp(-((radius-21.0)/.65)**2)+np.exp(-((radius-23.0)/.65)**2)
base=np.array(ROAD)[None,None,:]+(7*coarse+4*mid+AGGREGATE_STRENGTH*grain-3*ruts)[:,:,None]
colour=np.clip(base*(1-dust[:,:,None])+np.array(DIRT)[None,None,:]*dust[:,:,None],0,255).astype('uint8')
im=Image.fromarray(colour);d=ImageDraw.Draw(im)
# Sparse hairline branching cracks, not a dense noise overlay.
for _ in range(40):
 angle=r.uniform(0,math.tau);rad=r.uniform(19,25)
 px=(math.cos(angle)*rad/58+.5)*(n-1);py=(math.sin(angle)*rad/58+.5)*(n-1)
 points=[(px,py)]
 for j in range(r.randint(3,7)):
  px+=r.uniform(-7,7);py+=r.uniform(3,9);points.append((px,py))
 d.line(points,fill=(66,63,57),width=1)
 if r.random()<.5:
  px,py=points[len(points)//2];d.line([(px,py),(px+7,py+4),(px+11,py+12)],fill=(72,67,59),width=1)
def picnic_shoulder(image):
    """Bake the inner shoulder beside the east picnic table into the existing road UVs.

    Local mesh coordinates are x and (6 - world_z) / 0.65. Fade the new
    paint along the bend and across the lane into the original road.
    """
    def smooth(lo, hi, value):
        t = np.clip((value - lo) / (hi - lo), 0, 1)
        return t*t*(3-2*t)

    angle = np.degrees(a)
    sector = smooth(24, 39, angle) * (1 - smooth(77, 91, angle))
    depth = radius-inner
    coverage = sector * (1-smooth(3.8, 5.4, depth)) * smooth(-.4, .06, depth)
    # Distort the dust front at three scales. Broken islands interrupt its edge.
    broad, broken, grit = (noise(n, k) for k in (17, 66, 215))
    front = 1.15 + .80*np.sin(9*a+.8) + .72*broad + .48*broken
    deposit = front - depth + .62*broken + .37*grit
    dust_cover = smooth(-.38, .24, deposit)
    islands = smooth(.18, .73, broken+.52*grit) * (1-smooth(1.4, 3.6, depth))
    dust_cover = np.maximum(dust_cover, .86*islands)
    # Sparse embedded chips and muted warm aggregate. Grain fades into the mips.
    aggregate = 5.3*grain + 7*mid + 6*broad + 9*smooth(.60, .88, grit)
    tar = np.array([69, 66, 63]) + aggregate[:,:,None]
    earth = np.array(DIRT) + (11*broad+8*broken+4*grain)[:,:,None]*[1,.82,.62]
    paint = tar + (earth-tar)*dust_cover[:,:,None]
    original = np.asarray(image.convert('RGB')).astype(float)
    result = original + (paint-original)*coverage[:,:,None]
    return Image.fromarray(np.clip(result.round(),0,255).astype('uint8'))

im=picnic_shoulder(im).convert('RGBA');im.save(OUT/'servo_road.png')
print('Saved 512px dirt and 1024px opaque road with dusty shoulders.')
