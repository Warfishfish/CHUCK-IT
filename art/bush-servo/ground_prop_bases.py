"""Bake local contact and earth wear into a single feathered Bush Servo mesh.

Run with the unpainted surroundings GLB as the first argument and the runtime
destination as the second. Existing model data and materials are reused.
"""
import json
import math
import struct
import sys
from pathlib import Path

NAME = "East gum and picnic ground wear"
GROUND_Y = -0.018
SURFACE_Y = GROUND_Y + 0.006
SEGMENTS = 32
# Linear RGB and opacity: occlusion at the foot, compressed soil, dusty fringe.
RINGS = [
    (0.0, (.038, .019, .009), .62),
    (.48, (.055, .029, .012), .55),
    (.65, (.11, .054, .023), .43),
    (.81, (.27, .135, .055), .25),
    (1.0, (.33, .17, .075), 0.0),
]

def stamp(positions, colours, indices, x, z, width, depth, squareness, phase):
    """Concentric feathered rings. A rounded rectangle fits the esky's base."""
    centre = len(positions)
    for ring, (radius, rgb, alpha) in enumerate(RINGS):
        count = SEGMENTS if ring else 1
        for i in range(count):
            a = math.tau * i / SEGMENTS
            c, s = math.cos(a), math.sin(a)
            warp = 1 + radius * (.07 * math.sin(3*a+phase) + .045 * math.sin(7*a-phase))
            def axis(v):
                return math.copysign(abs(v)**squareness, v)
            positions.append((x+width*radius*warp*axis(c), SURFACE_Y, z+depth*radius*warp*axis(s)))
            colours.append(tuple(round(255*k) for k in (*rgb, alpha)))
        if ring:
            start = centre+1+(ring-1)*SEGMENTS
            for i in range(SEGMENTS):
                a, b = start+i, start+(i+1)%SEGMENTS
                if ring == 1:
                    indices.extend((centre,b,a))
                else:
                    indices.extend((a-SEGMENTS,b,a,b-SEGMENTS,b,a-SEGMENTS))

def bake(source, destination):
    raw = source.read_bytes()
    size = struct.unpack_from('<I',raw,12)[0]
    doc = json.loads(raw[20:20+size])
    assert all(n.get('name') != NAME for n in doc['nodes']), 'Use the original surroundings export'
    blob = bytearray(raw[28+size:])
    material = next(i for i,m in enumerate(doc['materials']) if m.get('name','').startswith('Entrance dust feather') and m.get('alphaMode')=='BLEND')
    positions, colours, indices = [], [], []
    footprints = [(13,-.3,.98,.90,1.0), (13.09,1.2,.31,.34,.72), (14.91,1.2,.32,.34,.72),
                  (11,-1.5,.97,.63,.48)]
    for dx in [-.83,.83]:
        for dz in [-.28,.28,-.95,.95]:
            footprints.append((8.5+dx,-2.5+dz,.24,.27,.85))
    for i, footprint in enumerate(footprints):
        stamp(positions,colours,indices,*footprint,i*1.73)
    def accessor(rows, fmt, component, kind, target, **extra):
        blob.extend(bytes(-len(blob)%4))
        offset = len(blob)
        for row in rows:
            blob.extend(struct.pack('<'+fmt,*row))
        view = len(doc['bufferViews'])
        doc['bufferViews'].append(dict(buffer=0,byteOffset=offset,byteLength=len(blob)-offset,target=target))
        index = len(doc['accessors'])
        doc['accessors'].append(dict(bufferView=view,componentType=component,count=len(rows),type=kind,**extra))
        return index
    pos = accessor(positions,'3f',5126,'VEC3',34962,min=list(map(min,zip(*positions))),max=list(map(max,zip(*positions))))
    norm = accessor([(0,1,0)]*len(positions),'3f',5126,'VEC3',34962)
    col = accessor(colours,'4B',5121,'VEC4',34962,normalized=True)
    idx = accessor([(i,) for i in indices],'H',5123,'SCALAR',34963)
    node = len(doc['nodes'])
    doc['nodes'].append(dict(name=NAME,mesh=len(doc['meshes'])))
    doc['meshes'].append(dict(name=NAME,primitives=[dict(attributes=dict(POSITION=pos,NORMAL=norm,COLOR_0=col),indices=idx,material=material)]))
    doc['scenes'][doc.get('scene',0)]['nodes'].append(node)
    blob.extend(bytes(-len(blob)%4))
    doc['buffers'][0]['byteLength']=len(blob)
    header=json.dumps(doc,separators=(',',':')).encode()
    header+=b' '*(-len(header)%4)
    destination.write_bytes(struct.pack('<4sIII4s',b'glTF',2,28+len(header)+len(blob),len(header),b'JSON')+header+struct.pack('<I4s',len(blob),b'BIN\0')+blob)
    print(len(footprints),'contact areas;',len(indices)//3,'triangles;',len(blob)-len(raw[28+size:]),'extra buffer bytes')

if __name__ == '__main__':
    bake(Path(sys.argv[1]),Path(sys.argv[2]))
