"""Live Blender authoring script. Metres; origin at Smoko centre; Z-up export to glTF Y-up.
Only the named shelter collection is exported. Existing scene objects are preserved.
"""
import bpy, math, random
from pathlib import Path
from mathutils import Vector
ROOT=Path('/Users/marcus/Claude/australian-bbq')
random.seed(26)
scene=bpy.data.scenes.new('Bush Servo Shelter')
bpy.context.window.scene=scene
collection=bpy.data.collections.new('SMOKO Shelter');scene.collection.children.link(collection)
objects=[]
def mat(name,color,rough=.8,metal=0):
 m=bpy.data.materials.new(name);m.use_nodes=True
 n=next(n for n in m.node_tree.nodes if n.type=='BSDF_PRINCIPLED')
 n.inputs['Base Color'].default_value=(*color,1);n.inputs['Roughness'].default_value=rough;n.inputs['Metallic'].default_value=metal
 m.diffuse_color=(*color,1);return m
wood=[mat('Sun bleached timber '+str(i),(.24+i*.025,.13+i*.017,.065+i*.012)) for i in range(5)]
# Small baked grain tiles travel with the GLB, unlike Blender-only procedural shaders.
for k,m in enumerate(wood):
 image=bpy.data.images.new('Baked timber grain '+str(k),width=128,height=512)
 pixels=[]
 for v in range(512):
  for u in range(128):
   wave=math.sin(u*.72+math.sin(v*.024)*.7)+.4*math.sin(u*2.3+v*.006)
   value=1+.075*wave+random.uniform(-.025,.025)
   for c in (.53+k*.018,.39+k*.018,.25+k*.014):pixels.append(max(0,min(1,c*value)))
   pixels.append(1)
 image.pixels=pixels;image.pack()
 node=m.node_tree.nodes.new('ShaderNodeTexImage');node.image=image
 shader=next(n for n in m.node_tree.nodes if n.type=='BSDF_PRINCIPLED')
 m.node_tree.links.new(node.outputs['Color'],shader.inputs['Base Color'])
end=mat('Timber endgrain',(.33,.21,.115));dark=mat('Wood cracks',(.085,.048,.025));steel=mat('Old galvanized steel',(.43,.45,.41),.62,.45)
rust=[mat('Corrugated iron '+str(i),c,.78,.22) for i,c in enumerate([(.43,.19,.095),(.53,.29,.16),(.38,.36,.30),(.57,.51,.40),(.30,.31,.28),(.58,.27,.12)])]
bolt=mat('Dark iron fasteners',(.10,.105,.09),.55,.65);teal=mat('Weathered teal sign',(.07,.19,.17));cream=mat('Warm sign lettering',(.88,.77,.53))
def link(o):
 for c in list(o.users_collection):c.objects.unlink(o)
 collection.objects.link(o);objects.append(o);return o
def cube(name,loc,size,material,bevel=.02):
 bpy.ops.mesh.primitive_cube_add(size=1,location=loc);o=link(bpy.context.object);o.name=name;o.dimensions=size
 bpy.ops.object.transform_apply(location=False,rotation=False,scale=True)
 o.data.materials.append(material)
 if bevel:
  m=o.modifiers.new('Soft worn edges','BEVEL');m.width=bevel;m.segments=2
  bpy.context.view_layer.objects.active=o;bpy.ops.object.modifier_apply(modifier=m.name)
 return o
def beam(name,a,b,width,depth,material):
 a,b=Vector(a),Vector(b);o=cube(name,(a+b)/2,(width,depth,(b-a).length),material,.012);o.rotation_euler=(b-a).to_track_quat('Z','Y').to_euler();return o
def screw(loc):
 bpy.ops.mesh.primitive_uv_sphere_add(segments=8,ring_count=4,radius=.027,location=loc);o=link(bpy.context.object);o.name='Forged bolt';o.data.materials.append(bolt)
# Exact existing collision footprints (24cm square, 3.8m high).
for x in [-4,4]:
 for y in [-4,4]:
  cube('Structural post',(x,y,1.9),(.24,.24,3.8),random.choice(wood),.018)
  cube('Galvanized post shoe',(x,y,.12),(.245,.245,.24),steel,.008)
  for h in [.2,3.48]:screw((x,y-.127,h))
  for n in range(6):
   z=random.uniform(.4,3.3);xx=x+random.uniform(-.08,.08)
   cube('Fine split in timber',(xx,y-.121,z),(.004,.001,random.uniform(.12,.42)),dark,0)
# Beams, rafters and high diagonal braces: keep clear underneath.
for y in [-4,4]:cube('Eave bearer',(0,y,3.66),(8.5,.20,.24),wood[2])
for x in [-4,4]:cube('Side bearer',(x,0,3.66),(.20,8.5,.24),wood[1])
for x in [-4,-2,0,2,4]:
 for side in [-1,1]:beam('Roof rafter',(x,0,4.36),(x,side*4.5,3.82),.12,.16,wood[2])
for x in [-4,4]:
 for y in [-4,4]:
  beam('Corner knee brace',(x,y,2.95),(x-math.copysign(.65,x),y,3.56),.105,.105,wood[1])
