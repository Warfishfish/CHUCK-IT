"""Build a small, original neighbourhood carport vignette for the Backyard scene.

Run through Blender MCP or Blender's Python console from the repository checkout.
The open artist scene is left untouched; geometry is authored in a temporary scene and
exported as one multi-material GLB. Coordinates use the game's (+Y up, +Z house-front)
axes before Blender/glTF conversion.
"""
from math import cos, pi, sin
from pathlib import Path
import bpy
from mathutils import Vector

OUT = Path(__file__).resolve().parents[1] / "crates/bbq_app/assets/models/backyard_neighbourhood_vignette.glb"


def game_to_blender(p):
    return (p[0], -p[2], p[1])


def material(name, rgb, roughness=.78, metallic=0.0):
    m = bpy.data.materials.new(name)
    m.diffuse_color = (*rgb, 1)
    m.use_nodes = True
    node = next(n for n in m.node_tree.nodes if n.type == "BSDF_PRINCIPLED")
    node.inputs["Base Color"].default_value = (*rgb, 1)
    node.inputs["Roughness"].default_value = roughness
    node.inputs["Metallic"].default_value = metallic
    return m


def box(name, center, size, mat, bevel=0, yaw=0):
    x, y, z = center
    sx, sy, sz = size
    verts = []
    for yy in (-sy/2, sy/2):
        for zz in (-sz/2, sz/2):
            for xx in (-sx/2, sx/2):
                verts.append((x + xx*cos(yaw) + zz*sin(yaw), y+yy,
                              z - xx*sin(yaw) + zz*cos(yaw)))
    faces = [(0,1,3,2),(4,6,7,5),(0,4,5,1),(2,3,7,6),(0,2,6,4),(1,5,7,3)]
    mesh = bpy.data.meshes.new(name + " mesh")
    mesh.from_pydata([game_to_blender(v) for v in verts], [], faces)
    mesh.materials.append(mat)
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    if bevel:
        b = obj.modifiers.new("Soft painted edges", "BEVEL")
        b.width = bevel
        b.segments = 3
        obj.modifiers.new("Weighted corner normals", "WEIGHTED_NORMAL")
    return obj


def rod(name, a, b, radius, mat, sides=10):
    a, b = Vector(a), Vector(b)
    axis = b-a
    midpoint = (a+b)*.5
    obj = bpy.data.objects.new(name, bpy.data.meshes.new(name+" mesh"))
    bpy.context.collection.objects.link(obj)
    obj.location = game_to_blender(midpoint)
    obj.rotation_mode = "QUATERNION"
    # Blender cylinder axis is local Z. Convert game-space vectors after swapping Y/Z.
    direction = Vector((axis.x, -axis.z, axis.y)).normalized()
    obj.rotation_quaternion = Vector((0,0,1)).rotation_difference(direction)
    bpy.ops.mesh.primitive_cylinder_add(vertices=sides, radius=radius, depth=axis.length,
                                        location=obj.location)
    primitive = bpy.context.object
    primitive.name = name
    primitive.data.materials.append(mat)
    if obj.name in bpy.data.objects:
        bpy.data.objects.remove(obj, do_unlink=True)
    return primitive


