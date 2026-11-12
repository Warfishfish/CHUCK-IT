"""Finish the shipped Smoko assets from 615b759. Pass separate input and output asset directories."""
import io
import json
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageFilter

from finish_approach import read_glb, write_glb

ART = Path(__file__).parent
LAYOUT = json.loads((ART / 'picnic-island.json').read_text())


def attribute(doc, blob, index):
    acc = doc['accessors'][index]
    view = doc['bufferViews'][acc['bufferView']]
    assert 'byteStride' not in view
    dtype = {5126: '<f4', 5123: '<u2', 5125: '<u4'}[acc['componentType']]
    width = {'VEC3': 3, 'SCALAR': 1}[acc['type']]
    return np.frombuffer(blob, dtype, count=width*acc['count'],
                         offset=view.get('byteOffset', 0)+acc.get('byteOffset', 0)).reshape(-1, width)


def bounds(doc, index, array):
    doc['accessors'][index].update(min=array.min(axis=0).tolist(), max=array.max(axis=0).tolist())


def inside(points, box):
    x, z, w, d, h = box
    return ((abs(points[:, 0]-x) <= w/2+.002) & (abs(points[:, 2]-z) <= d/2+.002)
            & (points[:, 1] >= -.002) & (points[:, 1] <= h+.002))


def whole_triangles(doc, blob, prim, mask):
    tri = mask[attribute(doc, blob, prim['indices']).reshape(-1, 3)]
    if np.any(tri.any(axis=1) != tri.all(axis=1)):
        raise ValueError('Selection cuts through a triangle')


def props(doc, blob):
    node = doc['nodes'][0]
    for old, dst in [((11, -1.5, 1.3, .75, .72), LAYOUT['esky']),
                     ((7.2, 14.8, .72, .72, 1.1), LAYOUT['bin'])]:
        moved = []
        for prim in doc['meshes'][node['mesh']]['primitives']:
            idx = prim['attributes']['POSITION']
            pos = attribute(doc, blob, idx)
            world = pos + node['translation']
            mask = inside(world, old)
            whole_triangles(doc, blob, prim, mask)
            if mask.any():
                moved.extend(world[mask])
                pos[mask] += [dst[0]-old[0], 0, dst[1]-old[1]]
                bounds(doc, idx, pos)
        assert np.allclose(np.ptp(moved, axis=0), [old[2], old[4], old[3]], atol=.006)


def stone_banks(doc, blob):
    old = json.loads((ART/'surroundings-layout.json').read_text())['rocks']
    node = doc['nodes'][0]
    for i, (src, dst) in enumerate(zip(old[10:20], LAYOUT['front_bank']), 10):
        prim = doc['meshes'][0]['primitives'][i % 4]
        idx = prim['attributes']['POSITION']
        pos = attribute(doc, blob, idx)
        world = pos + node['translation']
        # Each bevelled block has 216 exported vertices. Gravel with the same
        # material follows the five large blocks in this primitive.
        mask = np.zeros(len(world), dtype=bool)
        first = (i//4)*216
        mask[first:first+216] = True
        assert inside(world[mask], src).all()
        whole_triangles(doc, blob, prim, mask)
        x, z, w, d, h = dst
        scale = np.array([w/src[2], h/src[4], d/src[3]])
        assert np.allclose(np.ptp(world[mask], axis=0), [src[2], src[4], src[3]], atol=.006)
        pos[mask] = (world[mask]-[src[0],0,src[1]])*scale + [x,0,z] - node['translation']
        bounds(doc, idx, pos)
        normals = attribute(doc, blob, prim['attributes']['NORMAL'])
        n = normals[mask]/scale
        normals[mask] = n/np.linalg.norm(n, axis=1, keepdims=True)


def contact(doc, blob):
    mesh = next(m for m in doc['meshes'] if m['name'] == 'East gum and picnic ground wear')
    idx = mesh['primitives'][0]['attributes']['POSITION']
    pos = attribute(doc, blob, idx)
    # The fourth stamp from ground_prop_bases.py belongs to the blue cooler.
    stamp = pos[3*129:4*129]
    assert np.allclose(stamp[0, [0,2]], [11,-1.5])
    stamp += [LAYOUT['esky'][0]-11, 0, LAYOUT['esky'][1]+1.5]
    bounds(doc, idx, pos)


def surface(doc, blob):
    image = next(i for i in doc['images'] if i['name'] == 'East picnic weathered bitumen')
    view = doc['bufferViews'][image['bufferView']]
    start, length = view['byteOffset'], view['byteLength']
    pixels = np.array(Image.open(io.BytesIO(blob[start:start+length])).convert('RGBA'))
    y, x = np.mgrid[:pixels.shape[0], :pixels.shape[1]]
    u, v = x/(pixels.shape[1]-1), y/(pixels.shape[0]-1)
    depth = -2.1 + 8.9*v
    # Two broad wheel-polished paths wander through the dusty turn. Red fines
    # collect between them; the original texture supplies aggregate and edge breakup.
    bend = .44*np.sin(4.6*u+1.1)+.19*np.sin(12*u)
    wheel = sum(np.exp(-((depth-bend-centre)/width)**2) for centre,width in [(1.5,.31),(3.05,.36)])
    dust = np.exp(-((depth-bend-2.26)/.66)**2)*(.65+.35*np.cos(8*u))
    rgb = pixels[:,:,:3].astype(float)
    rgb += dust[:,:,None]*[17,8,2] - wheel[:,:,None]*[18,13,10]
    pixels[:,:,:3] = np.uint8(np.clip(rgb,0,255))
    stream = io.BytesIO()
    Image.fromarray(pixels).save(stream, format='PNG', optimize=True)
    packed = stream.getvalue()
    old_end = (start+length+3)//4*4
    replacement = packed + bytes(-len(packed)%4)
    blob[start:old_end] = replacement
    for other in doc['bufferViews']:
        if other.get('byteOffset',0) >= old_end:
            other['byteOffset'] += len(replacement) - (old_end-start)
    view['byteLength'] = len(packed)


def shade(src, dst):
    image = Image.open(src).convert('RGBA')
    alpha = image.getchannel('A')
    broad = np.asarray(alpha.filter(ImageFilter.GaussianBlur(3.0))).astype(float)
    original = np.asarray(alpha).astype(float)
    # Retain sun holes and deepen dense foliage; leave the edge of the roof soft.
    tuned = .98*original + .34*broad + .56*(original-broad)
    image.putalpha(Image.fromarray(np.uint8(np.clip(tuned,0,155))))
    image.save(dst, optimize=True)


def bake(src, dst):
    if src.resolve() == dst.resolve():
        raise ValueError('Export into a separate assets directory')
    for name in ['models','textures']:
        (dst/name).mkdir(exist_ok=True, parents=True)
    for name in ['smoko_props.glb','smoko_surroundings.glb']:
        doc, blob = read_glb(src/'models'/name)
        if name == 'smoko_props.glb':
            props(doc, blob)
        else:
            stone_banks(doc, blob)
            contact(doc, blob)
            surface(doc, blob)
        write_glb(dst/'models'/name, doc, blob)
    texture = Path('textures/smoko_picnic_shade.png')
    shade(src/texture, dst/texture)


if __name__ == '__main__':
    bake(*(Path(p) for p in sys.argv[1:]))
