"""Deterministic Bush Servo gum woodland, mesas and perimeter cover. Blender 5.x.
Concept refinement: open eucalyptus crowns made of tapered opaque leaves.
No alpha cards, extra draw-call batches or per-frame foliage work.
"""
import bpy, math, random, json, struct, zlib
import numpy as np
from pathlib import Path
from mathutils import Vector
ROOT=Path('/Users/marcus/Claude/australian-bbq')
layout=json.loads((ROOT/'art/bush-servo/landscape-layout.json').read_text())
rng=random.Random(layout['seed'])
NEAR_CROWN_LEAVES=120
DISTANT_CROWN_LEAVES=48
LEAF_LENGTH=(.32,.56)
BARK_RESOLUTION=512
BARK_NORMAL_STRENGTH=.20
BARK_VALUE_RANGE=(.61,.84)
STONE_TEXTURE_RESOLUTION=256
scene=bpy.data.scenes.new('Bush Servo Landscape');bpy.context.window.scene=scene
verts=[];faces=[];mids=[];materials=[]
def mat(name,c):
 m=bpy.data.materials.new(name);m.use_nodes=True
 n=next(n for n in m.node_tree.nodes if n.type=='BSDF_PRINCIPLED')
 n.inputs['Base Color'].default_value=(*c,1);n.inputs['Roughness'].default_value=.94
 # Solid leaf geometry: no alpha blending or alpha-test shimmer.
 m.use_backface_culling=False
 materials.append(m);return len(materials)-1
bark=mat('Gum pale bark',(.58,.53,.40));dark=mat('Gum peeling bark',(.29,.23,.15))
leaves=[mat('Gum foliage '+str(i),c) for i,c in enumerate([(.12,.18,.075),(.22,.28,.12),(.32,.35,.18),(.17,.25,.14)])]
groundcover=leaves
mesa=[mat('Ochre strata '+str(i),c) for i,c in enumerate([(.28,.075,.025),(.36,.115,.045),(.43,.16,.065),(.32,.09,.035)])]
rockmat=mat('Sun warmed rock',(.52,.36,.23))
pebblemat=mat('Desert stone edges',(.48,.31,.19))
def surface_map(mi,name):
 # Small baked colour variation; existing runtime mip filtering prevents shimmer.
 # No procedural shader work or new material batches at runtime.
 size=STONE_TEXTURE_RESOLUTION; yy,xx=np.mgrid[0:size,0:size]/size
 nr=np.random.default_rng(91+mi)
 shade=1+.065*np.sin(xx*math.tau*3+np.sin(yy*math.tau*2))+.045*np.sin(yy*math.tau*11)+nr.uniform(-.035,.035,(size,size))
 m=materials[mi];bs=next(n for n in m.node_tree.nodes if n.type=='BSDF_PRINCIPLED')
 colour=np.array(bs.inputs['Base Color'].default_value[:3])
 rgba=np.ones((size,size,4),np.float32);rgba[:,:,:3]=np.clip(colour[None,None,:]*shade[:,:,None],0,1)
 im=bpy.data.images.new(name,width=size,height=size);im.pixels.foreach_set(rgba.ravel());im.file_format='PNG'
 path=ROOT/'art/bush-servo/detail-textures'/ (name+'.png');path.parent.mkdir(exist_ok=True)
 im.filepath_raw=str(path);im.save();im.pack()
 tex=m.node_tree.nodes.new('ShaderNodeTexImage');tex.image=im
 m.node_tree.links.new(tex.outputs['Color'],bs.inputs['Base Color'])
# Reuse a packed CC0 Poly Haven source; export only compact, recoloured maps.
with bpy.data.libraries.load(str(ROOT/'art/bush-servo/library/eucalyptus-bark.blend'),link=False) as (src,dst):
 dst.materials=[n for n in src.materials if n.startswith('eucalyptus_bark')][:1]
