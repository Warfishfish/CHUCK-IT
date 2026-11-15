"""Create the backyard overflow channel, pool shower and open side gate GLBs.

Run from the repository root with Blender's Python console, or via MCP for Blender.
The script builds in a temporary scene and restores the artist's open scene afterwards.
All coordinates use the game's axes (+Y up, +Z toward the house front); GLB export converts
the assets to glTF's Y-up convention. No downloaded or licensed assets are used.
"""
from math import cos, pi, sin
from pathlib import Path

import bpy
from mathutils import Vector


OUT = Path(__file__).resolve().parents[1] / "crates/bbq_app/assets/models"
KEEP_PREVIEW = bool(globals().get("KEEP_PREVIEW", False))


def to_blender(p):
    return (p[0], -p[2], p[1])


def make_material(name, rgb, roughness=0.82, metallic=0.0):
    material = bpy.data.materials.new(name)
    material.diffuse_color = (*rgb, 1.0)
    material.use_nodes = True
    shader = next(n for n in material.node_tree.nodes if n.type == "BSDF_PRINCIPLED")
    shader.inputs["Base Color"].default_value = (*rgb, 1.0)
    shader.inputs["Roughness"].default_value = roughness
    shader.inputs["Metallic"].default_value = metallic
    return material


def mesh_object(name, vertices, faces, material, smooth=False):
    mesh = bpy.data.meshes.new(name + " mesh")
    mesh.from_pydata([to_blender(v) for v in vertices], [], faces)
    mesh.materials.append(material)
    mesh.update()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    if smooth:
        for polygon in mesh.polygons:
            polygon.use_smooth = True
    return obj


def box(name, center, size, material, yaw=0.0, bevel=0.0):
    cx, cy, cz = center
    sx, sy, sz = size
    verts = []
    for y in (-sy / 2, sy / 2):
        for z in (-sz / 2, sz / 2):
            for x in (-sx / 2, sx / 2):
                rx = x * cos(yaw) + z * sin(yaw)
                rz = -x * sin(yaw) + z * cos(yaw)
                verts.append((cx + rx, cy + y, cz + rz))
    faces = [(0, 1, 3, 2), (4, 6, 7, 5), (0, 4, 5, 1),
             (2, 3, 7, 6), (0, 2, 6, 4), (1, 5, 7, 3)]
    obj = mesh_object(name, verts, faces, material)
    if bevel:
        modifier = obj.modifiers.new("Soft worn edges", "BEVEL")
        modifier.width = bevel
        modifier.segments = 2
        obj.modifiers.new("Weighted corner normals", "WEIGHTED_NORMAL")
    return obj


def cylinder(name, a, b, radius, material, segments=10, radius_b=None):
    a, b = Vector(a), Vector(b)
    axis = (b - a).normalized()
    helper = Vector((1, 0, 0)) if abs(axis.dot(Vector((0, 1, 0)))) > 0.96 else Vector((0, 1, 0))
    u = axis.cross(helper).normalized()
    v = axis.cross(u).normalized()
    rb = radius if radius_b is None else radius_b
    verts = []
    for centre, r in ((a, radius), (b, rb)):
        verts.extend(tuple(centre + r * (u * cos(i * 2*pi/segments) + v * sin(i * 2*pi/segments)))
                     for i in range(segments))
    faces = [tuple(reversed(range(segments))), tuple(range(segments, 2*segments))]
    faces += [(i, (i+1) % segments, (i+1) % segments + segments, i + segments)
              for i in range(segments)]
    return mesh_object(name, verts, faces, material, smooth=segments >= 12)


def sphere(name, center, radii, material, segments=12, rings=8):
    cx, cy, cz = center
    rx, ry, rz = radii
    verts = [(cx, cy + ry, cz)]
    for j in range(1, rings):
        phi = pi * j / rings
        for i in range(segments):
            theta = 2*pi*i/segments
            verts.append((cx + rx*sin(phi)*cos(theta), cy + ry*cos(phi), cz + rz*sin(phi)*sin(theta)))
    verts.append((cx, cy - ry, cz))
    south = len(verts) - 1
    faces = [(0, 1 + i, 1 + (i+1) % segments) for i in range(segments)]
    for j in range(rings - 2):
        start, nxt = 1 + j*segments, 1 + (j+1)*segments
        faces.extend((start+i, nxt+i, nxt+(i+1) % segments, start+(i+1) % segments) for i in range(segments))
    last = 1 + (rings - 2)*segments
    faces.extend((last+i, south, last+(i+1) % segments) for i in range(segments))
    return mesh_object(name, verts, faces, material, smooth=True)


