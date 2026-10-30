"""Stage 6 prop kit. New scene only; preserves other open Blender scenes.
Layout uses game x,z; Blender (x,-z,height). Shared solid bounds exported for verification.
"""
import bpy, math, json, random, struct, zlib
import numpy as np
from pathlib import Path
from mathutils import Vector
ROOT=Path('/Users/marcus/Claude/australian-bbq')
layout=json.loads((ROOT/'art/bush-servo/props-layout.json').read_text())
scene=bpy.data.scenes.new('Bush Servo Small Props')
bpy.context.window.scene=scene
objects=[];report=[];rng=random.Random(6028)
# Baked once, shared by every prop: restrained age rather than noisy damage.
WEAR_SIZE=256
DUST_AMOUNT=.10
SCUFF_AMOUNT=.13
def mat(name,c,rough=.8,metal=0):
 m=bpy.data.materials.new('Servo props '+name);m.use_nodes=True
 n=next(n for n in m.node_tree.nodes if n.type=='BSDF_PRINCIPLED')
 n.inputs['Base Color'].default_value=(*c,1);n.inputs['Roughness'].default_value=rough;n.inputs['Metallic'].default_value=metal
 m.diffuse_color=(*c,1);return m
blue=mat('faded blue plastic',(.055,.22,.38),.53)
teal=mat('green plastic',(.075,.23,.19),.57)
cream=mat('warm ivory',(.82,.78,.63),.65)
rubber=mat('tyre rubber',(.026,.032,.033),.94)
tread=mat('worn rubber edges',(.048,.052,.05),.91)
steel=mat('galvanised steel',(.29,.31,.29),.56,.45)
dark=mat('iron',(.055,.062,.056),.62,.35)
wood=[mat('timber '+str(i),(.30+i*.022,.19+i*.017,.10+i*.012)) for i in range(3)]
# Reuse already-authored shelter wood; no additional downloads or generators.
with bpy.data.libraries.load(str(ROOT/'art/bush-servo/smoko-shelter.blend'),link=False) as (src,dst):
 names=[n for n in src.materials if n in ['Sun bleached timber 2.003','Sun bleached timber 3.003','Sun bleached timber 4.003']]
 dst.materials=names
for i,m in enumerate(dst.materials):
 if m:wood[i]=m
