"""One south shelter entrance and three arena supply crates. Existing scenery retained.
Run after refine_shelter_props.py; writes separate editable scenes and runtime assets.
"""
import bpy,math,json,random,runpy,hashlib
from pathlib import Path
from mathutils import Vector
ROOT=Path('/Users/marcus/Claude/australian-bbq');rng=random.Random(71126)
CRATES=[(-15.,3.,1.1,.8,.65),(15.,3.,1.1,.8,.65),(-10.,12.5,1.1,.8,.65)]
def load(file,name):
 with bpy.data.libraries.load(str(ROOT/'art/bush-servo'/file),link=False) as (src,dst):dst.scenes=src.scenes
 s=dst.scenes[0];s.name=name;bpy.context.window.scene=s;return s
def box(name,loc,dims,mat,bevel=.012):
 bpy.ops.object.select_all(action='DESELECT');bpy.ops.mesh.primitive_cube_add(size=1,location=loc);o=bpy.context.object;o.name=name;o.dimensions=dims
 bpy.ops.object.transform_apply(location=False,rotation=False,scale=True);o.data.materials.append(mat)
 b=o.modifiers.new('Worn corners','BEVEL');b.width=min(bevel,min(dims)*.24);b.segments=2;bpy.ops.object.modifier_apply(modifier=b.name)
 uv=o.data.uv_layers.active;long=max(range(3),key=lambda k:dims[k])
 for p in o.data.polygons:
  normal=max(range(3),key=lambda k:abs(p.normal[k]));axes=[k for k in range(3) if k!=normal];vaxis=long if long!=normal else axes[1];uaxis=next(k for k in axes if k!=vaxis)
  for li in p.loop_indices:
   v=o.data.vertices[o.data.loops[li].vertex_index].co;uv.data[li].uv=(v[uaxis]/dims[uaxis]+.5,v[vaxis]/dims[vaxis]+.5)
 return o
def export(scene,author,kind):
 objs=[o for o in scene.objects if o.type=='MESH'];bpy.data.libraries.write(str(ROOT/'art/bush-servo'/author),{scene},fake_user=True,compress=True)
 bpy.ops.object.select_all(action='DESELECT')
 joinable=[o for o in objs if not o.data.color_attributes]
 separate=[o for o in objs if o.data.color_attributes]
 for o in joinable:o.select_set(True)
 bpy.context.view_layer.objects.active=joinable[0];bpy.ops.object.duplicate();bpy.ops.object.join();o=bpy.context.object;o.name='Arena entrance '+kind
 for item in separate:item.select_set(True)
 p=ROOT/f'rust/crates/bbq_app/assets/models/smoko_{kind}.glb'
 bpy.ops.export_scene.gltf(filepath=str(p),use_selection=True,use_active_scene=True,export_apply=True)
 runpy.run_path(str(ROOT/'art/bush-servo/compact_glb.py'))['compact'](p)
 data,report=runpy.run_path(str(ROOT/'art/bush-servo/finish_arena_depth.py'))['finish'](p.read_bytes(),kind);p.write_bytes(data)
 for a in objs:a.hide_set(True);a.hide_render=True
 print(kind,report)
# Crates are actual shared low cover, with tabletop accessories kept within their solids.
s=load('shelter-props-detail.blend','Arena supply crates')
wood=next(o.data.materials[0] for o in s.objects if o.name.startswith('Split bench plank'))
steel=next(o.data.materials[0] for o in s.objects if o.name.startswith('Latch'))
dark=next(o.data.materials[0] for o in s.objects if o.name.startswith('Esky lid seal'))
report=[]
for number,(x,z,w,d,h) in enumerate(CRATES):
 new=[]
 new.append(box('Supply crate dark interior',(x,-z,h*.48),(w-.08,d-.08,h-.08),dark))
 for level in range(4):
  y=.08+level*.145
  for side in [-1,1]:
   new.append(box('Supply crate side board',(x,-z+side*(d/2-.025),y),(w-.04,.05,.12),wood))
   new.append(box('Supply crate end board',(x+side*(w/2-.025),-z,y),(.05,d-.04,.12),wood))
 for dx in [-w/2+.045,w/2-.045]:
  for dz in [-d/2+.045,d/2-.045]:new.append(box('Supply crate corner post',(x+dx,-z+dz,h/2),(.09,.09,h),wood))
 for i in range(5):new.append(box('Supply crate lid plank',(x,-z-d/2+(i+.5)*d/5,h-.025),(w,d/5-.009,.05),wood))
 for dx in [-.34,.34]:new.append(box('Crate lid strap',(x+dx,-z,h-.003),(.035,d-.03,.006),steel,.001))
 bpy.context.view_layer.update();pts=[o.matrix_world@Vector(v) for o in new for v in o.bound_box];lo=[min(v[k] for v in pts) for k in range(3)];hi=[max(v[k] for v in pts) for k in range(3)]
 assert max(abs(a-b) for a,b in zip(lo+hi,[x-w/2,-z-d/2,0,x+w/2,-z+d/2,h]))<.001
 report.append({'solid':[x,z,w,d,h],'bounds':[lo,hi]})
