"""Original beta nature library, authored via Blender MCP. No downloaded assets.
Three tree silhouettes plus a snagged kite and a roadside windsock. Meshes merged by material.
"""
import bpy, math, random
from mathutils import Vector
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'crates/bbq_app/assets/models'
old=bpy.context.window.scene
rng=random.Random(171117)
def pos(p):return (p[0],-p[2],p[1])
def material(name,c,rough=.85):
 m=bpy.data.materials.new(name);m.diffuse_color=(*c,1);m.use_nodes=True
 b=m.node_tree.nodes.get('Principled BSDF');b.inputs['Base Color'].default_value=(*c,1);b.inputs['Roughness'].default_value=rough
 return m
bark=material('Beta pale peeling bark',(.59,.53,.41));scar=material('Beta dark bark ridges',(.23,.21,.16))
leaves=material('Canopy soft sage foliage',(.29,.39,.25));fruit=material('Beta lemons',(.84,.65,.12))
cloth=material('Breeze sun faded teal',(.13,.42,.46));orange=material('Breeze ochre fabric',(.82,.32,.12));cream=material('Beta warm canvas',(.84,.79,.64))
rope=material('Beta rope',(.35,.29,.18))
def finish(o,name,m):o.name=name;o.data.materials.append(m);return o
def branch(a,b,r0,r1,m,name='Forked limb'):
 aa,bb=Vector(pos(a)),Vector(pos(b));d=bb-aa
 bpy.ops.mesh.primitive_cone_add(vertices=9,radius1=r0,radius2=r1,depth=d.length,location=(aa+bb)/2);o=bpy.context.object
 o.rotation_mode='QUATERNION';o.rotation_quaternion=Vector((0,0,1)).rotation_difference(d.normalized())
 for f in o.data.polygons:f.use_smooth=True
 return finish(o,name,m)
def crown(p,s,index):
 bpy.ops.mesh.primitive_uv_sphere_add(segments=12,ring_count=8,radius=1,location=pos(p));o=bpy.context.object
 for v in o.data.vertices:
  q=v.co;variation=1+.06*math.sin(q.x*7+q.y*4+index)+.035*math.sin(q.z*9-index)
  q*=variation
 o.scale=(s[0],s[2],s[1]);o.rotation_euler[2]=index*1.37
 for f in o.data.polygons:f.use_smooth=True
 return finish(o,'Canopy leaf spray',leaves)
def sphere(p,r,m,name):
 bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=2,radius=r,location=pos(p));return finish(bpy.context.object,name,m)
def scene(name):
 s=bpy.data.scenes.new(name);bpy.context.window.scene=s;return s
def export(s,name):
 for m in {o.data.materials[0] for o in s.objects if o.type=='MESH' and o.data.materials}:
  obs=[o for o in s.objects if o.type=='MESH' and o.data.materials and o.data.materials[0]==m]
  bpy.ops.object.select_all(action='DESELECT')
  for o in obs:o.select_set(True)
  bpy.context.view_layer.objects.active=obs[0];bpy.ops.object.join();obs[0].name=m.name
 bpy.ops.export_scene.gltf(filepath=str(OUT/name),export_format='GLB',use_active_scene=True,export_cameras=False,export_lights=False)
 print('EXPORTED',name,'groups',len(s.objects),'faces',sum(len(o.data.polygons) for o in s.objects if o.type=='MESH'))
for variant,height in [('gum',8.1),('citrus',5.5),('paperbark',9.2)]:
 s=scene('Beta nature '+variant);lean=.48 if variant!='citrus' else .16
 root=(0,0,0);mid=(lean,height*.36,-.12);fork=(lean*.7,height*.57,.2)
 branch(root,mid,.26,.19,bark);branch(mid,fork,.19,.12,bark)
 for k in range(5):
  a=k*math.tau/5;branch((0,.12,0),(math.cos(a)*.85,.025,math.sin(a)*.85),.13,.015,scar,'Root flare')
 count=6 if variant=='citrus' else 5
 for k in range(count):
  a=k*2.39996;spread=1.4 if variant=='paperbark' else 2.1
  if variant=='gum':spread*=.74+(k%3)*.34
  tip=(math.cos(a)*spread,height*(.70+(k%3)*.085),math.sin(a)*spread)
  bend=((fork[0]+tip[0])*.5,height*.65,(fork[2]+tip[2])*.5)
  branch(fork,bend,.11,.065,bark);branch(bend,tip,.065,.025,bark)
  for j in range(5):
   aa=j*2.4+k;rr=.52+(j%3)*.36
   p=(tip[0]+math.cos(aa)*rr,tip[1]+rng.uniform(-.42,.46),tip[2]+math.sin(aa)*rr)
   scale=(.86+rng.random()*.35,.58+rng.random()*.30,.72+rng.random()*.33)
   if variant=='gum':scale=(scale[0]*1.35,scale[1]*.68,scale[2])
   if variant=='citrus':scale=(1.0+.17*math.sin(aa),.80+.12*math.cos(aa),.95)
   crown(p,scale,k*5+j)
   if variant=='citrus' and j%2==0:sphere((p[0],p[1]-.65,p[2]),.11,fruit,'Ripe lemon')
  branch(bend,(tip[0]+.5,tip[1]+.6,tip[2]-.3),.022,.006,scar,'Bare gum twig')
 for i in range(9):
  y=.45+i*.35;branch((.22,y,.01),(.24,y+.26,-.015),.018,.008,scar,'Peeling bark scar')
 export(s,'beta_tree_'+variant+'.glb')
s=scene('Beta snagged kite')
vertices=[pos(p) for p in [(0,.8,0),(-.58,0,0),(0,-.95,0),(.58,0,0),(0,0,.10)]]
me=bpy.data.meshes.new('Sewn kite panels');me.from_pydata(vertices,[],[(0,1,4),(1,2,4),(2,3,4),(3,0,4)]);me.materials.append(cloth);me.materials.append(orange)
o=bpy.data.objects.new('Breeze kite',me);s.collection.objects.link(o)
for i,p in enumerate(me.polygons):p.material_index=i%2
for m in [cloth,orange]:m.surface_render_method='DITHERED';m.use_backface_culling=False
last=(0,-.95,0)
for i in range(7):
 next=(math.sin(i*.9)*.26,-1.1-i*.34,.08*i);branch(last,next,.009,.009,rope,'Breeze kite tail');last=next
 if i%2==0:
  branch((next[0]-.13,next[1],next[2]),(next[0]+.13,next[1]-.05,next[2]),.045,.025,cream,'Breeze tail bow')
export(s,'beta_kite.glb')
s=scene('Beta servo windsock')
branch((0,0,0),(0,5.2,0),.065,.045,scar,'Windsock pole')
for i in range(6):
 a=(.15+i*.28,4.9-i*.025,0);b=(.15+(i+1)*.28,4.9-(i+1)*.025,0)
 branch(a,b,.25-i*.029,.25-(i+1)*.029,orange if i%2==0 else cream,'Breeze windsock')
export(s,'beta_windsock.glb')
bpy.context.window.scene=old
