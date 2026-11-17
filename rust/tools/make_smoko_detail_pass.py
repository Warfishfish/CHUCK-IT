"""Original Smoko dressing, authored with Blender MCP. No downloaded assets.
Small details fit existing solid props; the two canopy additions stay above player height.
Exports one scene, joining meshes by material to limit runtime draws.
"""
import bpy, math, random
from pathlib import Path
from mathutils import Vector
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'crates/bbq_app/assets/models/smoko_detail_pass.glb'
rng=random.Random(171126)
old=bpy.context.window.scene
scene=bpy.data.scenes.new('Smoko detail pass 17 November')
bpy.context.window.scene=scene

def point(p):return (p[0],-p[2],p[1])
def mat(name,rgb,rough=.8,metal=0):
 m=bpy.data.materials.new('Smoko detail '+name);m.diffuse_color=(*rgb,1);m.use_nodes=True
 bs=m.node_tree.nodes.get('Principled BSDF');bs.inputs['Base Color'].default_value=(*rgb,1);bs.inputs['Roughness'].default_value=rough;bs.inputs['Metallic'].default_value=metal
 return m
wood=mat('sun faded timber',(.43,.29,.17));metal=mat('galvanized fittings',(.52,.57,.55),.48,.55)
cream=mat('enamel cream',(.83,.77,.60),.6);red=mat('tomato enamel',(.52,.11,.055),.65)
teal=mat('faded teal',(.16,.34,.31));dark=mat('rubber and bag',(.055,.068,.055));paper=mat('paper and rope',(.67,.57,.38))
leaf=[mat('gum leaf '+str(i),c) for i,c in enumerate([(.29,.37,.20),(.38,.45,.26),(.43,.49,.31)])]

def finish(o,name,m):o.name=name;o.data.materials.append(m);return o
def box(name,p,s,m,bevel=.01):
 bpy.ops.mesh.primitive_cube_add(size=1,location=point(p));o=bpy.context.object;o.scale=(s[0],s[2],s[1]);bpy.ops.object.transform_apply(location=False,rotation=False,scale=True)
 if bevel:
  b=o.modifiers.new('Worn edge','BEVEL');b.width=bevel;b.segments=2;bpy.ops.object.modifier_apply(modifier=b.name)
 return finish(o,name,m)
def cyl(name,p,r,h,m):
 bpy.ops.mesh.primitive_cylinder_add(vertices=12,radius=r,depth=h,location=point(p));return finish(bpy.context.object,name,m)
def sphere(name,p,s,m):
 bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=2,radius=1,location=point(p));o=bpy.context.object;o.scale=(s[0],s[2],s[1]);return finish(o,name,m)
def bar(name,a,b,r,m):
 aa,bb=Vector(point(a)),Vector(point(b));v=bb-aa
 bpy.ops.mesh.primitive_cylinder_add(vertices=8,radius=r,depth=v.length,location=(aa+bb)/2);o=bpy.context.object;o.rotation_mode='QUATERNION';o.rotation_quaternion=Vector((0,0,1)).rotation_difference(v.normalized());return finish(o,name,m)
# Picnic clutter and real hardware on the existing tables, all within their solid bounds.
for x in [-8.5,8.5]:
 for dx,dz in [(-.72,-.17),(.55,.17)]:
  cyl('Chipped enamel plate',(x+dx,.964,-2.5+dz),.14,.018,cream)
  cyl('Plate inner recess',(x+dx,.975,-2.5+dz),.105,.007,teal)
  cyl('Sun faded picnic cup',(x+dx+.21,1.015,-2.5+dz),.055,.13,cream)
 for dx in [-.30,.04]:
  cyl('Sauce bottle',(x+dx,1.065,-2.47),.055,.22,red if dx<0 else cream)
  cyl('Sauce cap',(x+dx,1.185,-2.47),.035,.035,dark)
 box('Folded snag napkins',(x+.9,.974,-2.42),(.23,.028,.22),paper,.005)
 for dx in [-1.02,1.02]:
  for dz in [-.28,.28]:cyl('Flush table bolt',(x+dx,.958,-2.5+dz),.022,.015,metal)
 for dx in [-.8,.8]:
  bar('Picnic brace',(x+dx,.18,-2.78),(x+dx,.86,-2.25),.025,metal)