export(s,'arena-crates.blend','props')
# Original joined sandstone/planting scene, only five south-east border blocks altered.
s=load('smoko-surroundings.blend','Shelter entrance stone and dust');base=next(o for o in s.objects if o.type=='MESH');base.data=base.data.copy()
sand=[m for m in base.data.materials if m and m.name.startswith('Sandstone')];grass=[m for m in base.data.materials if m and m.name.startswith('Dry grass')]
layout=json.loads((ROOT/'art/bush-servo/surroundings-layout.json').read_text());inv=base.matrix_world.inverted();changed=0
stone_vertices={i for p in base.data.polygons if base.data.materials[p.material_index] in sand for i in p.vertices}
for x,z,w,d,h in layout['rocks'][:5]:
 group=[v for v in base.data.vertices if v.index in stone_vertices and abs((base.matrix_world@v.co).x-x)<=w/2+.001 and abs((base.matrix_world@v.co).y+z)<=d/2+.001 and -.001<=(base.matrix_world@v.co).z<=h+.001]
 assert group
 for v in group:
  p=base.matrix_world@v.co
  # Broad planes remain; tighten small corner chips inside the existing box.
  if abs(p.x-x)>w*.32 and abs(p.y+z)>d*.29:
   p.x=x+(p.x-x)*rng.uniform(.94,.99);p.y=-z+(p.y+z)*rng.uniform(.94,.99);changed+=1
  v.co=inv@p
 # Keep exact visible collider extents.
 for axis,(centre,dim) in enumerate([(x,w),(-z,d),(h/2,h)]):
  vals=[(base.matrix_world@v.co)[axis] for v in group];lo,hi=min(vals),max(vals)
  for v in group:
   p=base.matrix_world@v.co;p[axis]=(p[axis]-lo)/(hi-lo)*dim+centre-dim/2;v.co=inv@p
# Pebbles stay boot-low and outside the 4.5m-wide approach lane.
for i in range(30):
 x=rng.uniform(2.45,5.4);z=rng.uniform(16.7,17.25)
 bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=1,radius=1,location=(x,-z,.007));o=bpy.context.object;o.name='Entrance embedded grit';o.scale=(rng.uniform(.035,.075),rng.uniform(.03,.07),rng.uniform(.012,.025));o.data.materials.append(sand[i%len(sand)])
# Quiet dust wash over the dirt/road transition. One feathered vertex-colour mesh.
m=bpy.data.materials.new('Entrance dust feather');m.use_nodes=True
bs=next(n for n in m.node_tree.nodes if n.type=='BSDF_PRINCIPLED');bs.inputs['Base Color'].default_value=(.49,.29,.15,1);bs.inputs['Roughness'].default_value=1;bs.inputs['Alpha'].default_value=.99
m.surface_render_method='DITHERED'
verts=[];faces=[];colors=[]
for ring in range(6):
 radius=ring/5
 for i in range(48):
  a=i*math.tau/48;r=radius*(1+.05*math.sin(a*7));verts.append((r*2.6,-18.1+r*1.6,.014));colors.append((.49,.29,.15,.22*(1-radius)**2))
for ring in range(5):
 for i in range(48):j=(i+1)%48;faces.append((ring*48+i,ring*48+j,(ring+1)*48+j,(ring+1)*48+i))
mesh=bpy.data.meshes.new('Feathered dust transition');mesh.from_pydata(verts,[],faces);mesh.update();o=bpy.data.objects.new('Entrance worn dust wash',mesh);s.collection.objects.link(o);mesh.materials.append(m)
attr=mesh.color_attributes.new(name='Color',type='FLOAT_COLOR',domain='POINT')
for v,col in zip(attr.data,colors):v.color=col
node=m.node_tree.nodes.new('ShaderNodeVertexColor');node.layer_name='Color'
m.node_tree.links.new(node.outputs['Color'],bs.inputs['Base Color']);m.node_tree.links.new(node.outputs['Alpha'],bs.inputs['Alpha'])
export(s,'arena-entrance.blend','surroundings')
(ROOT/'art/bush-servo/arena-entrance-bounds.json').write_text(json.dumps({'crates':report,'chipped_vertices':changed,'stone_bounds_preserved':True,'gravel_max_height':.032,'dust_height':.014},indent=2))
print('Arena entrance and 3 crates exported')
