"""Refine only the green shelter-side esky and adjacent bin.
Pipeline: build_props -> refine_picnic -> extend_picnic -> this script.
Shared existing materials; no new obstacles, footprint or gameplay changes.
"""
import bpy,json,runpy,hashlib
from pathlib import Path
from mathutils import Vector
ROOT=Path('/Users/marcus/Claude/australian-bbq')
SEAM_WIDTH=.012
with bpy.data.libraries.load(str(ROOT/'art/bush-servo/picnic-pair.blend'),link=False) as (src,dst):dst.scenes=src.scenes
scene=dst.scenes[0];scene.name='Shelter side prop details';bpy.context.window.scene=scene
objects=list(scene.objects)
PREFIX=('Esky','Recessed carry','Latch','Lid inset')
def esky(o,x,y):return o.type=='MESH' and abs(o.location.x-x)<.66 and abs(o.location.y-y)<.40 and o.name.startswith(PREFIX)
def signature(o):return hashlib.sha256(repr((list(o.matrix_world),[tuple(v.co) for v in o.data.vertices],[tuple(p.vertices) for p in o.data.polygons],[m.name for m in o.data.materials])).encode()).hexdigest()
old=[o for o in objects if esky(o,5.6,-15.9)];source=[o for o in objects if esky(o,11,1.5)]
assert len(old)==8 and len(source)==22,(len(old),len(source))
teal=next(o.data.materials[0] for o in old if o.name.startswith('Esky insulated'))
blue=next(o.data.materials[0] for o in source if o.name.startswith('Esky insulated'))
steel=next(o.data.materials[0] for o in source if o.name.startswith('Latch'))
dark=next(o.data.materials[0] for o in source if o.name.startswith('Esky lid seal'))
unchanged={o.name:signature(o) for o in objects if o not in old}
for o in old:objects.remove(o);bpy.data.objects.remove(o,do_unlink=True)
added=[]
for original in source:
 o=original.copy();scene.collection.objects.link(o);o.location+=Vector((-5.4,-17.4,0));o.name='Shelter '+original.name
 if blue in list(o.data.materials):
  o.data=o.data.copy()
  for i,m in enumerate(o.data.materials):
   if m==blue:o.data.materials[i]=teal
 objects.append(o);added.append(o)
def box(name,loc,dims,mat,bevel=.008):
 bpy.ops.object.select_all(action='DESELECT');bpy.ops.mesh.primitive_cube_add(size=1,location=loc);o=bpy.context.object;o.name=name;o.dimensions=dims
 bpy.ops.object.transform_apply(location=False,rotation=False,scale=True);o.data.materials.append(mat)
 b=o.modifiers.new('Soft fittings','BEVEL');b.width=min(bevel,min(dims)*.25);b.segments=2;bpy.ops.object.modifier_apply(modifier=b.name)
 # Upright texture projection for existing worn metal/plastic maps.
 uv=o.data.uv_layers.active
 for p in o.data.polygons:
  axis=max(range(3),key=lambda k:abs(p.normal[k]))
  for li in p.loop_indices:
   v=o.data.vertices[o.data.loops[li].vertex_index].co;w,d,h=dims
   uv.data[li].uv=(v.y/d+.5,v.z/h+.5) if axis==0 else ((v.x/w+.5,v.z/h+.5) if axis==1 else (v.x/w+.5,v.y/d+.5))
 objects.append(o);added.append(o);return o
# Bin at game (7.2,14.8), original solid .72 x .72 x 1.1.
x,y=7.2,-14.8
box('Bin recessed lid seam',(x,y,1.002),(.705,.705,.016),dark)
box('Bin lower protective rim',(x,y,.035),(.70,.70,.07),dark,.016)
# Front is positive Blender Y: reinforce the existing dark inset without protruding.
for dx in [-.242,.242]:box('Bin aperture frame',(x+dx,y+.345,.704),(.024,.020,.29),steel)
for zz in [.565,.843]:box('Bin aperture edge',(x,y+.345,zz),(.50,.020,.024),steel)
box('Bin inset lower flap',(x,y+.351,.592),(.42,.012,.045),teal)
for dx in [-.21,.21]:
 box('Bin lid rear hinge',(x+dx,y-.345,1.035),(.12,.026,.065),steel)
 for zz in [.16,.91]:
  bpy.ops.mesh.primitive_uv_sphere_add(segments=12,ring_count=6,radius=1,location=(x+dx,y+.345,zz));o=bpy.context.object;o.name='Bin rounded rivet';o.scale=(.012,.007,.012);o.data.materials.append(steel);objects.append(o);added.append(o)
for side in [-1,1]:
 for dy in [-.23,.23]:box('Bin pressed side seam',(x+side*.34,y+dy,.51),(SEAM_WIDTH,.025,.77),teal)
bpy.context.view_layer.update()
assert all(signature(scene.objects[n])==h for n,h in unchanged.items())
for o in added:
 for v in o.bound_box:
  p=o.matrix_world@Vector(v)
  if o.name.startswith('Shelter '):assert 4.9499<=p.x<=6.2501 and -16.2751<=p.y<=-15.5249 and -.001<=p.z<=.7201,(o.name,tuple(p))
  else:assert 6.8399<=p.x<=7.5601 and -15.1601<=p.y<=-14.4399 and -.001<=p.z<=1.1001,(o.name,tuple(p))
report={'preserved_objects':len(unchanged),'old_esky_objects':len(old),'detail_objects':len(added),'same_collision_bounds':True,'new_materials':0,'new_textures':0}
(ROOT/'art/bush-servo/shelter-prop-bounds.json').write_text(json.dumps(report,indent=2))
bpy.data.libraries.write(str(ROOT/'art/bush-servo/shelter-props-detail.blend'),{scene},fake_user=True,compress=True)
bpy.ops.object.select_all(action='DESELECT')
for o in objects:o.select_set(True)
bpy.context.view_layer.objects.active=objects[0];bpy.ops.object.duplicate();bpy.ops.object.join();asset=bpy.context.object;asset.name='Smoko Props Shelter Side Detail'
path=ROOT/'rust/crates/bbq_app/assets/models/smoko_props.glb'
bpy.ops.export_scene.gltf(filepath=str(path),use_selection=True,use_active_scene=True,export_apply=True)
print(runpy.run_path(str(ROOT/'art/bush-servo/compact_glb.py'))['compact'](path))
out,tint=runpy.run_path(str(ROOT/'art/bush-servo/finish_arena_depth.py'))['finish'](path.read_bytes(),'props');path.write_bytes(out)
for o in objects:o.hide_set(True);o.hide_render=True
print(report,tint)
