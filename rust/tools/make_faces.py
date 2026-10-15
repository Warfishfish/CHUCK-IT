"""Build smooth, shared face meshes in Blender; preserve the user's live scene.
Run through connected Blender, or blender --background --python rust/tools/make_faces.py.
Coordinates are game-native: X right, Y up, Z out; one head radius is one unit.
"""
import bpy
import json
import math
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUTPUT = ROOT / 'rust/crates/bbq_app/assets/models/face_pop.json'
shapes = {}


def save(name, vertices, faces):
    label = 'BBQ Face ' + name
    old = bpy.data.meshes.get(label)
    if old and old.users == 0:
        bpy.data.meshes.remove(old)
    mesh = bpy.data.meshes.new(label)
    mesh.from_pydata(vertices, [], faces)
    mesh.validate()
    mesh.update()
    for polygon in mesh.polygons:
        polygon.use_smooth = True
    mesh.calc_loop_triangles()
    shapes[name] = {
        'positions': [[round(c, 6) for c in v.co] for v in mesh.vertices],
        'normals': [[round(c, 6) for c in v.normal] for v in mesh.vertices],
        'indices': [i for tri in mesh.loop_triangles for i in tri.vertices],
    }


def surface(x, y):
    # Follow the head's curvature instead of disappearing into it at the ends.
    return math.sqrt(max(.01, 1 - x*x - y*y)) - 1 + .023


def tube(name, half_width, lift, tilt, radius, offset=0, bushy=False):
    verts, faces = [], []
    count, sides = 25, 10
    for i in range(count):
        u = i/(count-1)*2-1
        x = u*half_width + offset
        y = lift*u*u + tilt*u
        dy = (2*lift*u+tilt)/half_width
        length = math.sqrt(1+dy*dy)
        # Rounded tips, with a subtle organic swell in the bushy choice.
        taper = max(.06, math.sin(math.pi*i/(count-1))**.25)
        r = radius*taper*(1+.1*math.cos(u*12) if bushy else 1)
        for j in range(sides):
            a = j/sides*math.tau
            xx = x-math.sin(a)*r*dy/length
            yy = y+math.sin(a)*r/length
            verts.append((xx, yy, surface(xx, yy)+math.cos(a)*r*.7))
    for i in range(count-1):
        for j in range(sides):
            a=i*sides+j; b=i*sides+(j+1)%sides
            faces.append((a,a+sides,b+sides,b))
    faces += [tuple(range(sides)), tuple((count-1)*sides+j for j in reversed(range(sides)))]
    save(name, verts, faces)


def oval(name, width, upper, lower, y=0, z=0, depth=.016):
    verts, faces = [], []
    rings, sides = 12, 32
    # A flattened smooth closed surface; a flatter upper lip gives the grin its D shape.
    for i in range(rings+1):
        lat = math.pi*i/rings
        for j in range(sides):
            a = j/sides*math.tau
            x = math.sin(lat)*math.cos(a)*width
            sy = math.sin(lat)*math.sin(a)
            yy = y + sy*(upper if sy>=0 else lower)
            verts.append((x, yy, surface(x,yy)+z+math.cos(lat)*depth))
    for i in range(rings):
        for j in range(sides):
            a=i*sides+j; b=i*sides+(j+1)%sides
            faces.append((a,a+sides,b+sides,b))
    save(name, verts, faces)


tube('smile_dark', .32, .13, 0, .022)
tube('smirk_dark', .29, .09, .055, .024, offset=.025)
oval('grin_dark', .34, .075, .22)
oval('grin_teeth', .27, .025, .037, y=.018, z=.023, depth=.01)
oval('grin_tongue', .17, .052, .036, y=-.142, z=.023, depth=.01)
oval('grit_dark', .30, .08, .105)
oval('grit_teeth', .255, .04, .053, y=-.005, z=.023, depth=.01)
oval('ouch_dark', .16, .14, .20)
oval('ouch_tongue', .09, .035, .032, y=-.133, z=.023, depth=.01)
oval('sip_dark', .075, .085, .085)
tube('brow_flat', .235, -.012, 0, .047)
tube('brow_arched', .23, -.09, 0, .035)
tube('brow_bushy', .25, -.035, .012, .067, bushy=True)
OUTPUT.write_text(json.dumps(shapes, separators=(',', ':'))+'\n')
print(f'Exported {len(shapes)} meshes, {sum(len(m["indices"])//3 for m in shapes.values())} triangles, {OUTPUT.stat().st_size} bytes to {OUTPUT}')
