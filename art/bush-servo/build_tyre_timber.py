"""One arena gum prop group; run after refine_entrance.py. Existing assets retained."""
from pathlib import Path
import bpy, math, json, hashlib
from mathutils import Vector
ROOT=Path('/Users/marcus/Claude/australian-bbq')
# Reuse established modelling/export functions, without running the earlier scene edits.
source=(ROOT/'art/bush-servo/refine_entrance.py').read_text().split('# Crates are actual shared')[0]
exec(compile(source,'refine_entrance helpers','exec'))
s=load('arena-crates.blend','Arena tyre and timber group')
def signature(o):return hashlib.sha256(repr((list(o.matrix_world),[tuple(v.co) for v in o.data.vertices],[tuple(p.vertices) for p in o.data.polygons],[m.name for m in o.data.materials])).encode()).hexdigest()
original={o.name:signature(o) for o in s.objects if o.type=='MESH'}
wood=next(o.data.materials[0] for o in s.objects if o.name.startswith('Split bench plank'))
steel=next(o.data.materials[0] for o in s.objects if o.name.startswith('Latch'))
rubber=next(o.data.materials[0] for o in s.objects if o.name.startswith('Tyre sidewall'))
edge=next(o.data.materials[0] for o in s.objects if o.name.startswith('Raised tyre lettering ring'))
# Radial sidewall profile: open centre, flattened tread and rolled shoulders.
profile=[(.28,.05),(.31,.015),(.44,0),(.51,.04),(.55,.10),(.55,.26),(.51,.32),(.44,.36),(.31,.345),(.28,.31)]
added=[]
for layer in range(2):
 verts=[];faces=[];N=48
 for rad,h in profile:
  for i in range(N):
   a=i*math.tau/N;verts.append((13+rad*math.cos(a),.3+rad*math.sin(a),layer*.36+h))
 for j in range(len(profile)):
  for i in range(N):faces.append((j*N+i,j*N+(i+1)%N,((j+1)%len(profile))*N+(i+1)%N,((j+1)%len(profile))*N+i))
 mesh=bpy.data.meshes.new('Worn tyre profile');mesh.from_pydata(verts,[],faces);mesh.update();o=bpy.data.objects.new('Arena worn tyre',mesh);s.collection.objects.link(o);mesh.materials.append(rubber);mesh.materials.append(edge)
 for face in mesh.polygons:
  face.use_smooth=True
  # Alternating restrained tread bands and shoulder highlight, no extra geometry.
  band=face.index//N;face.material_index=1 if band in [2,6] or (band==4 and (face.index%N)%4==0) else 0
 added.append(o)
# Three worn horizontal planks, stout posts and small inset bolts.
for dx in [-.91,.91]:added.append(box('Arena timber end post',(14+dx,-1.2,.4),(.18,.28,.8),wood,.018))
for h in [.14,.37,.60]:added.append(box('Arena timber rail',(14,-1.2,h),(1.82,.18,.20),wood,.013))
for dx in [-.91,.91]:
 for h in [.14,.60]:added.append(box('Arena timber bolt',(14+dx,-1.337,h),(.035,.006,.035),steel,.004))
bpy.context.view_layer.update()
assert all(signature(s.objects[n])==h for n,h in original.items())
for o in added:
 for v in o.bound_box:
  p=o.matrix_world@Vector(v)
  if o.name.startswith('Arena worn tyre'):assert 12.4499<=p.x<=13.5501 and -.2501<=p.y<=.8501 and -.0001<=p.z<=.7201
  else:assert 12.9999<=p.x<=15.0001 and -1.3401<=p.y<=-1.0599 and -.0001<=p.z<=.8001
report={'preserved_objects':len(original),'new_objects':len(added),'new_textures':0,'new_materials':0,'solids':[[13,-.3,1.1,1.1,.72],[14,1.2,2,.28,.8]]}
(ROOT/'art/bush-servo/tyre-timber-bounds.json').write_text(json.dumps(report,indent=2))
export(s,'arena-tyre-timber.blend','props')
print(report)