def build_overflow(m):
    # A shallow, branching garden runnel continues from the tank outlet. Its low stones
    # and water are deliberately decorative and do not create a gameplay barrier.
    box("Rain garden dark earth", (2.86, 0.025, 1.04), (5.82, 0.045, 0.58), m["earth"], bevel=0.025)
    box("Still water in the runnel", (2.88, 0.055, 1.04), (5.55, 0.018, 0.19), m["water"], bevel=0.025)
    # Split stone kerb: a short lead-in, a dog-leg and a long shallow channel.
    for side in (-1, 1):
        for i in range(18):
            x = -0.12 + i * 0.34
            z = 0.74 if side < 0 else 1.34
            wobble = 0.025 * sin(i * 1.7 + side)
            material = m["stone_a"] if i % 3 else m["stone_b"]
            stone = sphere("Rounded creek stone", (x, 0.09, z + wobble), (0.18, 0.09, 0.12), material, 9, 6)
            stone.rotation_euler[2] = (i % 4 - 1.5) * 0.09
    # The little inlet visibly catches the tank outlet before turning into the channel.
    box("Outlet splash pad", (0.0, 0.032, 0.23), (0.46, 0.026, 0.54), m["water"], bevel=0.035)
    for i in range(3):
        x = 5.1 + i * 0.38
        for j in range(3):
            z = 0.79 + j * 0.24
            stem_x = x + (j - 1) * 0.09
            cylinder("Rain garden sedge", (stem_x, 0.04, z), (stem_x + (j-1)*0.04, 0.48 + 0.07*(i%2), z),
                     0.018, m["reed"], 6, 0.006)
    # A small grate at the start makes the connection to the downpipe legible.
    box("Leaf guard", (0.0, 0.075, 0.45), (0.36, 0.018, 0.17), m["metal"], bevel=0.012)
    for x in (-0.11, -0.04, 0.04, 0.11):
        box("Grate slot", (x, 0.087, 0.45), (0.018, 0.006, 0.145), m["darkmetal"])


def build_shower(m):
    # Freestanding poolside rinse station: small tiled pad, galvanized pipework,
    # coppery old taps, a drain and a faded towel. The shower itself is non-solid.
    box("Faded tiled shower pad", (0.0, 0.055, 0.0), (1.16, 0.11, 1.22), m["tilebase"], bevel=0.045)
    for x in (-0.39, 0.0, 0.39):
        for z in (-0.40, 0.0, 0.40):
            box("Sun-faded blue tile", (x, 0.114, z), (0.36, 0.012, 0.37),
                m["tileblue"] if (round((x+z)*10) % 2 == 0) else m["tilecream"], bevel=0.006)
    # Drain grate at the front edge of the pad.
    box("Square shower drain", (0.34, 0.126, 0.43), (0.14, 0.012, 0.14), m["darkmetal"])
    for offset in (-0.04, 0.0, 0.04):
        box("Drain slot", (0.34 + offset, 0.134, 0.43), (0.012, 0.004, 0.115), m["metal"])
    cylinder("Shower riser", (0.0, 0.12, -0.31), (0.0, 2.20, -0.31), 0.045, m["metal"], 12)
    cylinder("Wall fixing lower", (-0.09, 0.35, -0.31), (0.09, 0.35, -0.31), 0.025, m["darkmetal"], 8)
    cylinder("Wall fixing upper", (-0.09, 1.94, -0.31), (0.09, 1.94, -0.31), 0.025, m["darkmetal"], 8)
    cylinder("Shower arm", (0.0, 2.18, -0.31), (0.36, 2.18, -0.31), 0.035, m["metal"], 10)
    cylinder("Shower head neck", (0.36, 2.18, -0.31), (0.36, 2.02, -0.31), 0.03, m["metal"], 10)
    cylinder("Old shower rose", (0.36, 2.015, -0.31), (0.36, 1.99, -0.31), 0.16, m["darkmetal"], 16)
    cylinder("Shower rose face", (0.36, 1.985, -0.31), (0.36, 1.972, -0.31), 0.13, m["metal"], 16)
    for a in range(5):
        for b in range(5):
            sphere("Shower nozzle", (0.36 + (a-2)*0.045, 1.965, -0.31 + (b-2)*0.045),
                   (0.009, 0.008, 0.009), m["darkmetal"], 6, 4)
    # Two separate taps with cross handles and copper patina at the wall fixing.
    for x in (-0.19, 0.19):
        cylinder("Aged copper tap stem", (x, 0.87, -0.30), (x, 0.87, -0.46), 0.045, m["copper"], 10)
        cylinder("Tap handle hub", (x, 0.87, -0.46), (x, 0.87, -0.51), 0.055, m["copper_dark"], 10)
        cylinder("Tap cross handle", (x-0.10, 0.87, -0.52), (x+0.10, 0.87, -0.52), 0.022, m["copper"], 8)
        sphere("Tap knob cap", (x, 0.87, -0.52), (0.055, 0.055, 0.035), m["copper"], 8, 6)
    # Bent towel hook and a striped, sun-faded towel hanging beside the riser.
    cylinder("Towel hook stem", (0.08, 1.63, -0.31), (0.43, 1.63, -0.31), 0.018, m["metal"], 8)
    cylinder("Towel hook bend", (0.43, 1.63, -0.31), (0.43, 1.55, -0.31), 0.018, m["metal"], 8)
    box("Old striped pool towel", (0.63, 1.08, -0.30), (0.40, 0.95, 0.045), m["towel"], yaw=-0.06, bevel=0.025)
    for y in (0.73, 1.08, 1.43):
        box("Faded towel stripe", (0.63, y, -0.272), (0.40, 0.055, 0.008), m["towelstripe"], yaw=-0.06)
    # A faint chalky splash ring and a wall-side drain pipe make the pad feel used.
    cylinder("Shower waste outlet", (0.34, 0.13, 0.43), (0.34, 0.17, 0.43), 0.045, m["darkmetal"], 12)
    for x in (-0.46, 0.46):
        box("Tile grout shadow", (x, 0.119, 0.0), (0.012, 0.003, 1.12), m["grout"])


