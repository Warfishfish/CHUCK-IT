"""Stage 4 servo. Run in Blender through MCP; preserves existing scenes.
Metres, Blender Z up; game (x,y,z) = (Blender x,z,-y). Geometry stays north
of the arena fence (game z=-24). Sources/licences: assets/CREDITS.txt.
Edit these named values before rebuilding; no gameplay or lighting is embedded.
"""
import bpy, math, random, json, tempfile
import numpy as np
from pathlib import Path
from mathutils import Vector, Matrix
ROOT=Path('/Users/marcus/Claude/australian-bbq')
FACADE_X=-8.; FRONT_Y=25.5; WIDTH=23.; DEPTH=7.
TEXTURE_LIMIT=1024; TIMBER_TILE_METRES=3.2; NORMAL_STRENGTH=.16
random.seed(27)
scene=bpy.data.scenes.new('Bush Servo Stage 4');bpy.context.window.scene=scene
col=bpy.data.collections.new('Servo building and forecourt');scene.collection.children.link(col)
objects=[]
# Load the packed, licensed sources. No network or addon credentials required to rebuild.
with bpy.data.libraries.load(str(ROOT/'art/bush-servo/library/servo-library.blend'),link=False) as (src,dst):dst.objects=src.objects
source=[o for o in dst.objects if o]
for o in source:scene.collection.objects.link(o)
bpy.context.view_layer.update()
board=next(o for o in source if o.name.startswith('Material staging board'))
timber=board.data.materials[0].copy();timber.name='Servo pale weathered timber (Poly Haven)';timber.use_backface_culling=True
# glTF uses UVs; do not depend on unsupported Blender-only displacement.
bs=next(n for n in timber.node_tree.nodes if n.type=='BSDF_PRINCIPLED')
for n in timber.node_tree.nodes:
 if n.type=='NORMAL_MAP':n.inputs['Strength'].default_value=NORMAL_STRENGTH
 if n.type=='MAPPING':n.inputs['Scale'].default_value=(1,1,1)
for l in list(timber.node_tree.links):
 if l.to_node.type=='OUTPUT_MATERIAL' and l.to_socket.name=='Displacement':timber.node_tree.links.remove(l)
# Keep only maps used by runtime; scale copies so the preserved library remains lossless.
image_copies={}
def compact_images(material):
 for n in material.node_tree.nodes:
  if n.type!='TEX_IMAGE' or not n.image:continue
  im=n.image
  if im not in image_copies:
   cp=im.copy();size=512 if 'Emission' in im.name or 'Alpha' in im.name or 'distressed' in im.name else TEXTURE_LIMIT
   if max(cp.size)>size:cp.scale(size,size)
   if 'distressed' in im.name and 'Diffuse' in im.name:
    # A light cream paint glaze keeps scanned wear subordinate to the cartoon forms.
    rgba=np.empty(len(cp.pixels),dtype=np.float32);cp.pixels.foreach_get(rgba);rgba=rgba.reshape(-1,4)
    rgba[:,:3]=rgba[:,:3]*.62+np.array([.72,.65,.48],dtype=np.float32)*.38
    cp.pixels.foreach_set(rgba.ravel());cp.update()
   normals=[l.to_node for l in n.outputs['Color'].links if l.to_node.type=='NORMAL_MAP']
   if normals:
    # Bevy's current glTF loader ignores normalTexture.scale. Bake its strength
    # into tangent-space pixels so Blender and the game use the same calm detail.
    strength=normals[0].inputs['Strength'].default_value
    rgba=np.empty(len(cp.pixels),dtype=np.float32);cp.pixels.foreach_get(rgba);rgba=rgba.reshape(-1,4)
    v=rgba[:,:3]*2-1;v[:,:2]*=strength;v/=np.maximum(np.linalg.norm(v,axis=1,keepdims=True),1e-6)
    rgba[:,:3]=v*.5+.5;cp.pixels.foreach_set(rgba.ravel());cp.update()
    for normal in normals:normal.inputs['Strength'].default_value=1
   # Runtime Bevy image support is PNG; glTF AUTO follows this format.
   cp.file_format='PNG'
   png=Path(tempfile.gettempdir())/('servo-runtime-'+cp.name.replace('/','_')+'.png')
   cp.filepath_raw=str(png);cp.save()
   runtime=bpy.data.images.load(str(png),check_existing=False);runtime.colorspace_settings.name=cp.colorspace_settings.name;runtime.pack()
   image_copies[im]=runtime
  n.image=image_copies[im]
