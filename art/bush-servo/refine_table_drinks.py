"""Set the east picnic table for two. Input is the reviewed tyre-and-timber scene."""
from pathlib import Path
import math
import bpy
from mathutils import Vector

ROOT = Path('/Users/marcus/Claude/australian-bbq')
# Use the same author-scene loader, GLB compaction and baked depth treatment as the crate pass.
helpers = (ROOT / 'art/bush-servo/refine_entrance.py').read_text().split('# Crates are actual shared')[0]
exec(compile(helpers, 'arena export helpers', 'exec'))
scene = load('arena-tyre-timber.blend', 'Eastern picnic tea break')
accessories = [o for o in scene.objects if 7.2 < o.location.x < 9.8 and 2 < o.location.y < 3]
def material(prefix):
    return next(o.data.materials[0] for o in accessories if o.name.startswith(prefix))
ivory, steel, dark, green = map(material, ['Enamel mug','Mug lip','Tea surface','Smoko thermos'])
old = [o for o in accessories if o.name.startswith(('Enamel mug','Mug handle','Mug lip','Tea surface','Extra mug','Extra enamel','Smoko thermos','Thermos collar','Thermos cup lid'))]
assert len(old) == 14
for obj in old:
    bpy.data.objects.remove(obj, do_unlink=True)

def vessel(name, x, y, base, profile, segments=24):
    """Turn a radius/height profile with a material for each outgoing surface."""
    verts, rings, faces, finishes = [], [], [], []
    for radius, height, finish in profile:
        ring = []
        for i in range(segments if radius else 1):
            a = i * math.tau / segments
            ring.append(len(verts))
            verts.append((radius*math.cos(a),radius*math.sin(a),height))
        rings.append(ring)
    for j, (lower, upper) in enumerate(zip(rings, rings[1:])):
        for i in range(segments):
            k = (i+1) % segments
            if len(lower)==1: face = (lower[0],upper[k],upper[i])
            elif len(upper)==1: face = (lower[i],lower[k],upper[0])
            else: face = (lower[i],lower[k],upper[k],upper[i])
            faces.append(face)
            finishes.append(profile[j][2])
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    mesh.update()
    obj = bpy.data.objects.new(name, mesh)
    scene.collection.objects.link(obj)
    obj.location = (x,y,base)
    mats = list(dict.fromkeys(finishes))
    for mat in mats: mesh.materials.append(mat)
    for poly, mat in zip(mesh.polygons, finishes):
        poly.material_index = mats.index(mat)
        poly.use_smooth = abs(poly.normal.z) < .95
    return obj

def bent_handle(name,x,y,z,angle,width,height,radius,finish):
    """Rounded D-shaped handle whose ends enter the vessel wall."""
    curve = bpy.data.curves.new(name,'CURVE')
    curve.dimensions = '3D'
    curve.resolution_u = 4
    curve.bevel_depth = radius
    curve.bevel_resolution = 2
    spline = curve.splines.new('BEZIER')
    path = [(0,0),(.82,.08),(1,.45),(.86,.9),(0,1)]
    spline.bezier_points.add(len(path)-1)
    for point,(u,v) in zip(spline.bezier_points,path):
        point.co = (x+width*u*math.cos(angle),y+width*u*math.sin(angle),z+height*v)
        point.handle_left_type = point.handle_right_type = 'AUTO'
    obj = bpy.data.objects.new(name,curve)
    scene.collection.objects.link(obj)
    curve.materials.append(finish)
    bpy.ops.object.select_all(action='DESELECT')
    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.convert(target='MESH')

def mug(x,y,z,finish,angle,filled):
    profile=[(0,0,finish),(.064,0,finish),(.074,.009,finish),(.086,.148,finish),
             (.089,.155,steel),(.089,.164,steel),(.081,.166,ivory),(.077,.157,ivory)]
    profile += [(.075,.124,dark),(0,.124,dark)] if filled else [(.060,.018,ivory),(0,.018,ivory)]
    vessel('Enamel camp mug',x,y,z,profile)
    bent_handle('Rolled enamel handle',x+.072*math.cos(angle),y+.072*math.sin(angle),z+.036,angle,.078,.096,.012,finish)

# One cup on the existing saucer, the other opposite it with the handle facing its seat.
mug(8.68,2.38,.977,ivory,-.8,True)
mug(9.17,2.68,.95,green,2.3,False)
x,y=8.18,2.65
vessel('Insulated picnic flask',x,y,.95,[(0,0,steel),(.102,0,steel),(.112,.018,steel),
    (.112,.047,green),(.106,.054,green),(.107,.315,green),(.102,.35,steel),
    (.078,.374,steel),(.081,.385,dark),(.094,.395,ivory),(.093,.472,ivory),
    (.081,.487,ivory),(0,.487,ivory)])
bent_handle('Flask carry grip',x+.099,y,1.03,0,.070,.204,.018,dark)
vessel('Returnable soft drink bottle',7.66,2.36,.95,[(0,0,green),(.059,0,green),
    (.067,.014,green),(.068,.082,ivory),(.068,.165,green),(.066,.222,green),
    (.056,.252,green),(.029,.285,green),(.027,.347,green),(.033,.352,steel),
    (.034,.368,steel),(.028,.373,steel),(0,.373,steel)])
export(scene,'east-picnic-drinks.blend','props')