# Existing bins gain separate lids, handles, reinforcing bands and a folded liner.
for x,z in [(-7.2,14.8),(4.64,13.5),(-12.5,4),(12.5,4),(27,-20)]:
 box('Bin folded lid',(x,1.055,z),(.65,.055,.65),teal,.025)
 box('Bin recessed handle',(x,1.092,z),(.18,.018,.075),dark,.008)
 for side in [-1,1]:box('Bin rim band',(x+side*.325,.94,z),(.022,.045,.61),metal,.005)
 box('Bin liner folded over rim',(x+.21,1.085,z+.13),(.19,.013,.20),dark,.006)
# Pump hose loops and nozzle hardware remain within existing pump bases/frontage.
for x in [1.6,4.8]:
 z=-16.65
 points=[(x+.25,1.46,z),(x+.34,1.15,z),(x+.32,.47,z),(x+.20,.40,z),(x+.18,1.20,z)]
 for a,b in zip(points,points[1:]):bar('Petrol hose',a,b,.025,dark)
 box('Pump nozzle grip',(x+.20,1.24,z),(.10,.20,.065),metal,.012)
 box('Pump nozzle trigger',(x+.20,1.24,z-.045),(.045,.08,.022),dark,.004)
# Shelter post shoes, bolts and join straps bring the hero shelter into the same material family.
for x in [-4,4]:
 for z in [6,14]:
  box('Galvanized post collar',(x,.11,z),(.27,.16,.27),metal,.012)
  for y in [.12,3.46]:
   for side in [-1,1]:sphere('Post bolt head',(x+side*.13,y,z),(.025,.025,.025),metal)
  box('Shelter joint strap',(x,3.54,z-.135),(.18,.34,.018),metal,.006)
# Irregular broad sprays supplement existing fine gum foliage, above sightlines.
for x,z in [(12,1.5),(-11.5,2)]:
 for i in range(14):
  a=i*2.39996;r=1.2+(i%4)*.48;y=5.0+(i%5)*.48
  px,pz=x+math.cos(a)*r,z+math.sin(a)*r
  sphere('Gum broad leaf spray',(px,y,pz),(1.0+rng.random()*.65,.32+rng.random()*.22,.70+rng.random()*.45),leaf[i%3])
  if i%3==0:bar('Gum crown fork',(x,4.0,z),(px,y-.14,pz),.045,wood)
# Enamel pump dials, brass bezels and a cream inspection stripe.
for x in [1.6,4.8]:
 for dx in [-.085,.085]:
  box('Dial bezel',(x+dx,1.48,-16.63),(.15,.19,.08),metal)
  box('Dial window',(x+dx,1.48,-16.58),(.12,.15,.025),cream)
  bar('Dial needle',(x+dx-.035,1.44,-16.56),(x+dx+.035,1.51,-16.56),.009,dark)
for x in [-8.5,8.5]:
 box('Lunch tin',(x-.20,1.035,-2.69),(.31,.14,.21),teal,.025)
 box('Tin fitted lid',(x-.20,1.115,-2.69),(.32,.025,.22),cream)
for x in [-4,4]:
 for z in [6,14]:
  bar('Shelter knee brace',(x,2.90,z),(x*.83,3.55,z),.075,wood)
# Merge by material, preserve a few material groups rather than hundreds of objects.
for m in [wood,metal,cream,red,teal,dark,paper]+leaf:
 obs=[o for o in scene.objects if o.type=='MESH' and o.data.materials and o.data.materials[0]==m]
 if not obs:continue
 bpy.ops.object.select_all(action='DESELECT')
 for o in obs:o.select_set(True)
 bpy.context.view_layer.objects.active=obs[0];bpy.ops.object.join();obs[0].name=m.name
bpy.ops.export_scene.gltf(filepath=str(OUT),export_format='GLB',use_active_scene=True,export_yup=True,export_cameras=False,export_lights=False)
tri=sum(len(o.data.polygons) for o in scene.objects if o.type=='MESH')
print('EXPORTED',OUT,'mesh groups',len(scene.objects),'polygons',tri)
bpy.context.window.scene=old