compact_images(timber)
def mat(name,rgb,rough=.76,metal=0):
 m=bpy.data.materials.new(name);m.use_nodes=True;m.use_backface_culling=True;m.diffuse_color=(*rgb,1)
 n=next(n for n in m.node_tree.nodes if n.type=='BSDF_PRINCIPLED');n.inputs['Base Color'].default_value=(*rgb,1);n.inputs['Roughness'].default_value=rough;n.inputs['Metallic'].default_value=metal
 return m
cream=mat('Servo warm cream paint',(.73,.61,.40));red=mat('Servo faded red enamel',(.42,.065,.038),.57)
white=mat('Servo ivory trim',(.88,.81,.63),.6);dark=mat('Servo recess and rubber',(.035,.047,.044),.86)
glass=mat('Servo shaded blue glass',(.075,.16,.18),.3,.18);steel=mat('Servo zinc trim',(.33,.36,.34),.55,.5)
wood=mat('Servo porch timber',(.27,.145,.075));concrete=mat('Servo sandstone concrete',(.49,.38,.25))
teal=mat('Servo sign green',(.055,.16,.135));ute=mat('Servo sandy ute enamel',(.47,.40,.27),.52,.15)
roofs=[mat('Servo roof sheet '+str(i),c,.68,.3) for i,c in enumerate([(.38,.36,.30),(.46,.42,.34),(.42,.24,.15)])]
font=bpy.data.fonts.load(str(ROOT/'rust/crates/bbq_app/assets/fonts/BowlbyOne-Regular.ttf'))
def link(o):
 for c in list(o.users_collection):c.objects.unlink(o)
 col.objects.link(o);objects.append(o);return o
def uv_metres(o):
 # Axis-projected UVs, tunable broad tile. Front faces keep the planks upright.
 layer=o.data.uv_layers.active or o.data.uv_layers.new()
 for p in o.data.polygons:
  axis=max(range(3),key=lambda a:abs(p.normal[a]))
  for li in p.loop_indices:
   v=o.data.vertices[o.data.loops[li].vertex_index].co
   a,b=((v.y,v.z) if axis==0 else ((v.x,v.z) if axis==1 else (v.x,v.y)))
   layer.data[li].uv=(a/TIMBER_TILE_METRES,b/TIMBER_TILE_METRES)
def cube(name,loc,size,m,bev=.025):
 bpy.ops.mesh.primitive_cube_add(size=1,location=loc);o=link(bpy.context.object);o.name=name;o.dimensions=size
 bpy.ops.object.transform_apply(location=False,rotation=False,scale=True);uv_metres(o);o.data.materials.append(m)
 if bev:
  mod=o.modifiers.new('Soft edges','BEVEL');mod.width=bev;mod.segments=2;bpy.ops.object.modifier_apply(modifier=mod.name)
 return o
def rod(name,a,b,r,m,vertices=12):
 a,b=Vector(a),Vector(b);bpy.ops.mesh.primitive_cylinder_add(vertices=vertices,radius=r,depth=(b-a).length,location=(a+b)/2)
 o=link(bpy.context.object);o.name=name;o.rotation_euler=(b-a).to_track_quat('Z','Y').to_euler();o.data.materials.append(m);return o
def text(name,words,loc,width,m):
 cu=bpy.data.curves.new(name,'FONT');cu.body=words;cu.align_x='CENTER';cu.font=font;cu.size=1;cu.resolution_u=3;cu.extrude=.003
 o=bpy.data.objects.new(name,cu);col.objects.link(o);objects.append(o);o.location=loc;o.rotation_euler=(math.pi/2,0,0);cu.materials.append(m)
 bpy.ops.object.select_all(action='DESELECT');o.select_set(True);bpy.context.view_layer.objects.active=o;bpy.ops.object.convert(target='MESH');bpy.context.view_layer.update();o.scale*=width/max(o.dimensions.x,.001);return o
