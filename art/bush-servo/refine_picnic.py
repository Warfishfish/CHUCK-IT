"""One east picnic setting. Run in Blender after build_props.py, not instead of it.
Reads the untouched editable base kit and writes a separate editable detail scene.
Existing geometry/materials elsewhere are retained; no collision/gameplay edits.
"""
import bpy, math, json, runpy, hashlib
from pathlib import Path
from mathutils import Vector
ROOT=Path('/Users/marcus/Claude/australian-bbq')
LEG_WIDTH=.15
EDGE_BEVEL=.018
with bpy.data.libraries.load(str(ROOT/'art/bush-servo/smoko-props.blend'),link=False) as (src,dst):dst.scenes=src.scenes
scene=dst.scenes[0];scene.name='East picnic detail';bpy.context.window.scene=scene
objects=list(scene.objects)
def selected(o):
 return (7.29<o.location.x<9.71 and 1.35<o.location.y<3.65 and o.name.startswith(('Picnic','Table rim','Cross brace','Bench'))) or (10.34<o.location.x<11.66 and 1.1<o.location.y<1.9 and o.name.startswith(('Esky','Recessed carry','Latch','Lid inset')))
def signature(o):
 return hashlib.sha256(repr((list(o.matrix_world),[tuple(v.co) for v in o.data.vertices],[tuple(p.vertices) for p in o.data.polygons],[m.name for m in o.data.materials])).encode()).hexdigest()
untouched={o.name:signature(o) for o in objects if o.type=='MESH' and not selected(o)}
with bpy.data.libraries.load(str(ROOT/'art/bush-servo/smoko-shelter.blend'),link=False) as (src,dst):
 dst.materials=[next(n for n in sorted(src.materials) if n.startswith('Sun bleached timber '+str(i))) for i in [2,3,4]]+[next(n for n in src.materials if 'end' in n.lower() and 'grain' in n.lower())]
wood=list(dst.materials[:3]);end=dst.materials[3]
cream=next(o.data.materials[0] for o in objects if o.name.startswith('Esky rounded lid'))
steel=next(o.data.materials[0] for o in objects if o.name.startswith('Latch'))
dark=next(o.data.materials[0] for o in objects if o.name.startswith('Esky lid seal'))
blue=next(o.data.materials[0] for o in objects if o.name.startswith('Esky insulated body') and abs(o.location.x-11)<.01)
def timber(o,index=0):
 o.data=o.data.copy();o.data.materials.clear();o.data.materials.append(wood[index%3]);o.data.materials.append(end)
 dims=o.dimensions;axis=max(range(3),key=lambda k:dims[k]);uv=o.data.uv_layers.active
 for p in o.data.polygons:
  normal=max(range(3),key=lambda k:abs(p.normal[k]));p.material_index=1 if normal==axis else 0
  other=next(k for k in range(3) if k!=normal and k!=axis) if normal!=axis else (axis+1)%3
  for li in p.loop_indices:
   v=o.data.vertices[o.data.loops[li].vertex_index].co
   uv.data[li].uv=(v[other]/max(dims[other],.001)+.5,v[axis]/max(dims[axis],.001)+.5) if normal!=axis else (v[(axis+1)%3]/dims[(axis+1)%3]+.5,v[(axis+2)%3]/dims[(axis+2)%3]+.5)
def box(name,loc,dims,material,bevel=.012):
 bpy.ops.object.select_all(action='DESELECT');bpy.ops.mesh.primitive_cube_add(size=1,location=loc);o=bpy.context.object;o.name=name;o.dimensions=dims
 bpy.ops.object.transform_apply(location=False,rotation=False,scale=True);o.data.materials.append(material)
 b=o.modifiers.new('Soft worn edge','BEVEL');b.width=min(bevel,min(dims)*.25);b.segments=2;bpy.ops.object.modifier_apply(modifier=b.name)
 objects.append(o);return o