def build():
    zinc = material("Sun-faded corrugated zinc", (.48,.52,.48), .46, .48)
    timber = material("Carport ironbark posts", (.31,.22,.14))
    trim = material("Painted pale carport trim", (.72,.68,.55))
    body = material("Old cream sedan body", (.47,.45,.37), .72)
    primer = material("Dull red oxide primer", (.43,.20,.13), .76)
    glass = material("Smoky old car glass", (.12,.20,.22), .34, .14)
    rubber = material("Tyre rubber", (.075,.078,.07), .92)
    chrome = material("Cloudy chrome", (.43,.46,.43), .3, .72)
    rust = material("Orange surface rust", (.34,.17,.095), .82)
    concrete = material("Old concrete pad", (.43,.42,.37), .94)
    red = material("Faded red toolbox", (.52,.18,.12), .62)
    dark = material("Workshop shadow and openings", (.11,.12,.10), .9)

    # A narrow rear-lane hardstand and a recognisable open-sided carport.
    box("Carport concrete hardstand", (19.0,.035,44.5), (9.4,.07,10.5), concrete)
    for x in (15.0,23.0):
        for z in (41.0,48.0):
            box("Carport timber post", (x,2.05,z), (.20,4.1,.20), timber, .025)
            box("Post foot shoe", (x,.12,z), (.30,.24,.30), zinc, .02)
    box("Carport front beam", (19.0,3.95,40.98), (8.35,.22,.24), timber, .025)
    box("Carport rear beam", (19.0,3.90,48.0), (8.35,.22,.24), timber, .025)
    box("Carport side beam", (14.98,3.90,44.5), (.22,.22,7.0), timber, .025)
    box("Carport zinc roof", (19.0,4.12,44.5), (8.65,.14,7.55), zinc, .03, yaw=.018)
    for x in (15.2,16.15,17.1,18.05,19.0,19.95,20.9,21.85,22.8):
        box("Roof pan raised seam", (x,4.205,44.5), (.045,.05,7.45), trim, .008)
    for x in (15.2,22.8):
        rod("Diagonal timber brace", (x,2.8,41.05), (x,3.9,42.2), .07, timber, 6)

    # A weathered 1970s sedan shell with its bonnet removed for repair.
    cx, cz = 19.1, 44.5
    box("Sedan floor pan", (cx,.68,cz), (2.15,.25,4.45), dark, .08)
    box("Sedan lower body", (cx,.91,cz), (2.28,.63,4.35), body, .20)
    box("Sedan front guards", (cx,1.10,cz+1.47), (2.38,.52,1.15), body, .19)
    box("Sedan rear boot", (cx,1.05,cz-1.48), (2.34,.48,1.16), body, .18)
    box("Sedan cabin shell", (cx,1.58,cz-.12), (1.82,.95,2.20), body, .20)
    box("Front windscreen", (cx,1.68,cz+.985), (1.58,.64,.045), glass, .025, yaw=.02)
    box("Rear windscreen", (cx,1.68,cz-1.24), (1.55,.62,.045), glass, .025, yaw=-.02)
    for side in (-1,1):
        box("Sedan side window", (cx+side*.923,1.72,cz-.10), (.035,.53,1.74), glass, .02)
        box("Sedan faded chrome beltline", (cx+side*1.15,1.06,cz), (.035,.045,3.55), chrome, .008)
        for z in (cz-1.32,cz+1.34):
            wheel = bpy.data.objects.new("Restoration sedan tyre", bpy.data.meshes.new("Tyre mesh"))
            bpy.context.collection.objects.link(wheel)
            wheel.location = game_to_blender((cx+side*1.13,.58,z))
            # Cylinder's local Z becomes world X; align to car's side.
            bpy.ops.mesh.primitive_torus_add(major_radius=.36, minor_radius=.11, major_segments=20,
                                             minor_segments=8, location=wheel.location)
            tor = bpy.context.object
            tor.name = "Restoration sedan tyre"
            tor.data.materials.append(rubber)
            tor.rotation_euler[1] = pi/2
            bpy.data.objects.remove(wheel, do_unlink=True)
            hub = bpy.ops.mesh.primitive_cylinder_add(vertices=16, radius=.21, depth=.12,
                                                      location=game_to_blender((cx+side*1.13,.58,z)))
            rim = bpy.context.object
            rim.name = "Cloudy steel wheel hub"
            rim.data.materials.append(chrome)
            rim.rotation_euler[1] = pi/2
    # Removed bonnet resting on trestles, with hand-sanded primer patches and visible sanding swirls.
    for dz in (-.55,.55):
        box("Bonnet trestle leg", (16.25,.48,cz+2.05+dz), (.10,.85,.10), timber, .015, yaw=.08)
        box("Bonnet trestle foot", (16.25,.08,cz+2.05+dz), (.75,.10,.16), timber, .02)
    box("Bonnet trestle crossbar", (16.25,.80,cz+2.05), (.82,.12,.12), timber, .02)
    box("Detached sedan bonnet", (16.25,.92,cz+2.05), (2.0,.17,1.22), primer, .10, yaw=-.025)
    for i in range(7):
        # Narrow primer/sanding strips catch light at a gameplay camera distance.
        box("Bonnet sanding pass", (15.38+i*.28,1.014,cz+2.05), (.055,.012,.95),
            rust if i in (1,5) else body, .01, yaw=-.025)
    for side in (-1,1):
        box("Primer masking paper", (cx+side*1.2,.96,cz+.53), (.035,.40,.52), primer, .018)
        for z in (cz+1.76,cz-1.78):
            box("Chrome bumper", (cx,.72,z), (2.42,.16,.12), chrome, .035)
            for dx in (-.76,.76):
                box("Bumper rubber overrider", (cx+dx,.73,z), (.12,.20,.18), rubber, .025)
    # A few workshop clues; compact so the car remains the hero object.
    box("Steel tool chest", (22.4,.48,47.4), (.78,.82,.48), red, .06)
    for y in (.29,.53,.76):
        box("Tool chest drawer", (22.4,y,47.15), (.65,.19,.035), primer, .02)
        box("Drawer pull", (22.4,y,47.125), (.12,.025,.025), chrome, .01)
    box("Old oil tray", (22.4,.06,43.1), (.78,.12,.52), dark, .025)
    for x in (18.35,19.0,19.65):
        rod("Carport hanging bulb cord", (x,3.96,42.2), (x,3.15,42.2), .012, dark, 5)
    # Small garage door faces and a rear lane streetlight complete the neighbouring context.
    for x, z in ((-6.0,49.5),(6.0,49.5)):
        box("Rear lane garage opening", (x,1.55,z), (5.2,3.1,.16), dark, .03)
        box("Garage roller door", (x,1.55,z-.10), (4.85,2.8,.11), zinc, .03)
        for y in (.42,.74,1.06,1.38,1.70,2.02,2.34,2.66):
            box("Garage door rib", (x,y,z-.17), (4.78,.027,.035), trim, .007)
        box("Garage door handle", (x,.62,z-.23), (.28,.045,.03), dark, .012)
    box("Streetlight pole", (-.5,3.8,54.5), (.12,7.6,.12), zinc, .02)
    rod("Streetlight outreach", (-.5,7.5,54.5), (.25,7.5,54.5), .07, zinc, 8)
    box("Streetlight shade", (.35,7.40,54.5), (.55,.16,.28), zinc, .06)
    box("Streetlight warm lens", (.35,7.30,54.5), (.38,.06,.20), material("Warm street lamp", (.95,.72,.37), .35), .035)


original_scene = bpy.context.window.scene if bpy.context.window else bpy.context.scene
temp_scene = bpy.data.scenes.new("Neighbourhood Vignette Export")
try:
    if not bpy.context.window:
        raise RuntimeError("Blender window unavailable; leaving artist scene untouched")
    bpy.context.window.scene = temp_scene
    build()
    meshes = [o for o in temp_scene.objects if o.type == "MESH"]
    for obj in meshes:
        bpy.ops.object.select_all(action="DESELECT")
        obj.select_set(True)
        bpy.context.view_layer.objects.active = obj
        for mod in list(obj.modifiers):
            bpy.ops.object.modifier_apply(modifier=mod.name)
    bpy.ops.object.select_all(action="DESELECT")
    for obj in meshes:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = meshes[0]
    bpy.ops.object.join()
    OUT.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.export_scene.gltf(filepath=str(OUT), export_format="GLB", export_yup=True,
                              use_selection=True, use_active_scene=True, export_apply=True,
                              export_cameras=False, export_lights=False)
    print(f"Exported {OUT} with {len(meshes)} authored mesh components, joined into one scene object")
finally:
    bpy.context.window.scene = original_scene
    bpy.data.scenes.remove(temp_scene)
