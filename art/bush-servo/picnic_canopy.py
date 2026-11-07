"""Filter the east gum's sun projection into a small static shade atlas.

Arguments: original landscape GLB, output landscape GLB, output shade PNG.
Keep smoko_shelter.glb beside the output GLB. Nearby leaves still cast real shadows
on players, tables and the esky. The atlas carries the broader canopy shade.
"""
import json
import struct
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw, ImageFilter

TREE_XZ = np.array([12.0, 1.5])
SUN_XZ_PER_METRE = np.array([1.0, 16/24])  # servo_finish::SUN_POSITION / sun.y
X0, Z0, EXTENT, SIZE = -12, -12, 28, 512
LEAF_SCALE = .57


def bake(source, destination, shade_path):
    raw = source.read_bytes()
    size = struct.unpack_from('<I', raw, 12)[0]
    doc = json.loads(raw[20:20+size])
    if doc.get('extras', {}).get('picnicCanopyBaked'):
        raise ValueError('Use the pre-shade landscape export')
    blob = bytearray(raw[28+size:])
    canopy = Image.new('L', (SIZE, SIZE))
    pen = ImageDraw.Draw(canopy)
    count = 0
    for prim in doc['meshes'][0]['primitives']:
        if not doc['materials'][prim['material']]['name'].startswith('Gum foliage '):
            continue
        pa = doc['accessors'][prim['attributes']['POSITION']]
        view = doc['bufferViews'][pa['bufferView']]
        assert pa['type'] == 'VEC3' and pa['componentType'] == 5126 and 'byteStride' not in view
        # The Blender exporter writes separate, contiguous quads for these sprays.
        pos = np.frombuffer(blob, '<f4', count=pa['count']*3,
                            offset=view.get('byteOffset',0)+pa.get('byteOffset',0)).reshape(-1,4,3).copy()
        ia = doc['accessors'][prim['indices']]
        iv = doc['bufferViews'][ia['bufferView']]
        ix = np.frombuffer(blob, '<u2' if ia['componentType']==5123 else '<u4',
                           count=ia['count'], offset=iv.get('byteOffset',0)+ia.get('byteOffset',0))
        assert np.array_equal(ix.reshape(-1,6) - 4*np.arange(len(pos))[:,None], [[0,1,2,0,2,3]]*len(pos))
        del ix
        centre = pos.mean(axis=1)
        local = (np.linalg.norm(centre[:,[0,2]]-TREE_XZ,axis=1)<6) & (centre[:,1]>4)
        for points in pos[local]:
            projected = points[:,[0,2]]-points[:,1,None]*SUN_XZ_PER_METRE
            texels = (projected-[X0,Z0])*SIZE/EXTENT
            pen.polygon([tuple(p) for p in texels],fill=255)
        pos[local] = centre[local,None,:]+LEAF_SCALE*(pos[local]-centre[local,None,:])
        off=view.get('byteOffset',0)+pa.get('byteOffset',0)
        blob[off:off+pos.nbytes]=pos.tobytes()
        count += int(local.sum())
    assert count == 2100, f'Unexpected east canopy: {count} leaves'
    # Two penumbra widths preserve pools of sun between sprays at standing height.
    near = np.asarray(canopy.filter(ImageFilter.GaussianBlur(.19*SIZE/EXTENT)),dtype=float)
    far = np.asarray(canopy.filter(ImageFilter.GaussianBlur(.43*SIZE/EXTENT)),dtype=float)
    alpha = (.60*near+.40*far)*.45
    # A restrained penumbra at the groundward edge of the actual corrugated eaves.
    # The GLB node offset is relative to the shelter at world (0, 0, 10).
    roof = Image.new('L',(SIZE,SIZE))
    roof_pen = ImageDraw.Draw(roof)
    roof_bytes = (destination.parent/'smoko_shelter.glb').read_bytes()
    roof_size = struct.unpack_from('<I',roof_bytes,12)[0]
    shelter = json.loads(roof_bytes[20:20+roof_size])
    roof_blob = roof_bytes[28+roof_size:]
    roof_node = shelter['nodes'][0]
    offset = np.array(roof_node['translation'])+[0,0,10]
    def values(index, dtype, width):
        acc=shelter['accessors'][index]
        view=shelter['bufferViews'][acc['bufferView']]
        assert 'byteStride' not in view
        return np.frombuffer(roof_blob,dtype,count=acc['count']*width,
            offset=view.get('byteOffset',0)+acc.get('byteOffset',0)).reshape(-1,width)
    for prim in shelter['meshes'][roof_node['mesh']]['primitives']:
        if not shelter['materials'][prim['material']]['name'].startswith('Corrugated iron'):
            continue
        pts=values(prim['attributes']['POSITION'],'<f4',3)+offset
        proj=(pts[:,[0,2]]-pts[:,1,None]*SUN_XZ_PER_METRE-[X0,Z0])*SIZE/EXTENT
        index=prim['indices']
        triangles=values(index,'<u2' if shelter['accessors'][index]['componentType']==5123 else '<u4',1).reshape(-1,3)
        for tri in proj[triangles]:
            roof_pen.polygon([tuple(p) for p in tri],fill=255)
    roof_soft=np.asarray(roof.filter(ImageFilter.GaussianBlur(.38*SIZE/EXTENT)),dtype=float)
    alpha=np.maximum(alpha, .16*roof_soft)
    atlas=Image.new('RGBA',(SIZE,SIZE),(64,53,50,255))
    atlas.putalpha(Image.fromarray(np.uint8(np.round(alpha))))
    atlas.save(shade_path,optimize=True)
    doc.setdefault('extras',{})['picnicCanopyBaked']=True
    blob.extend(bytes(-len(blob)%4))
    doc['buffers'][0]['byteLength']=len(blob)
    header=json.dumps(doc,separators=(',',':')).encode()
    header+=b' '*(-len(header)%4)
    destination.write_bytes(struct.pack('<4sIII4s',b'glTF',2,28+len(header)+len(blob),len(header),b'JSON')+header+struct.pack('<I4s',len(blob),b'BIN\0')+blob)
    print(f'{count} leaves; {shade_path.stat().st_size} atlas bytes')


if __name__=='__main__':
    bake(*(Path(arg) for arg in sys.argv[1:]))
