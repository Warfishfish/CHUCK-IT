"""Original connected cloud meshes. Run through Blender MCP; no external assets."""
import bpy, math, random, json, struct
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
old=bpy.context.window.scene
try:
 for variant in range(3):
  scene=bpy.data.scenes.new('Connected cloud '+str(variant));bpy.context.window.scene=scene
  rng=random.Random(171118+variant);parts=[]
  def puff(x,y,z,sx,sy,sz):
   bpy.ops.mesh.primitive_uv_sphere_add(segments=20,ring_count=12,location=(x,-z,y));o=bpy.context.object;o.scale=(sx,sz,sy)
   bpy.ops.object.transform_apply(location=False,rotation=False,scale=True);parts.append(o)
  # Every lobe overlaps the core. No separate thin disc and no isolated lower puffs.
  puff(0,0,0,5.4,1.0,2.1)
  for k in range(5):
   x=(k-2)*1.75;r=rng.uniform(1.65,2.3)
   puff(x,.35+r*.25,rng.uniform(-.35,.35),r,r*rng.uniform(.7,1.0),r*.9)
  bpy.ops.object.select_all(action='SELECT');bpy.context.view_layer.objects.active=parts[0];bpy.ops.object.join();o=parts[0]
  rem=o.modifiers.new('Connected vapour','REMESH');rem.mode='VOXEL';rem.voxel_size=.20;rem.use_smooth_shade=True;bpy.ops.object.modifier_apply(modifier=rem.name)
  smooth=o.modifiers.new('Soft silhouette','SMOOTH');smooth.factor=1.0;smooth.iterations=5;bpy.ops.object.modifier_apply(modifier=smooth.name)
  dec=o.modifiers.new('Browser budget','DECIMATE');dec.ratio=.45;bpy.ops.object.modifier_apply(modifier=dec.name)
  for f in o.data.polygons:f.use_smooth=True
  # A baked broad underside grade avoids tiny hard shading circles and shadow seams.
  attr=o.data.color_attributes.new(name='CloudTone',type='FLOAT_COLOR',domain='POINT')
  for v in o.data.vertices:
   h=(v.co.z+o.location.z+1.1)/3.6;t=max(0,min(1,h));t=t*t*(3-2*t)
   low=(.65,.73,.79);high=(.96,.95,.90)
   attr.data[v.index].color=tuple(low[j]*(1-t)+high[j]*t for j in range(3))+(1,)
  m=bpy.data.materials.new('Soft cloud daylight');m.use_nodes=True;nodes=m.node_tree.nodes;nodes.clear()
  out=nodes.new('ShaderNodeOutputMaterial');bs=nodes.new('ShaderNodeBsdfPrincipled');col=nodes.new('ShaderNodeVertexColor');col.layer_name='CloudTone'
  bs.inputs['Roughness'].default_value=1.0
  m.node_tree.links.new(col.outputs['Color'],bs.inputs['Base Color']);m.node_tree.links.new(bs.outputs[0],out.inputs['Surface']);o.data.materials.clear();o.data.materials.append(m)
  o.name='Continuous cloud silhouette'
  # Verify there is one connected mesh island before export.
  links=[[] for _ in o.data.vertices]
  for e in o.data.edges:a,b=e.vertices;links[a].append(b);links[b].append(a)
  seen={0};stack=[0]
  while stack:
   for n in links[stack.pop()]:
    if n not in seen:seen.add(n);stack.append(n)
  assert len(seen)==len(o.data.vertices),'Detached cloud island'
  path=ROOT/'crates/bbq_app/assets/models'/('cloud_soft_'+str(variant)+'.glb')
  bpy.ops.export_scene.gltf(filepath=str(path),export_format='GLB',use_active_scene=True,export_cameras=False,export_lights=False)
  # Preserve the vertex-painted sky grade without dynamic lighting seams.
  raw=path.read_bytes();length=struct.unpack_from('<I',raw,12)[0];doc=json.loads(raw[20:20+length]);binary=raw[20+length:]
  doc.setdefault('extensionsUsed',[]).append('KHR_materials_unlit')
  for mat in doc['materials']:
   mat.setdefault('extensions',{})['KHR_materials_unlit']={}
   mat['pbrMetallicRoughness']['baseColorFactor']=[1,1,1,1]
  data=json.dumps(doc,separators=(',',':')).encode();data+=b' '*((-len(data))%4)
  path.write_bytes(struct.pack('<III',0x46546c67,2,20+len(data)+len(binary))+struct.pack('<II',len(data),0x4e4f534a)+data+binary)
  print(path.name,'vertices',len(o.data.vertices),'faces',len(o.data.polygons),'one connected island')
finally:bpy.context.window.scene=old
