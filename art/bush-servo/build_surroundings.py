import bpy, math, random, json
from pathlib import Path
from mathutils import Vector
ROOT=Path('/Users/marcus/Claude/australian-bbq')
layout=json.loads((ROOT/'art/bush-servo/surroundings-layout.json').read_text())
rng=random.Random(layout['seed'])
scene=bpy.data.scenes.new('Servo Surroundings')
bpy.context.window.scene=scene
materials=[]
def material(name,color):
 m=bpy.data.materials.new(name); m.use_nodes=True
 n=next(n for n in m.node_tree.nodes if n.type=='BSDF_PRINCIPLED')
 n.inputs['Base Color'].default_value=(*color,1); n.inputs['Roughness'].default_value=.92
 materials.append(m); return m
sand=[material('Sandstone '+str(i),c) for i,c in enumerate([(.57,.35,.19),(.65,.43,.25),(.72,.49,.29),(.61,.39,.22)])]
for i,m in enumerate(sand):
 grain=bpy.data.images.load(str(ROOT/f'art/bush-servo/sandstone-grain-{i}.png'),check_existing=True); grain.pack()
 n=next(n for n in m.node_tree.nodes if n.type=='BSDF_PRINCIPLED')
 tex=m.node_tree.nodes.new('ShaderNodeTexImage'); tex.image=grain
 m.node_tree.links.new(tex.outputs['Color'],n.inputs['Base Color'])
grass=[material('Dry grass '+str(i),c) for i,c in enumerate([(.37,.32,.12),(.54,.46,.22),(.45,.43,.19)])]
leaf=[material('Saltbush '+str(i),c) for i,c in enumerate([(.25,.32,.17),(.36,.41,.24)])]
objects=[]; report=[]
for i,(x,z,w,d,h) in enumerate(layout['rocks']):
 bpy.ops.mesh.primitive_cube_add(size=1,location=(x,-z,h/2))
 o=bpy.context.object; o.name=f'Sandstone_{i:02}'
 o.dimensions=(w,d,h); bpy.ops.object.transform_apply(location=False,rotation=False,scale=True)
 bevel=o.modifiers.new('Worn edges','BEVEL'); bevel.width=.045; bevel.segments=2
 bpy.context.view_layer.objects.active=o; bpy.ops.object.modifier_apply(modifier=bevel.name)
 o.data.materials.append(sand[i%4]); objects.append(o)
 coords=[o.matrix_world@Vector(c) for c in o.bound_box]
 actual=[max(v[k] for v in coords)-min(v[k] for v in coords) for k in range(3)]
 assert all(abs(a-b)<.001 for a,b in zip(actual,[w,d,h]))
 report.append({'stone':i,'dimensions':actual,'collision':[w,d,h]})
verts=[]; faces=[]; mids=[]
def quad(points,mi):
 start=len(verts); verts.extend(points); faces.append(tuple(range(start,start+len(points)))); mids.append(mi)
# Plants stay outside the scoring circle and four broad approach lanes.
for i in range(145):
 a=rng.uniform(0,math.tau); r=rng.uniform(6.9,8.8)
 x=math.cos(a)*r; z=10+math.sin(a)*r
 if abs(x)<layout['clear_lane_half_width'] or abs(z-10)<layout['clear_lane_half_width']: continue
 if abs(x-5.6)<1.2 and abs(z-15.9)<1.2: continue
 for j in range(rng.randint(7,11)):
  ang=rng.uniform(0,math.tau); h=rng.uniform(.13,.42); w=rng.uniform(.012,.025)
  px=x+rng.uniform(-.13,.13); py=-z+rng.uniform(-.13,.13)
  dx,dy=math.cos(ang),math.sin(ang); sx,sy=-dy*w,dx*w
  quad([(px+sx,py+sy,.015),(px-sx,py-sy,.015),(px+dx*h*.16-sx*.5,py+dy*h*.16-sy*.5,h*.6),(px+dx*h*.34,py+dy*h*.34,h)],rng.randrange(3))
# Compact native shrubs with tapered leaves, irregular open silhouettes.
for a in [32,57,124,147,213,237,302,327]:
 a=math.radians(a); x=8.2*math.cos(a); z=10+8.2*math.sin(a)
 for j in range(55):
  ang=rng.uniform(0,math.tau); r=rng.uniform(.04,.42); h=rng.uniform(.12,.55)*(1-r*.7)
  px=x+math.cos(ang)*r; py=-z+math.sin(ang)*r
  dx,dy=math.cos(ang)*.13,math.sin(ang)*.13; sx,sy=-math.sin(ang)*.04,math.cos(ang)*.04
  quad([(px-dx*.4,py-dy*.4,h-.05),(px+sx,py+sy,h),(px+dx,py+dy,h+.07),(px-sx,py-sy,h)],3+rng.randrange(2))
mesh=bpy.data.meshes.new('Native planting'); mesh.from_pydata(verts,[],faces); mesh.update()
o=bpy.data.objects.new('Dry grasses and saltbush',mesh); scene.collection.objects.link(o)
for m in grass+leaf: mesh.materials.append(m)
for p,mi in zip(mesh.polygons,mids): p.material_index=mi
objects.append(o)
bpy.ops.object.select_all(action='DESELECT')
for o in objects:o.select_set(True)
bpy.context.view_layer.objects.active=objects[0]
# Join stones and planting to minimize scene overhead; material batches remain separate.
bpy.ops.object.join(); asset=bpy.context.object; asset.name='Smoko sandstone and native planting'
bpy.ops.wm.save_as_mainfile(filepath=str(ROOT/'art/bush-servo/smoko-surroundings.blend'))
bpy.ops.export_scene.gltf(filepath=str(ROOT/'rust/crates/bbq_app/assets/models/smoko_surroundings.glb'),use_selection=True,use_active_scene=True,export_apply=True)
(ROOT/'art/bush-servo/surroundings-bounds.json').write_text(json.dumps(report,indent=2)+'\n')
print({'stones':len(report),'polygons':len(asset.data.polygons),'materials':len(asset.data.materials)})
