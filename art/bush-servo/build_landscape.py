"""Deterministic Bush Servo gum woodland, mesas and perimeter cover. Blender 5.x."""
import bpy, math, random, json
from pathlib import Path
from mathutils import Vector
ROOT=Path('/Users/marcus/Claude/australian-bbq')
layout=json.loads((ROOT/'art/bush-servo/landscape-layout.json').read_text())
rng=random.Random(layout['seed'])
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
leaves=[mat('Gum foliage '+str(i),c) for i,c in enumerate([(.20,.26,.085),(.29,.33,.12),(.38,.39,.16),(.25,.30,.13)])]
mesa=[mat('Ochre strata '+str(i),c) for i,c in enumerate([(.28,.075,.025),(.36,.115,.045),(.43,.16,.065),(.32,.09,.035)])]
rockmat=mat('Sun warmed rock',(.52,.36,.23))
def face(points,mi):
 i=len(verts);verts.extend([tuple(p) for p in points]);faces.append(tuple(range(i,i+len(points))));mids.append(mi)
def rod(a,b,r1,r2,mi,seg=7):
 a=Vector(a);b=Vector(b);d=(b-a).normalized();u=d.cross(Vector((0,0,1)))
 if u.length<.01:u=d.cross(Vector((1,0,0)))
 u.normalize();v=d.cross(u)
 aa=[a+(u*math.cos(i*math.tau/seg)+v*math.sin(i*math.tau/seg))*r1 for i in range(seg)]
 bb=[b+(u*math.cos(i*math.tau/seg)+v*math.sin(i*math.tau/seg))*r2 for i in range(seg)]
 for i in range(seg):face([aa[i],aa[(i+1)%seg],bb[(i+1)%seg],bb[i]],dark if mi==bark and i%5==0 else mi)
 face(bb,mi)
def leaf(p,angle,length,mi):
 p=Vector(p);d=Vector((math.cos(angle),math.sin(angle),-.4))*length;s=Vector((-math.sin(angle),math.cos(angle),0))*length*.17
 # Tapered leaf quad: two triangles, no transparent cards or dense folded geometry.
 face([p,p+d*.45+s+Vector((0,0,length*.04)),p+d,p+d*.45-s+Vector((0,0,length*.04))],mi)

def crown(p,seed):
 rr=random.Random(seed);p=Vector(p);n=9
 rings=[]
 for z,r in [(-.64,.20),(-.28,.88),(.24,1),(.76,.45)]:
  rings.append([p+Vector((math.cos(i*math.tau/n)*r*rr.uniform(.85,1.16),math.sin(i*math.tau/n)*r*rr.uniform(.85,1.16),z+rr.uniform(-.08,.08))) for i in range(n)])
 for k in range(3):
  for i in range(n):face([rings[k][i],rings[k][(i+1)%n],rings[k+1][(i+1)%n],rings[k+1][i]],leaves[seed%4])
 face(list(reversed(rings[0])),leaves[0]);face(rings[-1],leaves[2])
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
   crown(tip.lerp(end,.65),seed*20+j*4+k)
   for l in range(32 if h>=8 else 18):
    t=rr.uniform(.12,1.15);p=tip.lerp(end,t)+Vector((rr.uniform(-.7,.7),rr.uniform(-.7,.7),rr.uniform(-.55,.55)))
    leaf(p,rr.uniform(0,math.tau),rr.uniform(.28,.53)*(h/8)**.3,leaves[rr.randrange(4)])
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
# Scrub beyond the fence; no new invisible obstacles.
for i in range(160):
 a=rng.uniform(0,math.tau);r=rng.uniform(37,78);x=r*math.cos(a);z=r*.85*math.sin(a)
 if abs(x)<34 and abs(z)<25:continue
 for j in range(22):
  aa=rng.uniform(0,math.tau);p=Vector((x+rng.uniform(-.5,.5),-z+rng.uniform(-.5,.5),rng.uniform(.08,.65)))
  leaf(p,aa,rng.uniform(.2,.4),leaves[rng.randrange(4)])
mesh=bpy.data.meshes.new('Gum woodland and ochre ridges');mesh.from_pydata(verts,[],faces);mesh.update()
obj=bpy.data.objects.new('Gums scrub and distant mesas',mesh);scene.collection.objects.link(obj)
for m in materials:mesh.materials.append(m)
for p,mi in zip(mesh.polygons,mids):
 p.material_index=mi
 p.use_smooth=mi in leaves
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
# Preserve the user's prior scene; save the new editable landscape separately.
bpy.ops.wm.save_as_mainfile(filepath=str(ROOT/'art/bush-servo/smoko-landscape.blend'))
bpy.ops.export_scene.gltf(filepath=str(ROOT/'rust/crates/bbq_app/assets/models/smoko_landscape.glb'),use_selection=True,use_active_scene=True,export_apply=True)
(ROOT/'art/bush-servo/landscape-bounds.json').write_text(json.dumps(bounds,indent=2)+'\n')
print({'trees':len(trees)+28,'perimeter_cover':len(bounds),'polygons':len(obj.data.polygons),'materials':len(obj.data.materials)})