def build_gate(m):
    # Two matching brick piers leave a 1.5 m player opening. The gate is visibly
    # swung into the yard along +Z, exactly where the rules leave the collision gap.
    for side in (-1, 1):
        cx = side * 1.5
        box("Mortar wall core", (cx, 0.54, 0.0), (1.50, 1.08, 0.35), m["mortar"], bevel=0.025)
        for row in range(5):
            y = 0.11 + row * 0.205
            offset = 0.18 if row % 2 else 0.0
            for j in range(4):
                x = cx - 0.57 + j * 0.38 + offset
                if x < cx - 0.72 or x > cx + 0.72:
                    continue
                material = m["brick_a"] if (row + j) % 3 else m["brick_b"]
                box("Hand-laid garden brick", (x, y, -0.012), (0.34, 0.175, 0.365), material,
                    yaw=(j % 2 - 0.5) * 0.012, bevel=0.018)
        box("Coping brick", (cx, 1.11, 0.0), (1.57, 0.12, 0.43), m["cap"], bevel=0.035)
        box("Weathered timber gate post", (side * 0.75, 0.63, 0.0), (0.14, 1.27, 0.16), m["timber"], bevel=0.025)
        box("Gate post cap", (side * 0.75, 1.30, 0.0), (0.22, 0.08, 0.23), m["timberlight"], bevel=0.025)
    # Five close-set boards follow the open leaf, viewed from either side of the shortcut.
    hinge_x, zmid = -0.75, 0.62
    for i in range(5):
        z = 0.10 + i * 0.25
        lean = (i - 2) * 0.012
        box("Gate paling", (hinge_x + lean, 0.62 + (i % 2) * 0.012, z),
            (0.075, 0.98, 0.215), m["timberlight"] if i in (0, 4) else m["timber"],
            yaw=0.025 * (i - 2), bevel=0.012)
    # Rails are also turned edge-on along the swung leaf, with a crooked diagonal brace.
    for y in (0.20, 1.00):
        box("Gate cross rail", (hinge_x, y, zmid), (0.095, 0.115, 1.24), m["timber"], bevel=0.018)
    # The open-brace runs up from the hinge to the far latch corner.
    cylinder("Crooked gate diagonal brace", (hinge_x-0.015, 0.25, 0.12),
             (hinge_x-0.015, 0.94, 1.12), 0.048, m["timberlight"], 6, 0.038)
    # Dark iron hinge straps reach from the post to the gate leaf.
    for y in (0.28, 0.93):
        box("Forged gate hinge strap", (-0.75, y, 0.10), (0.16, 0.065, 0.50), m["iron"], bevel=0.012)
        cylinder("Gate hinge pin", (-0.78, y-0.08, -0.012), (-0.78, y+0.08, -0.012),
                 0.035, m["iron"], 8)
    # Bright latch is a readable landmark on the free edge of the open gate.
    box("Bright brass gate latch", (-0.75, 0.82, 1.13), (0.19, 0.12, 0.12), m["latch"], bevel=0.03)
    cylinder("Latch handle", (-0.75, 0.82, 1.11), (-0.75, 0.82, 0.98), 0.025, m["latchdark"], 8)
    for y in (0.25, 0.94):
        sphere("Gate iron nail", (-0.75, y, 0.13), (0.035, 0.035, 0.018), m["ironlight"], 8, 6)


