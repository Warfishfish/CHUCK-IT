"""Original seamless comic lawn and a Blender material comparison scene."""
import bpy, math, random
import numpy as np
from pathlib import Path
from mathutils import Vector

ROOT = Path('/Users/marcus/Claude/australian-bbq')
OUT = ROOT / 'checkpoints/2026-11-16-grass'
size = 512
y, x = np.mgrid[0:size, 0:size] / size
# Periodic broad painted shapes keep detail coherent across tile edges.
field = (np.sin(2*math.pi*(2*x+y))*.45 +
         np.sin(2*math.pi*(x-3*y)+1.7)*.25 +
         np.cos(2*math.pi*(4*x+2*y)+.8)*.15)
bands = np.round(field*3)/3
rgb = np.empty((size,size,3),dtype=np.float32)
rgb[:] = [0.42,0.49,0.34]
rgb += bands[:,:,None]*np.array([.025,.026,.018])
rng = random.Random(6241)
# Sparse tapered ink and paint strokes with wrapped coordinates.
for _ in range(190):
    px, py = rng.randrange(size), rng.randrange(size)
    length = rng.randrange(4,12)
    dx = rng.choice([-1,1])*.45
    colour = np.array(rng.choice([[.34,.40,.29],[.49,.54,.38],[.39,.46,.32]]))
    for k in range(length):
        ix, iy = int(px+dx*k)%size, (py+k)%size
        rgb[iy,ix] = rgb[iy,ix]*.4+colour*.6
rgba = np.concatenate([np.clip(rgb,0,1), np.ones((size,size,1))],axis=2)
image = bpy.data.images.new('BBQ sage comic lawn',width=size,height=size)
image.colorspace_settings.name='sRGB'
image.pixels.foreach_set(rgba.astype(np.float32).ravel())
image.filepath_raw=str(OUT/'sage-comic-lawn.png')
image.file_format='PNG'
image.save()

scene=bpy.data.scenes.new('BBQ grass material comparison')
scene.world=bpy.data.worlds.new('BBQ material study sky')
scene.world.use_nodes=True
bg=next(n for n in scene.world.node_tree.nodes if n.type=='BACKGROUND')
bg.inputs[0].default_value=(.65,.73,.82,1)
bg.inputs[1].default_value=.7
def material(name,colour,texture=None):
    mat=bpy.data.materials.new(name);mat.use_nodes=True
    bs=next(n for n in mat.node_tree.nodes if n.type=='BSDF_PRINCIPLED')
    bs.inputs['Base Color'].default_value=(*colour,1)
    bs.inputs['Roughness'].default_value=.95
    if texture:
        tex=mat.node_tree.nodes.new('ShaderNodeTexImage');tex.image=texture
        mat.node_tree.links.new(tex.outputs['Color'],bs.inputs['Base Color'])
    return mat
def box(name,loc,scale,mat):
    mesh=bpy.data.meshes.new(name)
    verts=[(a,b,c) for a in [-1,1] for b in [-1,1] for c in [-1,1]]
    faces=[(0,4,6,2),(1,3,7,5),(0,1,5,4),(2,6,7,3),(0,2,3,1),(4,5,7,6)]
    mesh.from_pydata(verts,[],faces);mesh.update()
    obj=bpy.data.objects.new(name,mesh);scene.collection.objects.link(obj)
    obj.location=loc;obj.scale=scale;obj.data.materials.append(mat)
    return obj
old=bpy.data.images.load(str(ROOT/'checkpoints/2026-11-16-grass/painted-grass-preview.png'),check_existing=True)
wood=material('Study warm timber',(.43,.26,.13))
cream=material('Study cream wall',(.72,.68,.53))
blue=material('Study esky blue',(.07,.29,.65))
pink=material('Study blob coral',(.78,.18,.23))
for centre,tex,title in [(-5,old,'Previous olive'),(5,image,'Revised sage')]:
    ground=box(title,(centre,0,-.08),(4.7,5,.08),material(title,(1,1,1),tex))
    # Cuboid UVs for repeating lawn, planar mapping generated explicitly.
    uv=ground.data.uv_layers.new()
    for poly in ground.data.polygons:
        for li in poly.loop_indices:
            co=ground.data.vertices[ground.data.loops[li].vertex_index].co
            uv.data[li].uv=((co.x+1)*1.5,(co.y+1)*1.5)
    box(title+' house',(centre,3,1.2),(3,.25,1.2),cream)
    box(title+' bench seat',(centre-1,0,.55),(1.2,.4,.08),wood)
    for dx in [-2,-.1]:
        box(title+' bench leg',(centre+dx,0,.25),(.08,.3,.25),wood)
    box(title+' esky',(centre+2,1,.35),(.5,.35,.35),blue)
    box(title+' esky lid',(centre+2,1,.73),(.53,.38,.05),cream)
    # Rounded proxy indicates silhouette/value separation in this material study.
    mesh=bpy.data.meshes.new(title+' blob mesh')
    from mathutils import Matrix
    import bmesh
    bm=bmesh.new();bmesh.ops.create_uvsphere(bm,u_segments=20,v_segments=12,radius=1)
    bm.to_mesh(mesh);bm.free()
    obj=bpy.data.objects.new(title+' coral proxy',mesh);scene.collection.objects.link(obj)
    obj.location=(centre+1,-1.8,.85);obj.scale=(.5,.5,.85);obj.data.materials.append(pink)
light=bpy.data.lights.new('Study sun','SUN');light.energy=2.0
sun=bpy.data.objects.new('Study sun',light);scene.collection.objects.link(sun);sun.rotation_euler=(.45,-.4,-.5)
camera=bpy.data.cameras.new('Study camera')
obj=bpy.data.objects.new('Study camera',camera);scene.collection.objects.link(obj)
obj.location=(0,-17,17);obj.rotation_euler=(Vector((0,0,0))-obj.location).to_track_quat('-Z','Y').to_euler()
camera.type='ORTHO';camera.ortho_scale=22;scene.camera=obj
scene.render.engine='BLENDER_EEVEE'
scene.render.resolution_x=1200;scene.render.resolution_y=700;scene.render.resolution_percentage=100
scene.render.filepath=str(OUT/'blender-grass-comparison.png')
bpy.ops.render.render(scene=scene.name,write_still=True)
print('Saved texture and comparison',image.filepath_raw,scene.render.filepath)
