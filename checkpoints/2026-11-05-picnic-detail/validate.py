from pathlib import Path
import struct,json,collections,hashlib
r=Path(__file__).resolve().parents[2];cp=Path(__file__).parent

def load(p):
 b=p.read_bytes();n=struct.unpack_from('<I',b,12)[0];d=json.loads(b[20:20+n]);return d,b[28+n:]
def acc(d,b,i):
 a=d['accessors'][i];v=d['bufferViews'][a['bufferView']];fmt={5126:'f',5125:'I',5123:'H',5121:'B'}[a['componentType']]*{'SCALAR':1,'VEC2':2,'VEC3':3,'VEC4':4}[a['type']];sz=struct.calcsize('<'+fmt)
 return [struct.unpack_from('<'+fmt,b,v.get('byteOffset',0)+a.get('byteOffset',0)+j*v.get('byteStride',sz)) for j in range(a['count'])]
def outside(d,b):
 tris=collections.Counter()
 for node in d['nodes']:
  if 'mesh' not in node:continue
  off=node.get('translation',[0,0,0]);assert 'matrix' not in node
  for p in d['meshes'][node['mesh']]['primitives']:
   pos=acc(d,b,p['attributes']['POSITION']);idx=acc(d,b,p['indices'])
   for k in range(0,len(idx),3):
    points=[tuple(round(pos[idx[k+j][0]][a]+off[a],4) for a in range(3)) for j in range(3)]
    # Whole east setting excluded; other geometry must match exactly at 0.1mm.
    if all(not(7.29<=v[0]<=11.66 and -3.64<=v[2]<=-1.11) for v in points):tris[tuple(sorted(points))]+=1
 return tris
old=load(cp/'before-props.glb');new=load(r/'rust/crates/bbq_app/assets/models/smoko_props.glb')
a=outside(*old);b=outside(*new);assert a==b,(len(a-b),len(b-a))
report={'unchanged_outside_triangles':sum(a.values())}
for label,(d,blob),path in [('before',old,cp/'before-props.glb'),('after',new,r/'rust/crates/bbq_app/assets/models/smoko_props.glb')]:
 report[label]={'bytes':path.stat().st_size,'triangles':sum(d['accessors'][p['indices']]['count']//3 for m in d['meshes'] for p in m['primitives']),'batches':sum(len(m['primitives']) for m in d['meshes']),'images':len(d['images'])}
start=json.loads((cp/'starting-state.json').read_text())['sha256'];changed=[]
for p,h in start.items():
 if (r/p).is_file() and hashlib.sha256((r/p).read_bytes()).hexdigest()!=h:changed.append(p)
assert all(p in ['rust/crates/bbq_app/assets/models/smoko_props.glb','checkpoints/LATEST.md'] for p in changed),changed
report['existing_files_changed']=changed
(cp/'validation.json').write_text(json.dumps(report,indent=2));print(json.dumps(report,indent=2))