def materials():
    return {
        "earth": make_material("Creek dark damp earth", (0.20, 0.18, 0.13)),
        "water": make_material("Shallow rainwater", (0.22, 0.47, 0.48), 0.24, 0.12),
        "stone_a": make_material("Warm creek stone", (0.48, 0.45, 0.37)),
        "stone_b": make_material("Pale creek stone", (0.60, 0.56, 0.46)),
        "reed": make_material("Rain garden sedge", (0.34, 0.42, 0.24)),
        "metal": make_material("Pool shower galvanised steel", (0.48, 0.53, 0.50), 0.34, 0.70),
        "darkmetal": make_material("Dark shower fittings", (0.20, 0.24, 0.22), 0.48, 0.55),
        "tilebase": make_material("Old pale pool tile bed", (0.69, 0.69, 0.58)),
        "tileblue": make_material("Faded aqua tiles", (0.29, 0.55, 0.60)),
        "tilecream": make_material("Sun-faded cream tiles", (0.77, 0.72, 0.58)),
        "grout": make_material("Tile grout", (0.48, 0.48, 0.40)),
        "copper": make_material("Oxidised copper taps", (0.57, 0.34, 0.22), 0.5, 0.65),
        "copper_dark": make_material("Dark copper patina", (0.27, 0.40, 0.31), 0.55, 0.42),
        "towel": make_material("Faded pool towel", (0.74, 0.58, 0.38)),
        "towelstripe": make_material("Towel pale stripes", (0.88, 0.79, 0.61)),
        "mortar": make_material("Aged garden mortar", (0.56, 0.51, 0.43)),
        "brick_a": make_material("Old red garden brick", (0.49, 0.23, 0.16)),
        "brick_b": make_material("Sun-faded garden brick", (0.58, 0.31, 0.22)),
        "cap": make_material("Worn brick coping", (0.61, 0.36, 0.27)),
        "timber": make_material("Weathered fence timber", (0.43, 0.31, 0.20)),
        "timberlight": make_material("Sun-bleached gate boards", (0.59, 0.45, 0.29)),
        "iron": make_material("Old gate iron", (0.18, 0.20, 0.18), 0.52, 0.56),
        "ironlight": make_material("Worn iron heads", (0.42, 0.39, 0.31), 0.48, 0.52),
        "latch": make_material("Bright brass gate latch", (0.93, 0.63, 0.13), 0.36, 0.55),
        "latchdark": make_material("Latch stem", (0.56, 0.35, 0.10), 0.45, 0.5),
    }


original_scene = bpy.context.window.scene if bpy.context.window else bpy.context.scene
temp_scene = bpy.data.scenes.new("Backyard Props Export")
try:
    if bpy.context.window:
        bpy.context.window.scene = temp_scene
    else:
        raise RuntimeError("Blender window is unavailable; refusing to modify the open scene")
    mats = materials()
    builders = [
        ("backyard_overflow.glb", "Backyard rainwater overflow", build_overflow),
        ("poolside_shower.glb", "Poolside outdoor shower", build_shower),
        ("side_gate.glb", "Open brick side gate", build_gate),
    ]
    OUT.mkdir(parents=True, exist_ok=True)
    for filename, name, build in builders:
        # Clear only the temporary export scene between independent assets.
        for obj in list(temp_scene.objects):
            bpy.data.objects.remove(obj, do_unlink=True)
        build(mats)
        bpy.ops.object.select_all(action="DESELECT")
        for obj in temp_scene.objects:
            obj.select_set(True)
        bpy.context.view_layer.objects.active = next(iter(temp_scene.objects), None)
        path = str(OUT / filename)
        bpy.ops.export_scene.gltf(filepath=path, export_format="GLB", export_yup=True,
                                  use_selection=True, use_active_scene=True, export_apply=True,
                                  export_cameras=False, export_lights=False)
        print(f"Exported {name}: {path} ({len(temp_scene.objects)} meshes)")
        if KEEP_PREVIEW and filename == "side_gate.glb":
            break
    if KEEP_PREVIEW:
        print("Preview scene left active temporarily; restore with the cleanup call before finishing.")
finally:
    if not KEEP_PREVIEW:
        if bpy.context.window:
            bpy.context.window.scene = original_scene
        bpy.data.scenes.remove(temp_scene)
