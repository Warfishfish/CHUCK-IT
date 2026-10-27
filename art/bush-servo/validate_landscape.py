"""Validate authored bounds, shared collider table and glTF asset cost."""
import json,re,struct
from pathlib import Path
r=Path(__file__).resolve().parents[2]
layout=json.loads((r/'art/bush-servo/landscape-layout.json').read_text())['cover']
s=(r/'rust/crates/bbq_core/src/bush_servo.rs').read_text().split('pub const NATURE_SOLIDS:')[1].split('];')[0]
rows=[[float(v) for v in row.split(',') if v] for row in re.findall(r'\(([-\d.,]+)\)',s)]
assert rows==layout
bounds=json.loads((r/'art/bush-servo/landscape-bounds.json').read_text())
assert len(bounds)==len(rows)
for row,b in zip(rows,bounds): assert all(abs(a-c)<.001 for a,c in zip(row[2:],b['dimensions']))
b=(r/'rust/crates/bbq_app/assets/models/smoko_landscape.glb').read_bytes();n=struct.unpack_from('<I',b,12)[0];g=json.loads(b[20:20+n])
assert all('uri' not in x for x in g.get('buffers',[]))
report={'cover_bounds_match':True,'solid_boulders':len(rows),'triangles':sum(g['accessors'][p['indices']]['count']//3 for m in g['meshes'] for p in m['primitives']),'material_batches':sum(len(m['primitives']) for m in g['meshes']),'glb_bytes':len(b),'textures':len(g.get('images',[]))}
(r/'checkpoints/2026-10-27-servo-landscape/evidence/asset-validation.json').write_text(json.dumps(report,indent=2)+'\n')
print(report)