def weather(material,seed):
 bs=next(n for n in material.node_tree.nodes if n.type=='BSDF_PRINCIPLED')
 c=np.array(bs.inputs['Base Color'].default_value[:3]);rr=np.random.default_rng(seed)
 yy,xx=np.mgrid[0:WEAR_SIZE,0:WEAR_SIZE]/(WEAR_SIZE-1)
 # Image rows run bottom-up in Blender; soft dirt collects toward object bases.
 dust=DUST_AMOUNT*(1-yy)**3
 variation=.018*np.sin(xx*math.tau*4+np.sin(yy*8))+.012*rr.uniform(-1,1,xx.shape)
 wear=np.zeros(xx.shape)
 for i in range(35):
  cx,cy=rr.uniform(.04,.96,2);length=rr.uniform(.012,.10)
  mark=np.exp(-((xx-cx)/length)**2-((yy-cy)/.0025)**2)
  wear=np.maximum(wear,mark*SCUFF_AMOUNT)
 colour=np.clip(c[None,None,:]*(1+variation[:,:,None])*(1-dust[:,:,None]-wear[:,:,None])+np.array([.38,.27,.16])*dust[:,:,None]+np.array([.55,.53,.43])*wear[:,:,None],0,1)
 srgb=np.where(colour<=.0031308,colour*12.92,1.055*np.power(colour,1/2.4)-.055)
 rgba=np.empty((WEAR_SIZE,WEAR_SIZE,4),dtype=np.uint8);rgba[:,:,:3]=(srgb*255+.5).astype(np.uint8);rgba[:,:,3]=255
 # PNG top row is Blender's highest v.
 raw=b''.join(b'\0'+row.tobytes() for row in rgba[::-1])
 def chunk(k,v):return struct.pack('>I',len(v))+k+v+struct.pack('>I',zlib.crc32(k+v)&0xffffffff)
 folder=ROOT/'art/bush-servo/prop-textures';folder.mkdir(exist_ok=True)
 path=folder/f'weather-{seed}.png';path.write_bytes(b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',WEAR_SIZE,WEAR_SIZE,8,6,0,0,0))+chunk(b'IDAT',zlib.compress(raw,8))+chunk(b'IEND',b''))
 im=bpy.data.images.load(str(path),check_existing=False);im.pack()
 node=material.node_tree.nodes.new('ShaderNodeTexImage');node.image=im
 material.node_tree.links.new(node.outputs['Color'],bs.inputs['Base Color'])
for i,m in enumerate([blue,teal,cream,rubber,steel]):weather(m,3010+i)

def box(name,x,z,y,w,d,h,m,bevel=.025):
 bpy.ops.mesh.primitive_cube_add(size=1,location=(x,-z,y));o=bpy.context.object;o.name=name;o.dimensions=(w,d,h)
 bpy.ops.object.transform_apply(location=False,rotation=False,scale=True);o.data.materials.append(m)
 if bevel:
  b=o.modifiers.new('Rounded edges','BEVEL');b.width=min(bevel,min(w,d,h)*.3);b.segments=2;bpy.ops.object.modifier_apply(modifier=b.name)
 # Project each face using its two dominant axes, with vertical sides bottom-up.
 uv=o.data.uv_layers.active or o.data.uv_layers.new(name='Prop surface')
 for f in o.data.polygons:
  axis=max(range(3),key=lambda k:abs(f.normal[k]))
  for li in f.loop_indices:
   v=o.data.vertices[o.data.loops[li].vertex_index].co
   uv.data[li].uv=((v.y/d+.5,v.z/h+.5) if axis==0 else ((v.x/w+.5,v.z/h+.5) if axis==1 else (v.x/w+.5,v.y/d+.5)))
 objects.append(o);return o
def cylinder(name,x,z,y,r,depth,m):
 bpy.ops.mesh.primitive_cylinder_add(vertices=20,radius=r,depth=depth,location=(x,-z,y));o=bpy.context.object;o.name=name;o.data.materials.append(m);objects.append(o);return o
def torus(name,x,z,y,major,minor,m,rotation=None):
 bpy.ops.mesh.primitive_torus_add(major_segments=24,minor_segments=10,major_radius=major,minor_radius=minor,location=(x,-z,y))
 o=bpy.context.object;o.name=name;o.data.materials.append(m)
 if rotation:o.rotation_euler=rotation
 for p in o.data.polygons:p.use_smooth=True
 objects.append(o);return o
def record(name,start,expected):
 coords=[o.matrix_world@Vector(v) for o in objects[start:] for v in o.bound_box]
 # Ensure transforms have evaluated before comparing dimensions.
 bpy.context.view_layer.update()
 coords=[o.matrix_world@Vector(v) for o in objects[start:] for v in o.bound_box]
 lo=[min(v[k] for v in coords) for k in range(3)];hi=[max(v[k] for v in coords) for k in range(3)]
 x,z,w,d,h=expected
 assert max(abs(a-b) for a,b in zip([lo[0],hi[0],-hi[1],-lo[1],hi[2]],[x-w/2,x+w/2,z-d/2,z+d/2,h]))<.025,(name,lo,hi,expected)
 report.append({'name':name,'solid':expected,'visual_min':lo,'visual_max':hi})
for i,(x,z,w,d,h) in enumerate(layout['eskies']):
 start=len(objects);colour=teal if i==0 else blue
 box('Esky insulated body',x,z,(h-.09)/2,w-.10,d-.04,h-.09,colour,.065)
 box('Esky lid seal',x,z,h-.105,w-.06,d-.02,.025,dark,.008)
 box('Esky rounded lid',x,z,h-.045,w,d,.09,cream,.035)
 for sx in [-1,1]:
  box('Recessed carry grip',x+sx*(w/2-.027),z,h*.50,.054,.28,.12,cream,.02)
 for dx in [-.30,.30]:box('Latch',x+dx,z-d/2+.025,h-.11,.08,.05,.14,steel,.012)
 box('Lid inset',x,z,h-.004,w*.65,d*.60,.008,cream,.02)
 record('esky'+str(i),start,[x,z,w,d,h])
for i,(x,z,w,d,h) in enumerate(layout['bins']):
 start=len(objects);colour=teal if i%2==0 else steel
 box('Bin body',x,z,(h-.09)/2,w-.05,d-.05,h-.09,colour,.045)
 box('Bin rolled lid',x,z,h-.045,w,d,.09,colour,.025)
 box('Dark front inset',x,z-d/2+.02,h*.64,w*.62,.025,h*.24,dark,.025)
 # Flush handle and restrained pressed-metal seams, within the shared bounding box.
 box('Bin handle',x,z-d/2+.018,h*.83,.22,.036,.055,steel,.012)
 for dx in [-.22,.22]:box('Bin rib',x+dx,z-d/2+.02,h*.31,.025,.04,h*.46,colour,.005)
 record('bin'+str(i),start,[x,z,w,d,h])
for i,(x,z,w,d,h) in enumerate(layout['tyres']):
 start=len(objects)
 for j in range(round(h/.36)):
  yy=.18+j*.36
  torus('Tyre sidewall',x,z,yy,.42,.18,rubber)
  # Narrow sidewall rings and shallow diagonal tread blocks remain within 1.2m bounds.
  torus('Raised tyre lettering ring',x,z,yy+.14,.42,.018,tread)
  for k in range(24):
   a=k*math.tau/24+j*.15
   o=box('Tread',x+.574*math.cos(a),z+.574*math.sin(a),yy,.035,.09,.20,tread,0)
   o.rotation_euler.z=-a
 record('tyre'+str(i),start,[x,z,w,d,h])
for i,(x,z,w,d,h) in enumerate(layout['tables']):
 start=len(objects)
 for j in range(5):box('Picnic top plank',x,z-d/2+(j+.5)*d/5,h-.045,w,d/5-.012,.09,wood[j%3],.018)
 # End caps keep the full top width/depth explicit for collision matching.
 for zz in [z-d/2+.012,z+d/2-.012]:box('Table rim',x,zz,h-.045,w,.024,.09,wood[1],.005)
 for dx in [-.83,.83]:
  for dz in [-.28,.28]:box('Picnic leg',x+dx,z+dz,(h-.09)/2,.11,.12,h-.09,wood[0],.012)
  box('Cross brace',x+dx,z,.35,.10,.72,.10,wood[1],.01)
 record('table'+str(i),start,[x,z,w,d,h])
 for side in [-1,1]:
  zz=z+side*.95;start=len(objects)
  box('Bench seat',x,zz,.435,w,.35,.09,wood[1],.02)
  for dx in [-.83,.83]:box('Bench leg',x+dx,zz,.195,.14,.24,.39,wood[0],.012)
  record('bench'+str(i)+'_'+str(side),start,[x,zz,w,.35,.48])
 # Small decorative enamel mugs: no glowing pickup plates, hollow silhouettes.
 for dx,dz in [(-.55,-.1),(.55,.08)]:
  cx=x+dx;cz=z+dz
  cylinder('Enamel mug',cx,cz,h+.09,.075,.18,cream)
  cylinder('Tea surface',cx,cz,h+.183,.061,.004,dark)
  torus('Mug lip',cx,cz,h+.18,.068,.009,steel)
  torus('Mug handle',cx+.095,cz,h+.1,.042,.011,cream,(math.pi/2,0,0))
 box('Napkin tray',x,z+.1,h+.018,.33,.19,.036,steel,.015)
# Concept-inspired picnic clutter. Floor props fit inside the existing table solid.
decorative_floor_bounds=[]
for i,(x,z,w,d,h) in enumerate(layout['tables']):
 floor_start=len(objects)
 cx=x-.30;cz=z
 box('Supply crate interior',cx,cz,.15,.57,.43,.25,wood[0],.015)
 for level in [.08,.19,.30]:
  for side in [-1,1]:
   box('Crate side slat',cx,cz+side*.25,level,.65,.04,.075,wood[1],.008)
   box('Crate end slat',cx+side*.305,cz,level,.04,.46,.075,wood[2],.008)
 for dx in [-.29,.29]:
  for dz in [-.23,.23]:box('Crate corner',cx+dx,cz+dz,.17,.055,.055,.34,wood[0],.005)
 # Hollow galvanised bucket and raised wire handle under the table.
 bx=x+.48;bz=z
 cylinder('Bucket shell',bx,bz,.16,.16,.30,steel)
 cylinder('Bucket dark opening',bx,bz,.312,.144,.004,dark)
 torus('Bucket rolled lip',bx,bz,.315,.15,.012,steel)
 torus('Bucket wire handle',bx,bz,.36,.16,.006,steel,(math.pi/2,0,0))
 bpy.context.view_layer.update()
 coords=[o.matrix_world@Vector(v) for o in objects[floor_start:] for v in o.bound_box]
 lo=[min(v[k] for v in coords) for k in range(3)];hi=[max(v[k] for v in coords) for k in range(3)]
 assert lo[0]>=x-w/2 and hi[0]<=x+w/2 and lo[1]>=-z-d/2 and hi[1]<=-z+d/2 and lo[2]>=-.001 and hi[2]<h
 decorative_floor_bounds.append({'table':i,'min':lo,'max':hi,'within_existing_solid':True})
 # Thermos, enamel side plate and an extra mug form a readable tabletop grouping.
 tx=x-.10;tz=z-.20
 cylinder('Smoko thermos',tx,tz,h+.18,.085,.36,teal)
 cylinder('Thermos collar',tx,tz,h+.35,.087,.025,steel)
 cylinder('Thermos cup lid',tx,tz,h+.395,.09,.075,cream)
 cylinder('Enamel plate',x+.18,z+.12,h+.012,.145,.024,cream)
 torus('Plate rim',x+.18,z+.12,h+.026,.135,.01,steel)
 cylinder('Extra enamel mug',x-.85,z+.18,h+.085,.07,.17,cream)
 cylinder('Extra mug tea',x-.85,z+.18,h+.173,.055,.004,dark)
 torus('Extra mug handle',x-.76,z+.18,h+.09,.04,.01,cream,(math.pi/2,0,0))
 # Countersunk fasteners and quiet worn edges, still within the tabletop bounds.
 for dx in [-.80,.80]:
  for dz in [-.30,.30]:cylinder('Table fastener',x+dx,z+dz,h+.001,.012,.002,dark)

for i,(x,z) in enumerate(layout['signs']):
 box('Roadside sign post',x,z,1.15,.16,.16,2.3,wood[0],.02)
 box('Roadside sign board',x,z,1.85,1.8,.10,.85,teal,.035)
 for text,y in [('GOOD FOOD',2.02),('GOOD MATES',1.70)]:
  curve=bpy.data.curves.new(text,'FONT');curve.body=text;curve.align_x='CENTER';curve.size=.20;curve.extrude=.001
  o=bpy.data.objects.new(text,curve);scene.collection.objects.link(o);o.location=(x,-z+(.061 if i==0 else -.061),y);o.rotation_euler=(math.pi/2,0,math.pi if i==0 else 0);o.data.materials.append(cream)
  bpy.context.view_layer.objects.active=o;o.select_set(True);bpy.ops.object.convert(target='MESH');objects.append(bpy.context.object);o.select_set(False)
# Preserve individual editable objects in the authoring file, export a joined temporary copy.
bpy.ops.object.select_all(action='DESELECT')
for o in objects:o.select_set(True)
bpy.context.view_layer.objects.active=objects[0]
bpy.data.libraries.write(str(ROOT/'art/bush-servo/smoko-props.blend'), {scene}, fake_user=True, compress=True)
bpy.ops.object.duplicate();bpy.ops.object.join();asset=bpy.context.object;asset.name='Smoko Small Props Runtime'
bpy.ops.export_scene.gltf(filepath=str(ROOT/'rust/crates/bbq_app/assets/models/smoko_props.glb'),use_selection=True,use_active_scene=True,export_apply=True)
import runpy
print(runpy.run_path(str(ROOT/'art/bush-servo/compact_glb.py'))['compact'](ROOT/'rust/crates/bbq_app/assets/models/smoko_props.glb'))
(ROOT/'art/bush-servo/props-bounds.json').write_text(json.dumps(report,indent=2)+'\n')
(ROOT/'art/bush-servo/prop-decoration-bounds.json').write_text(json.dumps(decorative_floor_bounds,indent=2)+'\n')
print({'solids':len(report),'objects':len(objects),'polygons':len(asset.data.polygons),'materials':len(asset.data.materials)})