source_bark=dst.materials[0]
bark_bs=next(n for n in materials[bark].node_tree.nodes if n.type=='BSDF_PRINCIPLED')
for source_node in source_bark.node_tree.nodes:
 if source_node.type!='TEX_IMAGE' or not source_node.image:continue
 is_colour='Diffuse' in source_node.image.name;is_normal='nor_gl' in source_node.image.name
 if not (is_colour or is_normal):continue
 im=source_node.image.copy();im.scale(BARK_RESOLUTION,BARK_RESOLUTION)
 pixels=np.empty(len(im.pixels),dtype=np.float32);im.pixels.foreach_get(pixels);pixels=pixels.reshape(-1,4)
 if is_colour:
  grey=pixels[:,:3]@np.array([.2126,.7152,.0722])
  low,high=np.percentile(grey,[3,97]);value=BARK_VALUE_RANGE[0]+(BARK_VALUE_RANGE[1]-BARK_VALUE_RANGE[0])*np.clip((grey-low)/max(high-low,.001),0,1)
  pixels[:,:3]=value[:,None]*np.array([1.,.97,.90])
  name=f'gum-bark-pale-{BARK_RESOLUTION}'
 else:
  v=pixels[:,:3]*2-1;v[:,:2]*=BARK_NORMAL_STRENGTH;v/=np.maximum(np.linalg.norm(v,axis=1,keepdims=True),1e-6)
  pixels[:,:3]=v*.5+.5;name=f'gum-bark-normal-{BARK_RESOLUTION}'
 # Write 8-bit PNG then reload: copying packed source images otherwise leaves
 # the original 1K/16-bit payload attached and glTF exports that stale payload.
 path=ROOT/'art/bush-servo/detail-textures'/(name+'.png');path.parent.mkdir(exist_ok=True)
 rgba=(np.clip(pixels,0,1)*255+.5).astype(np.uint8).reshape(BARK_RESOLUTION,BARK_RESOLUTION,4)
 def chunk(kind,data):return struct.pack('>I',len(data))+kind+data+struct.pack('>I',zlib.crc32(kind+data)&0xffffffff)
 raw=b''.join(b'\0'+row.tobytes() for row in rgba)
 path.write_bytes(b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',BARK_RESOLUTION,BARK_RESOLUTION,8,6,0,0,0))+chunk(b'IDAT',zlib.compress(raw,8))+chunk(b'IEND',b''))
 runtime=bpy.data.images.load(str(path),check_existing=False);runtime.colorspace_settings.name=im.colorspace_settings.name;runtime.pack()
 tex=materials[bark].node_tree.nodes.new('ShaderNodeTexImage');tex.image=runtime
 if is_colour:materials[bark].node_tree.links.new(tex.outputs['Color'],bark_bs.inputs['Base Color'])
 else:
  normal=materials[bark].node_tree.nodes.new('ShaderNodeNormalMap')
  materials[bark].node_tree.links.new(tex.outputs['Color'],normal.inputs['Color'])
  materials[bark].node_tree.links.new(normal.outputs['Normal'],bark_bs.inputs['Normal'])
surface_map(rockmat,'warm-rock-grain')
surface_map(pebblemat,'desert-stone-edge')
for mi in mesa:surface_map(mi,'ochre-rock-'+str(mi))
def face(points,mi):
 i=len(verts);verts.extend([tuple(p) for p in points]);faces.append(tuple(range(i,i+len(points))));mids.append(mi)
def rod(a,b,r1,r2,mi,seg=7):
 a=Vector(a);b=Vector(b);d=(b-a).normalized();u=d.cross(Vector((0,0,1)))
 if u.length<.01:u=d.cross(Vector((1,0,0)))
 u.normalize();v=d.cross(u)
 aa=[a+(u*math.cos(i*math.tau/seg)+v*math.sin(i*math.tau/seg))*r1 for i in range(seg)]
 bb=[b+(u*math.cos(i*math.tau/seg)+v*math.sin(i*math.tau/seg))*r2 for i in range(seg)]
 for i in range(seg):face([aa[i],aa[(i+1)%seg],bb[(i+1)%seg],bb[i]],mi)
 face(bb,mi)
def leaf(p,angle,length,mi,droop=.4):
 p=Vector(p);d=Vector((math.cos(angle),math.sin(angle),-droop)).normalized()*length;s=Vector((-math.sin(angle),math.cos(angle),0))*length*.12
 # Tapered leaf quad: two triangles, no transparent cards or dense folded geometry.
 face([p,p+d*.45+s+Vector((0,0,length*.04)),p+d,p+d*.45-s+Vector((0,0,length*.04))],mi)

def crown(p,seed,near=True):
 # Open, drooping eucalyptus sprays replace the old solid polygon blobs.
 # Stratified placement keeps the silhouette even without uniform rows.
 rr=random.Random(seed);p=Vector(p)
 count=NEAR_CROWN_LEAVES if near else DISTANT_CROWN_LEAVES
 for i in range(count):
  a=i*2.399963+rr.uniform(-.22,.22)
  r=math.sqrt((i+.5)/count)*rr.uniform(.85,1.20)
  q=p+Vector((math.cos(a)*r,math.sin(a)*r,rr.uniform(-.65,.75)*(1-r*.3)))
  length=rr.uniform(*LEAF_LENGTH)*(1 if near else 1.2)
  direction=a+rr.uniform(-.7,.7)
  leaf(q,direction,length,leaves[rr.choices(range(4),[2,4,2,3])[0]],rr.uniform(.2,1.5))
def tree(x,z,h,seed):
 rr=random.Random(seed);base=Vector((x,-z,0));bend=Vector((rr.uniform(-.55,.55),rr.uniform(-.55,.55),h*.43));fork=base+bend
 rod(base,fork,h*.055,h*.034,bark,9)
 for j in range(4):
  a=j*math.tau/4+rr.uniform(-.4,.4)
  tip=base+Vector((math.cos(a)*h*.27,math.sin(a)*h*.27,h*rr.uniform(.7,.9)))
  elbow=fork.lerp(tip,.5)+Vector((0,0,.3));rod(fork,elbow,h*.031,h*.018,bark);rod(elbow,tip,h*.018,h*.008,bark)
  for k in range(4):
   aa=a+(k-1.5)*.7
   end=tip+Vector((math.cos(aa)*h*.20,math.sin(aa)*h*.20,h*rr.uniform(-.06,.15)))
   rod(tip,end,h*.008,.012,bark,5)
   crown(tip.lerp(end,.65),seed*20+j*4+k,h>=8)
# Frame the arena without a uniform ring or blocking the servo facade.
trees=[(-36,-19,9),(-37,1,10),(-35,20,11),(-22,27,9),(-6,30,8),(14,29,10),(33,26,11),(36,7,11),(36,-16,10),(24,-30,9),(7,-35,8),(-30,-33,9)]
for i,(x,z,h) in enumerate(trees):tree(x,z,h,500+i)
# Two uneven distant woodland layers, smaller crowns and height variation.
for i in range(28):
 a=i*2.399963;r=rng.uniform(47,72);x=math.cos(a)*r;z=math.sin(a)*r*.84
 tree(x,z,rng.uniform(4.5,7.5),800+i)
# Broad terraced mesas, organic outlines instead of smooth pale hemispheres.
for j in range(10):
 a=j*math.tau/10+.13;x=105*math.cos(a);y=-92*math.sin(a)
 w=rng.uniform(15,26);d=rng.uniform(12,19);h=rng.uniform(9,16);n=28
 radial=[rng.uniform(.88,1.12) for _ in range(n)]
 rings=[]
 for lev,scale in [(0,1.3),(.25,1),(.34,.91),(.72,.82),(.82,.76),(1,.67)]:
  rings.append([Vector((x+math.cos(i*math.tau/n)*w*scale*radial[i],y+math.sin(i*math.tau/n)*d*scale*radial[i],-1+lev*h+(.3*math.sin(i*2.1) if lev else 0))) for i in range(n)])
 for k in range(len(rings)-1):
  for i in range(n):face([rings[k][i],rings[k][(i+1)%n],rings[k+1][(i+1)%n],rings[k+1][i]],mesa[(k+j)%4])
 face(rings[-1],mesa[2])
# Layered spinifex and saltbush clusters beyond the fence add near-ground scale.
# Keep a clear margin around the playable yard and road; these are visual-only.
for i in range(160):
 a=rng.uniform(0,math.tau);r=rng.uniform(37,78);x=r*math.cos(a);z=r*.85*math.sin(a)
 if abs(x)<34 and abs(z)<25:continue
 for j in range(22):
  aa=rng.uniform(0,math.tau);p=Vector((x+rng.uniform(-.5,.5),-z+rng.uniform(-.5,.5),rng.uniform(.08,.65)))
  leaf(p,aa,rng.uniform(.2,.4),leaves[rng.randrange(4)])
# Broad, low grass fans. Their centres stay outside the play fence; varied
# height, lean and warm tips keep the silhouette readable without alpha cards.
for i in range(380):
 a=rng.uniform(0,math.tau);r=rng.uniform(34.5,56)
 x=r*math.cos(a);z=r*.84*math.sin(a)
 if abs(x)<35 and abs(z)<26:continue
 centre=Vector((x,-z,.10));blades=rng.randint(7,11)
 for j in range(blades):
  aa=rng.uniform(0,math.tau);length=rng.uniform(.32,.76)
  lean=rng.uniform(.18,.48)
  leaf(centre+Vector((rng.uniform(-.10,.10),rng.uniform(-.10,.10),0)),aa,length,
       groundcover[0 if j%4==0 else rng.randrange(3)],droop=-1.2)
# Small, partly buried ochre stones break up the hard soil/rock boundary around
# the outer woodland. All stay outside the fence; none alter shared collision.
for i in range(110):
 a=rng.uniform(0,math.tau);r=rng.uniform(35,51)
 cx=r*math.cos(a);cz=r*.84*math.sin(a)
 if abs(cx)<35 and abs(cz)<26:continue
 for k in range(rng.randint(2,4)):
  x=cx+rng.uniform(-.75,.75);z=cz+rng.uniform(-.75,.75)
  if abs(x)<35 and abs(z)<26:continue
  size=rng.uniform(.22,.62);sx=size*rng.uniform(1.1,1.9);sy=size*rng.uniform(.7,1.2)
  sides=7;phase=rng.uniform(0,math.tau);lower=[];upper=[]
  for j in range(sides):
   aa=phase+j*math.tau/sides;f=rng.uniform(.82,1.18)
   lower.append(Vector((x+math.cos(aa)*sx*f,-z+math.sin(aa)*sy*f,.035)))
   upper.append(Vector((x+math.cos(aa)*sx*f*.78,-z+math.sin(aa)*sy*f*.78,size*rng.uniform(.25,.48))))
  for j in range(sides):face([lower[j],lower[(j+1)%sides],upper[(j+1)%sides],upper[j]],pebblemat)
  face(upper,pebblemat)
mesh=bpy.data.meshes.new('Gum woodland and ochre ridges');mesh.from_pydata(verts,[],faces);mesh.update()
obj=bpy.data.objects.new('Gums scrub and distant mesas',mesh);scene.collection.objects.link(obj)
for m in materials:mesh.materials.append(m)
for p,mi in zip(mesh.polygons,mids):
 p.material_index=mi
 p.use_smooth=mi in leaves or mi==bark
# Share touching crown vertices for soft normals, retaining crisp trunks and rocks.
import bmesh
bm=bmesh.new();bm.from_mesh(mesh);bmesh.ops.remove_doubles(bm,verts=list(bm.verts),dist=.00001);bm.to_mesh(mesh);bm.free()
objects=[obj];bounds=[]
for i,(x,z,w,d,h) in enumerate(layout['cover']):
 bpy.ops.mesh.primitive_cube_add(size=1,location=(x,-z,h/2))
 o=bpy.context.object;o.name=f'Perimeter rock {i}';o.dimensions=(w,d,h);bpy.ops.object.transform_apply(location=False,rotation=False,scale=True)
 mod=o.modifiers.new('Weathered corners','BEVEL');mod.width=.09;mod.segments=2
 bpy.ops.object.modifier_apply(modifier=mod.name);o.data.materials.append(materials[rockmat]);objects.append(o)
 bounds.append({'id':i,'dimensions':list(o.dimensions),'layout':[w,d,h]})
 assert all(abs(a-b)<.001 for a,b in zip(o.dimensions,[w,d,h]))
bpy.ops.object.select_all(action='DESELECT')
for o in objects:o.select_set(True)
bpy.context.view_layer.objects.active=obj;bpy.ops.object.join()
# Axis-projected metre-scale UVs for subtle bark and stone variation.
bpy.ops.object.transform_apply(location=True,rotation=True,scale=True)
while obj.data.uv_layers:obj.data.uv_layers.remove(obj.data.uv_layers[0])
uv=obj.data.uv_layers.new(name='Surface metres')
for poly in obj.data.polygons:
 axis=max(range(3),key=lambda a:abs(poly.normal[a]))
 tile=1.81 if poly.material_index==bark else 3.5
 for li in poly.loop_indices:
  v=obj.data.vertices[obj.data.loops[li].vertex_index].co
  u,w=((v.y,v.z) if axis==0 else ((v.x,v.z) if axis==1 else (v.x,v.y)))
  uv.data[li].uv=(u/tile,w/tile)
# Save only the new scene, preserving every other open Blender scene.
bpy.data.libraries.write(str(ROOT/'art/bush-servo/smoko-landscape.blend'),{scene},fake_user=True,compress=True)
bpy.ops.export_scene.gltf(filepath=str(ROOT/'rust/crates/bbq_app/assets/models/smoko_landscape.glb'),use_selection=True,use_active_scene=True,export_apply=True)
import runpy
print(runpy.run_path(str(ROOT/'art/bush-servo/compact_glb.py'))['compact'](ROOT/'rust/crates/bbq_app/assets/models/smoko_landscape.glb'))
(ROOT/'art/bush-servo/landscape-bounds.json').write_text(json.dumps(bounds,indent=2)+'\n')
print({'trees':len(trees)+28,'perimeter_cover':len(bounds),'polygons':len(obj.data.polygons),'materials':len(obj.data.materials)})
