"""Split the Backyard props export into fixed scenery and a movable wheelbarrow."""
from copy import deepcopy
import json
from pathlib import Path
import runpy
import struct
import sys


def export(source, scenery, wheelbarrow):
    data = source.read_bytes()
    size = struct.unpack_from('<I', data, 12)[0]
    doc = json.loads(data[20:20 + size])
    original = data[28 + size:]
    assert len(doc['nodes']) == len(doc['meshes']) == 1

    def values(index, fmt):
        acc = doc['accessors'][index]
        view = doc['bufferViews'][acc['bufferView']]
        assert 'byteStride' not in view
        offset = view.get('byteOffset', 0) + acc.get('byteOffset', 0)
        return list(struct.iter_unpack(fmt, original[offset:offset + struct.calcsize(fmt) * acc['count']]))

    groups = [[], []]
    blob = bytearray(original)
    for prim in doc['meshes'][0]['primitives']:
        material = doc['materials'][prim['material']]['name']
        pos = values(prim['attributes']['POSITION'], '<3f')
        src = doc['accessors'][prim['indices']]
        fmt = {5123: '<H', 5125: '<I'}[src['componentType']]
        indices = values(prim['indices'], fmt)
        split = [[], []]
        for start in range(0, len(indices), 3):
            triangle = indices[start:start+3]
            ours = all(3.4 <= pos[i][0] <= 5 and 14.36 <= pos[i][2] <= 16.66 for (i,) in triangle)
            if ours and material.startswith(('Fresh garden soil', 'Moist soil', 'Worn hand tool', 'Fresh green', 'Weathered garden timber')):
                continue
            split[int(ours)].extend(i for (i,) in triangle)

        for group, selected in zip(groups, split):
            if not selected:
                continue
            blob.extend(bytes(-len(blob) % 4))
            packed = b''.join(struct.pack(fmt, i) for i in selected)
            view = len(doc['bufferViews'])
            doc['bufferViews'].append(dict(buffer=0, byteOffset=len(blob), byteLength=len(packed), target=34963))
            blob.extend(packed)
            index = len(doc['accessors'])
            doc['accessors'].append(dict(bufferView=view, componentType=src['componentType'], count=len(selected),
                                          type='SCALAR', min=[min(selected)], max=[max(selected)]))
            group.append(dict(prim, indices=index))

    assert len(groups[1]) >= 5
    compact = runpy.run_path(str(Path(__file__).resolve().parents[2] / 'art/bush-servo/compact_glb.py'))['compact']
    blob.extend(bytes(-len(blob) % 4))
    for path, group in zip((scenery, wheelbarrow), groups):
        out = deepcopy(doc)
        out['meshes'][0]['primitives'] = group
        out['buffers'][0]['byteLength'] = len(blob)
        header = json.dumps(out, separators=(',', ':')).encode()
        header += b' ' * (-len(header) % 4)
        path.write_bytes(struct.pack('<4sIII4s', b'glTF', 2, 28+len(header)+len(blob), len(header), b'JSON')
                         + header + struct.pack('<I4s', len(blob), b'BIN\0') + blob)
        compact(path)


if __name__ == '__main__':
    export(*(Path(value) for value in sys.argv[1:]))
