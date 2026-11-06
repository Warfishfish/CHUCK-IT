"""Extend the east picnic finish to the west setting, sharing meshes/materials.
Pipeline: build_props.py -> refine_picnic.py -> extend_picnic.py.
Reads east authoring scene; leaves it and every non-west object unchanged.
"""
import bpy,json,runpy,hashlib
from pathlib import Path
from mathutils import Vector
ROOT=Path('/Users/marcus/Claude/australian-bbq')
with bpy.data.libraries.load(str(ROOT/'art/bush-servo/picnic-detail.blend'),link=False) as (src,dst):dst.scenes=src.scenes
scene=dst.scenes[0];scene.name='Both picnic settings';bpy.context.window.scene=scene
objects=list(scene.objects)
TABLE_PREFIX=('Picnic','Table rim','Cross brace','Bench','Split bench','Table underside','Table low','Table knee')
ESKY_PREFIX=('Esky','Recessed carry','Latch','Lid inset')
def part(o,side):
 if o.type!='MESH':return None
 if abs(o.location.x-side*8.5)<1.21 and 1.35<o.location.y<3.65 and o.name.startswith(TABLE_PREFIX):return 'table'
 if abs(o.location.x-side*11)<.66 and 1.1<o.location.y<1.9 and o.name.startswith(ESKY_PREFIX):return 'esky'
def signature(o):return hashlib.sha256(repr((list(o.matrix_world),[tuple(v.co) for v in o.data.vertices],[tuple(p.vertices) for p in o.data.polygons],[m.name for m in o.data.materials])).encode()).hexdigest()
west=[o for o in objects if part(o,-1)]
east=[o for o in objects if part(o,1)]
assert len(west)==27 and len(east)==48,(len(west),len(east))
unchanged={o.name:signature(o) for o in objects if o not in west}
for o in west:objects.remove(o);bpy.data.objects.remove(o,do_unlink=True)
added=[]
for original in east:
 o=original.copy();scene.collection.objects.link(o);o.location.x-=17 if part(original,1)=='table' else 22
 o.name='West '+original.name;objects.append(o);added.append(o)
bpy.context.view_layer.update()
assert all(signature(scene.objects[n])==h for n,h in unchanged.items())
for o in added:
 esky=o.name.startswith(tuple('West '+p for p in ESKY_PREFIX))
 for v in o.bound_box:
  p=o.matrix_world@Vector(v)
  if esky:assert -11.6501<=p.x<=-10.3499 and 1.1249<=p.y<=1.8751 and -.001<=p.z<=.7201,(o.name,tuple(p))
  else:assert -9.7001<=p.x<=-7.2999 and 1.3749<=p.y<=3.6251 and -.001<=p.z<=.9501,(o.name,tuple(p))
report={'preserved_objects':len(unchanged),'replaced_west_objects':len(west),'shared_detail_objects':len(added),'new_materials':0,'new_textures':0,'same_collision_footprints':True}
(ROOT/'art/bush-servo/picnic-pair-bounds.json').write_text(json.dumps(report,indent=2))
bpy.data.libraries.write(str(ROOT/'art/bush-servo/picnic-pair.blend'),{scene},fake_user=True,compress=True)
bpy.ops.object.select_all(action='DESELECT')
for o in objects:o.select_set(True)
bpy.context.view_layer.objects.active=objects[0];bpy.ops.object.duplicate();bpy.ops.object.join();asset=bpy.context.object;asset.name='Smoko Props Both Picnic Settings'
path=ROOT/'rust/crates/bbq_app/assets/models/smoko_props.glb'
bpy.ops.export_scene.gltf(filepath=str(path),use_selection=True,use_active_scene=True,export_apply=True)
print(runpy.run_path(str(ROOT/'art/bush-servo/compact_glb.py'))['compact'](path))
out,tint=runpy.run_path(str(ROOT/'art/bush-servo/finish_arena_depth.py'))['finish'](path.read_bytes(),'props');path.write_bytes(out)
for o in objects:o.hide_set(True);o.hide_render=True
print(report,tint)
