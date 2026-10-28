"""Check authored prop bounds against shared gameplay solids and report runtime costs."""
from pathlib import Path
import json,re,struct
r=Path(__file__).resolve().parents[2]
layout=json.loads((r/'art/bush-servo/props-layout.json').read_text())
report=json.loads((r/'art/bush-servo/props-bounds.json').read_text())
source=(r/'rust/crates/bbq_core/src/bush_servo.rs').read_text()
block=source.split('pub const PROP_SOLIDS:',1)[1].split('= [',1)[1].split('];',1)[0]
shared=[list(map(float,m.split(','))) for m in re.findall(r'\(([^()]+)\)',block)]
authored=[p['solid'] for p in report if p['name']!='esky0']
assert len(shared)==len(authored)==20
assert sorted(shared)==sorted(authored)
for p in report:
    x,z,w,d,h=p['solid'];lo=p['visual_min'];hi=p['visual_max']
    assert max(abs(a-b) for a,b in zip([lo[0],hi[0],-hi[1],-lo[1],hi[2]],[x-w/2,x+w/2,z-d/2,z+d/2,h]))<.025,p['name']
data=(r/'rust/crates/bbq_app/assets/models/smoko_props.glb').read_bytes()
n=struct.unpack_from('<I',data,12)[0];g=json.loads(data[20:20+n]);a=g['accessors']
assert not any('uri' in i for i in g.get('images',[])+g['buffers'])
result={'new_colliders':len(shared),'detailed_existing_esky':1,'eskies':len(layout['eskies']),
 'bins':len(layout['bins']),'tyre_stacks':len(layout['tyres']),'picnic_tables':len(layout['tables']),
 'roadside_signs':len(layout['signs']),'triangles':sum(a[p['indices']]['count']//3 for m in g['meshes'] for p in m['primitives']),
 'material_batches':len(g['materials']),'glb_bytes':len(data),'images':len(g.get('images',[]))}
print(json.dumps(result,indent=2))
