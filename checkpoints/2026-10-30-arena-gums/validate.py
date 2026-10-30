"""Verify the new tree preserves the previous scenery and matches shared collision."""
from pathlib import Path
import struct,json,tarfile,re
r=Path(__file__).resolve().parents[2];c=Path(__file__).parent
asset='rust/crates/bbq_app/assets/models/smoko_landscape.glb'
def inspect(b):
 assert struct.unpack_from('<I',b,8)[0]==len(b)
 n=struct.unpack_from('<I',b,12)[0];g=json.loads(b[20:20+n]);binary=b[28+n:];positions=set()
 for m in g['meshes']:
  for p in m['primitives']:
   a=g['accessors'][p['attributes']['POSITION']];v=g['bufferViews'][a['bufferView']];start=v.get('byteOffset',0)+a.get('byteOffset',0);stride=v.get('byteStride',12)
   for i in range(a['count']):positions.add(tuple(round(x,5) for x in struct.unpack_from('<fff',binary,start+i*stride)))
 return positions,{'bytes':len(b),'triangles':sum(g['accessors'][p['indices']]['count']//3 for m in g['meshes'] for p in m['primitives']),'batches':sum(len(m['primitives']) for m in g['meshes'])}
with tarfile.open(r.parent/'australian-bbq-before-arena-gums-2026-10-30.tar.gz') as t:before,bc=inspect(t.extractfile(asset).read())
after,ac=inspect((r/asset).read_bytes())
# Blender's merged/exported float vertices can shift by a few micrometres.
missing=before-after; extra=after-before
for p in missing:
 nearest=min(extra,key=lambda q:sum((a-b)**2 for a,b in zip(p,q)))
 assert sum((a-b)**2 for a,b in zip(p,nearest))<(.00002)**2,'Previous geometry changed'
 extra.remove(nearest)
added=extra
assert all(-18<x<-5 and 0<=y<11 and -5<z<9 for x,y,z in added),'New geometry outside arena slice'
layout=json.loads((r/'art/bush-servo/arena-trees.json').read_text())['trunks']
s=(r/'rust/crates/bbq_core/src/bush_servo.rs').read_text().split('pub const ARENA_TREE_TRUNKS:')[1].split('];')[0]
rows=[[float(v) for v in row.split(',') if v] for row in re.findall(r'\(([-\d.,]+)\)',s)]
assert rows==layout
report={'before':bc,'after':ac,'previous_vertices_preserved_within_0_02mm':True,'new_vertices_inside_arena':len(added),'shared_trunks_match':True}
(c/'evidence/asset-validation.json').write_text(json.dumps(report,indent=2));print(report)