# Actual wave-profile corrugated sheets, individually coloured with restrained rust patches.
# Eight strips per slope, 6 ribs per strip, 8 samples per rib; ~6k roof triangles.
for side in [-1,1]:
 for panel in range(8):
  x0=-4.65+panel*1.1625; vertices=[];faces=[]
  nx=48;ny=5
  for j in range(ny+1):
   t=j/ny; y=side*t*4.57
   for i in range(nx+1):
    x=x0+i/nx*1.1625; wave=.026*math.cos(i/nx*math.tau*6)
    vertices.append((x,y,4.40-.12*abs(y)+wave))
  for j in range(ny):
   for i in range(nx):
    a=j*(nx+1)+i;f=(a,a+1,a+nx+2,a+nx+1)
    faces.append(f if side>0 else tuple(reversed(f)))
  mesh=bpy.data.meshes.new('Rippled sheet');mesh.from_pydata(vertices,[],faces);mesh.update()
  o=bpy.data.objects.new('Roof sheet %s %s'%(side,panel),mesh);collection.objects.link(o);objects.append(o)
  for m in rust:mesh.materials.append(m)
  base=random.choice([0,1,2,3,4])
  for f in mesh.polygons:f.material_index=base if random.random()>.14 else random.choice([0,1,5]);f.use_smooth=True
  sol=o.modifiers.new('Iron thickness','SOLIDIFY');sol.thickness=.018
  bpy.context.view_layer.objects.active=o;o.select_set(True);bpy.ops.object.modifier_apply(modifier=sol.name);o.select_set(False)
  for xx in [x0+.18,x0+.96]:
   for yy in [side*.45,side*4.25]:screw((xx,yy,4.43-.12*abs(yy)))
cube('Ridge cap',(0,0,4.45),(9.4,.20,.07),steel,.025)
for y in [-4.58,4.58]:cube('Weathered fascia',(0,y,3.84),(9.4,.10,.16),wood[3])
# Large framed sign on BOTH approaches; lower edge remains above character heads.
for side in [-1,1]:
 y=side*4.70
 cube('SMOKO timber sign backing',(0,y,4.02),(4.25,.13,1.05),teal,.025)
 for x in [-2.16,2.16]:cube('Sign frame upright',(x,y,4.02),(.09,.19,1.17),wood[3],.015)
 for z in [3.46,4.58]:cube('Sign frame rail',(0,y,z),(4.4,.19,.09),wood[3],.012)
 # Text faces outward in Blender -Y/+Y, exported with the shelter.
 curve=bpy.data.curves.new('SMOKO lettering','FONT');curve.body='SMOKO';curve.align_x='CENTER';curve.size=.64;curve.font=bpy.data.fonts.load(str(ROOT/'rust/crates/bbq_app/assets/fonts/BowlbyOne-Regular.ttf'));curve.resolution_u=4;curve.extrude=0;curve.bevel_depth=0
 o=bpy.data.objects.new('SMOKO raised cream lettering',curve);collection.objects.link(o);objects.append(o)
 o.location=(0,y+side*.077,3.74);o.rotation_euler=(math.pi/2 if side<0 else math.pi/2,0,0 if side<0 else math.pi)
 curve.materials.append(cream)
 bpy.context.view_layer.objects.active=o;o.select_set(True);bpy.ops.object.convert(target='MESH');bpy.context.view_layer.update();o.scale *= 3.30 / o.dimensions.x;o.select_set(False)
# Rear benches occupy exactly existing solid cuboids. Planked top and inset supports.
for cx in [-2.3,2.3]:
 for j in range(3):cube('Bench seat plank',(cx,-5.2-.16+j*.16,.405),(3.2,.153,.11),wood[j+1],.018)
 for x in [cx-1.20,cx+1.20]:
  cube('Bench trestle foot',(x,-5.2,.15),(.18,.46,.30),wood[1],.015)
 beam('Bench stretcher',(cx-1.2,-5.2,.2),(cx+1.2,-5.2,.2),.10,.10,wood[2])
# Reusable glTF: join by material to keep draw calls low. Apply all geometry first.
bpy.ops.object.select_all(action='DESELECT')
meshes=[o for o in objects if o.type=='MESH']
for o in meshes:o.select_set(True)
bpy.context.view_layer.objects.active=meshes[0];bpy.ops.object.join();model=bpy.context.object;model.name='BushServo_SmokoShelter'
bpy.ops.object.transform_apply(location=False,rotation=False,scale=True)
# Export ONLY the asset, excluding preview lighting/camera.
asset=ROOT/'rust/crates/bbq_app/assets/models/smoko_shelter.glb'
bpy.ops.export_scene.gltf(filepath=str(asset),use_selection=True,use_active_scene=True,export_apply=True)
# Preview scene (does not affect exported model).
bpy.ops.object.light_add(type='AREA',location=(0,-3,10));light=bpy.context.object;light.data.energy=2000;light.data.shape='DISK';light.data.size=8
bpy.ops.object.camera_add(location=(13,-16,11));cam=bpy.context.object;cam.rotation_euler=(Vector((0,0,2))-cam.location).to_track_quat('-Z','Y').to_euler();cam.data.type='ORTHO';cam.data.ortho_scale=17;scene.camera=cam
scene.world=bpy.data.worlds.new('Warm preview world');scene.world.use_nodes=True
next(n for n in scene.world.node_tree.nodes if n.type=='BACKGROUND').inputs[0].default_value=(.3,.38,.48,1)
scene.render.resolution_x=1200;scene.render.resolution_y=900;scene.render.resolution_percentage=100
bpy.ops.wm.save_as_mainfile(filepath=str(ROOT/'art/bush-servo/smoko-shelter.blend'))
print('Exported',asset,'vertices',len(model.data.vertices),'polygons',len(model.data.polygons),'bytes',asset.stat().st_size)
