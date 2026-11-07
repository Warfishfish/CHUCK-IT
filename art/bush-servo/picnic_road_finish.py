"""Bake a local high-resolution shoulder skin onto the shipped surroundings GLB.

Usage: python picnic_road_finish.py input.glb output.glb  (NumPy, Pillow)
Coordinates follow the road mesh. Two strips meet at the irregular asphalt lip,
one at bitumen height and one on the lower dirt plane.
"""
import io
import json
import math
import struct
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw

NAME = "East picnic weathered bitumen"
WIDTH, HEIGHT = 1024, 512
ANGLE_START, ANGLE_END = 24, 91
MIN_OFFSET, MAX_OFFSET = -2.1, 6.8
STEPS = 72


def smooth(lo, hi, value):
    t = np.clip((value-lo)/(hi-lo), 0, 1)
    return t*t*(3-2*t)


def surface():
    rng = np.random.default_rng(71026)
    def field(cols, rows):
        grid = rng.integers(0, 256, (rows, cols), dtype=np.uint8)
        image = Image.fromarray(grid).resize((WIDTH, HEIGHT), Image.Resampling.BICUBIC)
        return np.asarray(image).astype(float)/127.5-1

    v, u = np.mgrid[0:HEIGHT, 0:WIDTH]
    u, v = u/(WIDTH-1), v/(HEIGHT-1)
    d = MIN_OFFSET+(MAX_OFFSET-MIN_OFFSET)*v
    patch, flake, grit = field(19, 10), field(96, 55), field(337, 173)
    grain = rng.normal(0, 1, d.shape)
    # Fine chips, larger red-earth fans, eroded tongues following the road edge.
    lip = .36+.36*np.sin(11*u)+.27*np.cos(23*u)
    for pos, width, reach in [(.19,.060,.8),(.46,.089,1.4),(.71,.045,1.1),(.85,.038,.47)]:
        lip += reach*np.exp(-((u-pos-.015*d)/width)**2)
    lip += .37*patch+.21*flake+.12*grit
    worn = d-lip
    cover = 1-smooth(-.08,.12,worn)
    scatter = smooth(.25,.70,.50*flake+.58*grit+.21*grain)
    cover += (1-cover)*scatter*(1-smooth(.02,1.15,worn))
    # Warm, desaturated aggregate and faded seal; sparse bitumen islands in the dust.
    tar_noise = 8*patch+4*flake+3*grit+2.9*grain
    tar = np.array([71,64,65])+tar_noise[:,:,None]
    sand_noise = 8*patch+5*flake+4*grit+2*grain
    sand = np.array([187,110,67])+sand_noise[:,:,None]*[1,.75,.52]
    loose = .19*np.exp(-((worn-.62)/.84)**2)*(.85+.15*patch)
    cover = cover+(1-cover)*loose
    rgb = tar+(sand-tar)*cover[:,:,None]
    # Flecks of light stone concentrate along the lip, not in the driving lane.
    image = Image.fromarray(np.uint8(np.clip(rgb,0,255)))
    pen = ImageDraw.Draw(image)
    for _ in range(5300):
        x = int(rng.integers(WIDTH))
        depth = float(lip[HEIGHT//3,x]+rng.normal(0,.64))
        y = int((depth-MIN_OFFSET)/(MAX_OFFSET-MIN_OFFSET)*HEIGHT)
        if not 2 <= y < HEIGHT-2:
            continue
        size = int(rng.choice([1,1,2,2,3]))
        shade = tuple(int(i) for i in rng.choice([[141,92,62],[161,117,83],[109,80,64],[192,138,91]]))
        pen.line((x,y,x+size,y-1), fill=shade)
    # Game road PNGs are interpreted as linear values. glTF base colour is sRGB.
    # Transfer the painted palette once so the shoulder shares the yard lighting.
    linear = np.asarray(image).astype(float)/255
    encoded = np.where(linear <= .0031308, 12.92*linear, 1.055*linear**(1/2.4)-.055)
    image = Image.fromarray(np.uint8(np.round(255*encoded)))
    alpha = (smooth(0,.12,u)*(1-smooth(.85,1,u))*
             smooth(MIN_OFFSET,-.65,d)*(1-smooth(4.3,MAX_OFFSET,d)))
    image.putalpha(Image.fromarray(np.uint8(np.round(255*alpha))))
    out = io.BytesIO()
    image.save(out,format="PNG",optimize=True)
    return out.getvalue()


def export(source, destination):
    raw = source.read_bytes()
    size = struct.unpack_from('<I',raw,12)[0]
    doc = json.loads(raw[20:20+size])
    if any(m.get('name')==NAME for m in doc['meshes']):
        raise ValueError("Use the pre-finish surroundings GLB as input")
    blob = bytearray(raw[28+size:])
    def view(data, target=None):
        blob.extend(bytes(-len(blob)%4))
        v = dict(buffer=0,byteOffset=len(blob),byteLength=len(data))
        if target:
            v['target']=target
        blob.extend(data)
        doc['bufferViews'].append(v)
        return len(doc['bufferViews'])-1
    def acc(values, dtype, kind, component, target, **extra):
        arr=np.array(values,dtype=dtype)
        doc['accessors'].append(dict(bufferView=view(arr.tobytes(),target),type=kind,
                                     componentType=component,count=len(arr),**extra))
        return len(doc['accessors'])-1
    image = len(doc.setdefault('images',[]))
    doc['images'].append(dict(name=NAME,bufferView=view(surface()),mimeType='image/png'))
    texture = len(doc.setdefault('textures',[]))
    doc['textures'].append(dict(source=image))
    material = len(doc['materials'])
    doc['materials'].append(dict(name=NAME,alphaMode='BLEND',pbrMetallicRoughness=dict(
        baseColorTexture=dict(index=texture),metallicFactor=0,roughnessFactor=1)))
    pos, uv, idx = [], [], []
    for lo,hi,height in [(MIN_OFFSET,.015,-.009),(-.02,MAX_OFFSET,.012)]:
        base=len(pos)
        for i in range(STEPS+1):
            u=i/STEPS
            a=math.radians(ANGLE_START+(ANGLE_END-ANGLE_START)*u)
            inner=18+.18*math.sin(17*a)+.11*math.sin(39*a+1.2)
            for d in (lo,hi):
                r=inner+d
                pos.append((r*math.cos(a),height,6-.65*r*math.sin(a)))
                uv.append((u,(d-MIN_OFFSET)/(MAX_OFFSET-MIN_OFFSET)))
            if i:
                j=base+2*i-2
                idx.extend((j,j+1,j+2,j+1,j+3,j+2))
    attrs=dict(POSITION=acc(pos,'<f4','VEC3',5126,34962,min=list(map(min,zip(*pos))),max=list(map(max,zip(*pos)))),
               NORMAL=acc([(0,1,0)]*len(pos),'<f4','VEC3',5126,34962),
               TEXCOORD_0=acc(uv,'<f4','VEC2',5126,34962))
    node=len(doc['nodes'])
    doc['nodes'].append(dict(name=NAME,mesh=len(doc['meshes'])))
    doc['scenes'][doc.get('scene',0)]['nodes'].append(node)
    doc['meshes'].append(dict(name=NAME,primitives=[dict(attributes=attrs,indices=acc(idx,'<u2','SCALAR',5123,34963),material=material)]))
    blob.extend(bytes(-len(blob)%4))
    doc['buffers'][0]['byteLength']=len(blob)
    header=json.dumps(doc,separators=(',',':')).encode()
    header+=b' '*(-len(header)%4)
    destination.write_bytes(struct.pack('<4sIII4s',b'glTF',2,28+len(header)+len(blob),len(header),b'JSON')+header+struct.pack('<I4s',len(blob),b'BIN\0')+blob)


if __name__=='__main__':
    export(Path(sys.argv[1]),Path(sys.argv[2]))
