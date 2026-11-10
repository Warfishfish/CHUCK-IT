"""Finish the two existing Smoko exports. Run with separate input/output model directories."""
import copy
import json
import math
import struct
import sys
from pathlib import Path


def read_glb(path):
    data = path.read_bytes()
    length = struct.unpack_from('<I', data, 12)[0]
    return json.loads(data[20:20 + length]), bytearray(data[28 + length:])


def write_glb(path, doc, blob):
    doc['buffers'][0]['byteLength'] = len(blob)
    data = json.dumps(doc, separators=(',', ':')).encode()
    data += b' ' * (-len(data) % 4)
    path.write_bytes(struct.pack('<4sIII4s', b'glTF', 2, 28 + len(data) + len(blob), len(data), b'JSON')
                     + data + struct.pack('<I4s', len(blob), b'BIN\0') + blob)


def vertices(doc, blob, attr, transform):
    accessor = doc['accessors'][attr]
    view = doc['bufferViews'][accessor['bufferView']]
    positions = []
    for i in range(accessor['count']):
        offset = view.get('byteOffset', 0) + accessor.get('byteOffset', 0) + i * view.get('byteStride', 12)
        pos = transform(*struct.unpack_from('<3f', blob, offset))
        struct.pack_into('<3f', blob, offset, *pos)
        positions.append(pos)
    accessor.update(min=list(map(min, zip(*positions))), max=list(map(max, zip(*positions))))


def sign(doc, blob):
    ox, oy, oz = doc['nodes'][0]['translation']
    def stretch(x, y, z):
        # End boards, lettering and surrounding timber frame lie beyond the fascia.
        if abs(z + oz) > 4.60 and abs(x + ox) < 2.25 and y + oy > 3.41:
            return (x + (x + ox) * .38, y + (y + oy - 3.41) * .18, z)
        return x, y, z
    for prim in doc['meshes'][0]['primitives']:
        vertices(doc, blob, prim['attributes']['POSITION'], stretch)


def shoulder(doc, blob):
    # Reuse the inner shoulder's baked material, UVs and index buffers. Reversing
    # angle and radial direction together preserves winding on the outer shoulder.
    source = next(m for m in doc['meshes'] if m['name'] == 'East picnic weathered bitumen')
    mesh = copy.deepcopy(source)
    mesh['name'] = 'Eastern entrance dust lip'
    if any(m['name'] == mesh['name'] for m in doc['meshes']):
        raise ValueError('Use the original surroundings as input')
    prim = mesh['primitives'][0]
    acc = copy.deepcopy(doc['accessors'][prim['attributes']['POSITION']])
    view = copy.deepcopy(doc['bufferViews'][acc['bufferView']])
    start = view['byteOffset']
    view['byteOffset'] = len(blob)
    blob.extend(blob[start:start + view['byteLength']])
    acc['bufferView'] = len(doc['bufferViews'])
    doc['bufferViews'].append(view)
    index = len(doc['accessors'])
    doc['accessors'].append(acc)
    prim['attributes']['POSITION'] = index

    def lip(angle):
        return .18 * math.sin(17 * angle) + .11 * math.sin(39 * angle + 1.2)

    def outer(x, y, z):
        north = (6 - z) / .65
        a = math.atan2(north, x)
        depth = math.hypot(x, north) - 18 - lip(a)
        b = math.radians(78) - (a - math.radians(24)) * .91
        r = 26 + 1.4 * lip(b) - depth * .48
        return r * math.cos(b), y, 6 - .65 * r * math.sin(b)

    vertices(doc, blob, index, outer)
    doc['scenes'][doc.get('scene', 0)]['nodes'].append(len(doc['nodes']))
    doc['nodes'].append({'mesh': len(doc['meshes']), 'name': mesh['name']})
    doc['meshes'].append(mesh)


if __name__ == '__main__':
    original, output = map(Path, sys.argv[1:])
    if original.resolve() == output.resolve():
        raise SystemExit('Choose separate original and output model directories')
    output.mkdir(parents=True, exist_ok=True)
    for kind, edit in [('shelter', sign), ('surroundings', shoulder)]:
        name = f'smoko_{kind}.glb'
        doc, blob = read_glb(original / name)
        edit(doc, blob)
        write_glb(output / name, doc, blob)