for i,o in enumerate(list(objects)):
 if not selected(o):continue
 if o.name.startswith(('Picnic','Table rim','Cross brace','Bench')):
  if o.name.startswith('Picnic leg'):o.dimensions.x=LEG_WIDTH;o.dimensions.y=.15
  if o.name.startswith('Cross brace'):o.dimensions.z=.14
  bpy.context.view_layer.objects.active=o;o.select_set(True);bpy.ops.object.transform_apply(location=False,rotation=False,scale=True);o.select_set(False)
  timber(o,i)
  if o.name.startswith('Bench seat'):
   # Two clear seat planks, same exterior width/depth and exact top height.
   x,y,z=o.location;objects.remove(o);bpy.data.objects.remove(o,do_unlink=True)
   for side in [-1,1]:timber(box('Split bench plank',(x,y+side*.09,z),(2.4,.17,.09),wood[1],EDGE_BEVEL),1)
# Supports remain below the table and inside its existing solid.
for x in [7.67,9.33]:
 timber(box('Table underside bearer',(x,2.5,.815),(.18,.84,.09),wood[0]))
# Low central stretcher and small diagonal knees clarify construction.
timber(box('Table low stretcher',(8.5,2.5,.30),(1.8,.11,.12),wood[1]))
for x,sgn in [(7.85,1),(9.15,-1)]:
 o=box('Table knee brace',(x,2.5,.63),(.11,.11,.47),wood[0]);o.rotation_euler.y=sgn*.55;timber(o)
# Esky detail lies entirely inside 1.3 x .75 x .72 collider.
for x in [10.70,11.30]:
 box('Esky rear hinge',(x,1.157,.628),(.18,.055,.065),steel)
 box('Esky latch inset',(x,1.855,.595),(.105,.018,.115),dark,.01)
 box('Esky latch catch',(x,1.865,.637),(.055,.016,.045),steel,.006)
for x in [10.405,11.595]:
 box('Esky grip shadow',(x,1.5,.36),(.012,.245,.075),dark,.012)
 box('Esky grip bridge',(x,1.5,.397),(.020,.235,.028),cream,.009)
for x in [10.51,11.49]:
 box('Esky moulded corner rib',(x,1.845,.30),(.055,.018,.44),blue,.012)
for y in [1.37,1.63]:box('Esky lid channel',(11,y,.718),(.72,.012,.004),steel,.001)
# Verify every old non-target object, plus all new decoration against footprint.
bpy.context.view_layer.update()
assert all(signature(scene.objects[n])==h for n,h in untouched.items())
new=[o for o in objects if o.name not in untouched and o.type=='MESH']
for o in new:
 for v in o.bound_box:
  p=o.matrix_world@Vector(v)
  if o.name.startswith('Esky') or o.name.startswith(('Recessed','Latch','Lid inset')):
   assert 10.3499<=p.x<=11.6501 and 1.1249<=p.y<=1.8751 and -.001<=p.z<=.7201,(o.name,tuple(p))
  else:assert 7.2999<=p.x<=9.7001 and 1.3749<=p.y<=3.6251 and -.001<=p.z<=.9501,(o.name,tuple(p))
report={'unchanged_objects':len(untouched),'table_bounds':[7.3,9.7,2.05,2.95,0,.95],'esky_bounds':[10.35,11.65,1.125,1.875,0,.72],'all_details_inside_existing_bounds':True}
(ROOT/'art/bush-servo/picnic-detail-bounds.json').write_text(json.dumps(report,indent=2))
bpy.data.libraries.write(str(ROOT/'art/bush-servo/picnic-detail.blend'),{scene},fake_user=True,compress=True)
bpy.ops.object.select_all(action='DESELECT')
for o in objects:o.select_set(True)
bpy.context.view_layer.objects.active=objects[0];bpy.ops.object.duplicate();bpy.ops.object.join();asset=bpy.context.object;asset.name='Smoko Props Picnic Detail'
path=ROOT/'rust/crates/bbq_app/assets/models/smoko_props.glb'
bpy.ops.export_scene.gltf(filepath=str(path),use_selection=True,use_active_scene=True,export_apply=True)
print(runpy.run_path(str(ROOT/'art/bush-servo/compact_glb.py'))['compact'](path))
out,tint=runpy.run_path(str(ROOT/'art/bush-servo/finish_arena_depth.py'))['finish'](path.read_bytes(),'props');path.write_bytes(out)
for o in objects:o.hide_set(True);o.hide_render=True
print(report,tint)
