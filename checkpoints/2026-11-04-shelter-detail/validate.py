"""Check runtime cost, fixed clearance and the scope of this asset-only pass."""
from pathlib import Path
import json,struct,subprocess
r=Path(__file__).resolve().parents[2];folder=Path(__file__).parent
stats=json.loads((folder/'asset-costs.json').read_text())
b=(r/'rust/crates/bbq_app/assets/models/smoko_shelter.glb').read_bytes();n=struct.unpack_from('<I',b,12)[0];g=json.loads(b[20:20+n]);blob=b[28+n:]
assert struct.unpack_from('<I',b,8)[0]==len(b)
a={'bytes':len(b),'triangles':sum(g['accessors'][p['indices']]['count']//3 for m in g['meshes'] for p in m['primitives']),'batches':sum(len(m['primitives']) for m in g['meshes']),'images':len(g.get('images',[]))}
assert a['triangles']==stats['before']['triangles'] and a['batches']==stats['before']['batches']
dims=[]
for i in g['images']:
 v=g['bufferViews'][i['bufferView']];data=blob[v['byteOffset']:v['byteOffset']+v['byteLength']];assert data.startswith(b'\x89PNG');dims.append(struct.unpack_from('>II',data,16))
a['image_dimensions']=dims;a['rgba_bytes_with_mips']=sum(w*h*4*4//3 for w,h in dims);stats['after']=a
bounds=json.loads((r/'art/bush-servo/shelter-detail-bounds.json').read_text());assert abs(bounds['eave_lower_edge']-bounds['prior_eave_lower_edge'])<1e-6
assert max(abs(a-b) for a,b in zip(bounds['post_dimensions'],[.24,.24,3.8]))<1e-4
changes=subprocess.check_output(['git','diff','--name-only','9d6c694','--','rust/crates','public/rust'],cwd=r,text=True).splitlines();assert changes==['rust/crates/bbq_app/assets/models/smoko_shelter.glb'],changes
(folder/'asset-costs.json').write_text(json.dumps(stats,indent=2))
print('PASS: unchanged triangle/batch counts, fixed post bounds and eave clearance, no gameplay/exterior/public-bundle changes')