def roof(name,x0,x1,y0,y1,z0,z1):
 # Real corrugations: 4 samples/rib, broad material variation by sheet, not pixel noise.
 nx=round((x1-x0)/.21)*4;vs=[];fs=[]
 for j in range(2):
  for i in range(nx+1):vs.append((x0+(x1-x0)*i/nx,y0+(y1-y0)*j,z0+(z1-z0)*j+.022*math.cos(i*math.pi/2)))
 for i in range(nx):fs.append((i,i+1,nx+2+i,nx+1+i))
 me=bpy.data.meshes.new(name);me.from_pydata(vs,[],fs);me.update();o=bpy.data.objects.new(name,me);col.objects.link(o);objects.append(o)
 for m in roofs:m.use_backface_culling=False;me.materials.append(m)
 for p in me.polygons:p.material_index=[0,0,1,0,1,0,2,0,1][(p.index//20)%9];p.use_smooth=True
 return o
# Same building envelope as the original. A recessed dark shop interior improves depth.
cube('Weatherboard main building',(-8,29,2.25),(23,7,4.5),timber,.045)
cube('Concrete veranda slab',(-8,25.0,.12),(24.1,1.85,.24),concrete,.06)
# Low weatherboard courses and corners break up broad planes without dense texture noise.
for z in [.42,.7,1.0]:cube('Weatherboard lower course',(-8,25.465,z),(23,.045,.045),cream,.008)
for x in [-19.46,3.46]:cube('Corner trim',(x,25.4,2.26),(.15,.13,4.5),white)
# Recessed shop windows, shelves, thick sills and small pane reflections.
for x,w in [(-15.8,5.1),(-9.55,5.1),(.15,5.25)]:
 cube('Deep shop opening',(x,25.39,2.25),(w,.22,2.38),dark)
 cube('Shop glazing',(x,25.25,2.30),(w-.18,.025,2.12),glass,0)
 for xx in [x-w/2,x,x+w/2]:cube('Window mullion',(xx,25.16,2.28),(.105,.16,2.44),white,.012)
 for z in [1.07,2.88,3.49]:cube('Window frame',(x,25.14,z),(w+.15,.19,.10),cream,.012)
 cube('Window sill',(x,25.05,1.02),(w+.26,.38,.13),white)
 # Slender, muted reflected stripe: readable glazing without transparent sorting.
 for xx in [x-w*.32,x+w*.18]:cube('Glass reflection',(xx,25.225,2.10),(.035,.012,1.35),steel,0)
 cube('Window kickboard',(x,25.36,.57),(w,.12,.8),timber)
# Central entrance; decorative beyond fence, never looks like a traversable doorway.
cube('Entry jamb',(-5.0,25.32,1.65),(2.5,.28,3.22),white)
cube('Red shop door',(-5.,25.13,1.62),(2.2,.11,3.03),red)
cube('Door glass',(-5.,25.06,2.12),(1.86,.04,1.74),glass)
for xx in [-5.98,-4.02]:cube('Door vertical trim',(xx,25.015,1.7),(.065,.065,2.85),white,.008)
rod('Door pull',(-4.28,24.97,1.30),(-4.28,24.97,1.67),.027,steel)
text('Door open sign','OPEN',(-5,24.995,2.35),1.1,white)
# Roof and deeply shaded veranda, posts completely outside the arena.
roof('Main front corrugated roof',-20.15,4.15,24.95,29.,4.63,5.17)
roof('Main rear corrugated roof',-20.15,4.15,29.,33.,5.17,4.63)
cube('Roof ridge',(-8,29,5.20),(24.5,.20,.07),steel)
roof('Veranda corrugations',-20.,4.,24.12,25.8,3.62,3.92)
cube('Red veranda fascia',(-8,24.10,3.52),(24.1,.16,.33),red)
cube('Ivory fascia lip',(-8,24.12,3.72),(24.2,.18,.07),white,.012)
for x in [-19.2,-12.4,-5.7,3.2]:
 cube('Veranda timber post',(x,24.25,1.76),(.17,.17,3.52),cream)
 cube('Post dark shoe',(x,24.25,.18),(.19,.19,.36),steel,.01)
 rod('Veranda knee brace',(x,24.25,3.04),(x+.43,24.25,3.5),.055,wood,8)
for x in [-19.7,3.7]:
 rod('Downpipe',(x,24.25,.22),(x,24.25,3.66),.065,steel)
 rod('Gutter outlet',(x,24.25,3.66),(x,24.08,3.66),.065,steel)
# Broad raised sign above the awning; warm, legible concept lettering.
cube('Main sign cream frame',(-8,25.12,4.40),(19.8,.20,1.05),white)
cube('Main sign red enamel',(-8,24.995,4.40),(19.55,.06,.86),red,.012)
text('Main BUSH SERVO lettering','BUSH SERVO',(-8,24.95,4.04),8.4,white)
text('Shop snacks lettering','COLD DRINKS',(-15.9,25.10,2.99),3.1,white)
text('Shop food lettering','FOOD & SUPPLIES',(.05,25.10,2.99),3.5,white)
# Roof ventilator and chimney.
cube('Roof vent base',(-16,29.7,5.18),(1.0,.8,.38),steel)
cube('Roof vent lid',(-16,29.7,5.42),(1.2,1.,.08),steel)
# Tall roadside pylon. One clear silhouette, restrained signage.
for x in [-23.3,-21.4]:rod('Pylon leg',(x,27.9,0),(x,27.9,7.2),.075,steel)
cube('Tall pylon surround',(-22.35,27.9,5.55),(2.8,.23,3.5),red)
for z,word in [(6.55,'FUEL'),(5.85,'FOOD'),(5.15,'DRINKS'),(4.45,'TYRES')]:
 cube('Pylon ivory panel',(-22.35,27.74,z),(2.55,.1,.64),cream)
 text('Pylon '+word,word,(-22.35,27.67,z-.19),2.12,red)
# Ice freezer, readable at gameplay scale, plus lid hinges and handles.
cube('ICE freezer body',(-18.,24.85,.77),(1.9,1.05,1.25),white,.07)
cube('ICE freezer base',(-18.,24.85,.16),(1.78,.98,.15),dark)
cube('ICE freezer lid',(-18.,24.85,1.43),(2.,1.11,.12),cream,.04)
text('ICE freezer label','ICE',(-18,24.311,.65),1.35,red)
for x in [-18.6,-17.4]:cube('Freezer lid hinge',(x,25.4,1.42),(.2,.1,.10),steel,.015)
cube('Freezer handle',(-18,24.27,1.35),(.45,.08,.06),steel,.015)
# Selected licensed pump: bake original hierarchy transforms, reuse meshes for both instances.
pump_sources=[o for o in source if o.type=='MESH' and o!=board]
pump_materials={}
for src in pump_sources:
 mesh=src.data.copy();mesh.transform(src.matrix_world)
 for i,m in enumerate(mesh.materials):
  if m not in pump_materials:
   nm=m.copy();nm.name='Servo '+m.name;nm.use_backface_culling='Alpha' not in m.name
   bs=next(n for n in nm.node_tree.nodes if n.type=='BSDF_PRINCIPLED')
   bs.inputs['Emission Strength'].default_value=0
   for n in nm.node_tree.nodes:
    if n.type=='NORMAL_MAP':n.inputs['Strength'].default_value=.25
   compact_images(nm)
   pump_materials[m]=nm
  mesh.materials[i]=pump_materials[m]
 for x in [7.6,10.8]:
  o=bpy.data.objects.new('Vintage fuel pump (Rescue3D)',mesh);col.objects.link(o);objects.append(o);o.location=(x,26.15,.20)
for x in [7.6,10.8]:
 cube('Pump island',(x,26.15,.10),(1.6,1.7,.20),concrete,.10)
 for dx in [-.8,.8]:
  rod('Pump protective bollard',(x+dx,25.32,0),(x+dx,25.32,.68),.065,red)
# Simple, low-cost parked Australian ute, rear of forecourt. No gameplay collider.
cx=6.7;cy=31.4
cube('Ute chassis',(cx,cy,.55),(5.5,2.05,.30),dark,.08)
cube('Ute bonnet',(cx-1.8,cy,1.10),(1.65,2.12,.62),ute,.12)
cube('Ute cabin',(cx-.3,cy,1.55),(1.8,2.02,1.55),ute,.16)
cube('Ute front windscreen',(cx-1.22,cy,1.85),(.025,1.72,.74),glass,.02)
for side in [-1,1]:
 cube('Ute side window',(cx-.3,cy+side*1.02,1.87),(1.4,.03,.68),glass,.04)
 cube('Ute door handle',(cx+.2,cy+side*1.06,1.38),(.24,.04,.06),steel,.01)
 cube('Ute tray side',(cx+1.7,cy+side*1.00,1.04),(2.3,.15,.66),ute,.04)
 cube('Ute mirror',(cx-.91,cy+side*1.18,1.84),(.28,.21,.22),dark,.04)
cube('Ute tray floor',(cx+1.7,cy,.78),(2.3,1.95,.14),steel)
cube('Ute tailgate',(cx+2.8,cy,1.04),(.12,2.05,.66),ute)
cube('Ute dark grille',(cx-2.65,cy,1.00),(.05,1.25,.31),dark)
for side in [-1,1]:cube('Ute headlight',(cx-2.66,cy+side*.79,1.15),(.06,.35,.26),white)
for x in [cx-1.75,cx+1.65]:
 for y in [cy-1.04,cy+1.04]:
  rod('Ute tyre',(x,y-.15,.55),(x,y+.15,.55),.51,dark,16)
  rod('Ute wheel hub',(x,y-.16,.55),(x,y+.16,.55),.26,steel,12)
for y in [cy-.8,cy+.8]:rod('Ute bull bar',(cx-2.84,y,.7),(cx-2.84,y,1.52),.055,steel)
rod('Ute bull bar crosspiece',(cx-2.84,cy-.9,1.4),(cx-2.84,cy+.9,1.4),.06,steel)
# Remove only appended staging sources, never other open scenes.
for o in source:bpy.data.objects.remove(o,do_unlink=True)
# Consolidate authored geometry by material in a single mesh (few runtime draw calls).
bpy.ops.object.select_all(action='DESELECT')
meshes=[o for o in objects if o.type=='MESH']
for o in meshes:o.select_set(True)
bpy.context.view_layer.objects.active=meshes[0];bpy.ops.object.join();model=bpy.context.object;model.name='BushServo_BuildingForecourt'
bpy.ops.object.transform_apply(location=True,rotation=True,scale=True)
# Assert no added geometry crosses into playable northern boundary.
assert min(v.co.y for v in model.data.vertices)>24.0
asset=ROOT/'rust/crates/bbq_app/assets/models/smoko_servo.glb'
bpy.ops.export_scene.gltf(filepath=str(asset),use_selection=True,use_active_scene=True,export_apply=True)
# Save only this scene and its dependencies (not earlier large open landscape scenes).
bpy.data.libraries.write(str(ROOT/'art/bush-servo/smoko-servo.blend'),{scene},fake_user=True,compress=True)
report={'asset_bytes':asset.stat().st_size,'vertices':len(model.data.vertices),'polygons':len(model.data.polygons),'materials':len(model.data.materials),'minimum_blender_y':min(v.co.y for v in model.data.vertices),'images':{n.image.name:list(n.image.size) for m in model.data.materials if m and m.use_nodes for n in m.node_tree.nodes if n.type=='TEX_IMAGE' and n.image}}
(ROOT/'checkpoints/2026-10-27-servo-building/asset-report.json').write_text(json.dumps(report,indent=2));print(json.dumps(report,indent=2))
