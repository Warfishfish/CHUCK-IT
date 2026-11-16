"""Create the original Heist turf tile and Smoko roadside prop cluster in Blender.

Run from the live Blender MCP session or with Blender's bundled Python. The script makes its
own temporary scene, exports only its authored data, then restores the user's original scene.
"""

import array
import math
import os
import random

import bpy
from mathutils import Vector


ROOT = "/Users/marcus/Claude/australian-bbq"
ASSET_DIR = os.path.join(ROOT, "rust/crates/bbq_app/assets")
TURF_PATH = os.path.join(ASSET_DIR, "textures/heist_turf.png")
GLB_PATH = os.path.join(ASSET_DIR, "models/smoko_roadside_cluster.glb")


def make_heist_turf():
    """A soft, seamless olive field turf with directional close-cut fibers."""
    size = 512
    image = bpy.data.images.new("Heist Turf - seamless", width=size, height=size, alpha=False)
    image.colorspace_settings.name = "sRGB"
    pixels = array.array("f")

    def noise(x, y):
        n = (x * 374761393 + y * 668265263) & 0xFFFFFFFF
        n = ((n ^ (n >> 13)) * 1274126177) & 0xFFFFFFFF
        return ((n ^ (n >> 16)) & 0xFFFFFFFF) / 4294967295.0

    for y in range(size):
        yf = y / size
        for x in range(size):
            xf = x / size
            grain = noise(x, y) - 0.5
            broad = math.sin((xf * 2.0 + yf * 0.65) * math.tau) * 0.020
            patches = math.sin((xf * 5.0 + yf * 3.0) * math.tau) * 0.016
            fiber = max(0.0, math.sin((x + y * 0.32) * 0.56 + grain * 2.8)) ** 9 * 0.060
            v = grain * 0.070 + broad + patches + fiber
            pixels.extend((
                max(0.0, min(1.0, 0.165 + v * 0.75)),
                max(0.0, min(1.0, 0.270 + v)),
                max(0.0, min(1.0, 0.145 + v * 0.60)),
                1.0,
            ))

    image.pixels.foreach_set(pixels)
    image.filepath_raw = TURF_PATH
    image.file_format = "PNG"
    image.save()
    bpy.data.images.remove(image)


def build_cluster():
    mats = {}

    def material(name, color, metallic=0.0, roughness=0.78):
        if name in mats:
            return mats[name]
        m = bpy.data.materials.new(name)
        m.diffuse_color = (*color, 1.0)
        m.use_nodes = True
        bsdf = next(n for n in m.node_tree.nodes if n.type == "BSDF_PRINCIPLED")
        bsdf.inputs["Base Color"].default_value = (*color, 1.0)
        bsdf.inputs["Metallic"].default_value = metallic
        bsdf.inputs["Roughness"].default_value = roughness
        mats[name] = m
        return m

    rubber = material("sun-baked charcoal rubber", (0.045, 0.050, 0.044))
    rubber_edge = material("rubber sidewall", (0.085, 0.092, 0.078))
    wood = material("old red-gum timber", (0.31, 0.19, 0.105))
    wood_light = material("sun-faded rail timber", (0.53, 0.36, 0.19))
    rust = material("rusted steel", (0.20, 0.12, 0.075), metallic=0.52, roughness=0.72)
    cream = material("sun-faded sign enamel", (0.82, 0.72, 0.52), metallic=0.12, roughness=0.58)
    ink = material("bottle-green sign paint", (0.055, 0.17, 0.12), metallic=0.08, roughness=0.66)
    blue = material("faded esky blue", (0.12, 0.37, 0.55), metallic=0.04, roughness=0.47)
    white = material("chalky esky lid", (0.76, 0.80, 0.73), metallic=0.02, roughness=0.53)
    orange = material("old safety orange", (0.78, 0.29, 0.075), metallic=0.08, roughness=0.66)

    created = []

    def box(name, dims, loc, mat, bevel=0.025):
        bpy.ops.mesh.primitive_cube_add(size=1.0, location=loc)
        obj = bpy.context.object
        obj.name = name
        obj.dimensions = dims
        bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
        obj.data.materials.append(mat)
        if bevel > 0:
            mod = obj.modifiers.new("small worn edges", "BEVEL")
            mod.width = bevel
            mod.segments = 2
            bpy.context.view_layer.objects.active = obj
            bpy.ops.object.modifier_apply(modifier=mod.name)
            mod = obj.modifiers.new("weighted corner normals", "WEIGHTED_NORMAL")
            bpy.ops.object.modifier_apply(modifier=mod.name)
        created.append(obj)
        return obj

    def cylinder(name, radius, depth, loc, mat, vertices=12, axis=None):
        bpy.ops.mesh.primitive_cylinder_add(vertices=vertices, radius=radius, depth=depth, location=loc)
        obj = bpy.context.object
        obj.name = name
        if axis is not None:
            obj.rotation_quaternion = Vector((0, 0, 1)).rotation_difference(Vector(axis).normalized())
        obj.data.materials.append(mat)
        mod = obj.modifiers.new("soft edge", "BEVEL")
        mod.width = min(0.025, radius * 0.12)
        mod.segments = 2
        bpy.context.view_layer.objects.active = obj
        bpy.ops.object.modifier_apply(modifier=mod.name)
        created.append(obj)
        return obj

    def torus(name, major, minor, loc, mat):
        bpy.ops.mesh.primitive_torus_add(
            major_radius=major, minor_radius=minor, major_segments=32, minor_segments=10, location=loc
        )
        obj = bpy.context.object
        obj.name = name
        obj.data.materials.append(mat)
        for face in obj.data.polygons:
            face.use_smooth = True
        created.append(obj)
        return obj

    def beam(name, start, end, width, depth, mat):
        start, end = Vector(start), Vector(end)
        delta = end - start
        obj = box(name, (width, depth, delta.length), (start + end) * 0.5, mat, 0.015)
        obj.rotation_mode = "QUATERNION"
        obj.rotation_quaternion = Vector((0, 0, 1)).rotation_difference(delta.normalized())
        return obj

    # A sun-faded roadside marker with large, readable lettering.
    cylinder("sign post", 0.065, 1.75, (-2.45, 0.10, 0.88), rust, 10)
    box("sign board", (1.38, 0.105, 0.82), (-2.34, 0.10, 1.43), cream, 0.055)
    box("sign inset", (1.19, 0.025, 0.62), (-2.34, 0.035, 1.43), ink, 0.018)
    for z in (1.12, 1.74):
        cylinder("forged sign bolt", 0.035, 0.035, (-2.34, 0.012, z), rust, 10, axis=(0, 1, 0))
    # Centre the whole word on the board; the previous half-metre offset clipped the first
    # letters behind the post in an end-on view.
    bpy.ops.object.text_add(location=(-2.34, 0.008, 1.50), rotation=(math.pi / 2, 0, 0))
    lettering = bpy.context.object
    lettering.name = "hand-painted TYRES lettering"
    lettering.data.body = "TYRES"
    lettering.data.align_x = "CENTER"
    lettering.data.align_y = "CENTER"
    lettering.data.size = 0.30
    lettering.data.extrude = 0.004
    lettering.data.materials.append(cream)
    bpy.ops.object.convert(target="MESH")
    created.append(bpy.context.object)

    # A stack of chunky, deeply grooved tyres. Treads are grouped into the final static mesh.
    tire_x, tire_y = -0.70, 0.05
    for row in range(3):
        z = 0.18 + row * 0.35
        torus("stacked worn tyre", 0.48, 0.145, (tire_x, tire_y, z), rubber if row != 1 else rubber_edge)
        torus("moulded tyre bead", 0.375, 0.028, (tire_x, tire_y, z), rubber_edge)
        for k in range(18):
            a = k * math.tau / 18 + row * 0.08
            x = tire_x + math.cos(a) * 0.60
            y = tire_y + math.sin(a) * 0.60
            tread = box("raised tyre tread", (0.17, 0.07, 0.23), (x, y, z), rubber_edge, 0.025)
            tread.rotation_euler[2] = a

    # A rough timber parts crate, with slats, nail heads and an old orange stencil.
    crate_x, crate_y = 0.82, -0.68
    box("crate core", (0.96, 0.78, 0.70), (crate_x, crate_y, 0.40), wood, 0.03)
    for side in (-1, 1):
        y = crate_y + side * 0.402
        for i in range(5):
            z = 0.13 + i * 0.135
            box("crate face slat", (0.94, 0.045, 0.105), (crate_x, y, z), wood_light if i == 2 else wood, 0.012)
        for x in (crate_x - 0.39, crate_x + 0.39):
            for z in (0.12, 0.68):
                cylinder("crate nail", 0.018, 0.018, (x, y + side * 0.027, z), rust, 8, axis=(0, 1, 0))
    for x in (crate_x - 0.49, crate_x + 0.49):
        for z in (0.2, 0.6):
            box("crate corner strap", (0.055, 0.81, 0.06), (x, crate_y, z), rust, 0.009)
    box("orange stencil patch", (0.36, 0.025, 0.16), (crate_x, crate_y - 0.416, 0.40), orange, 0.012)

    # A faded blue esky with a cream lid, moulded ribs, carry grip and two catches.
    esky_x, esky_y = 2.05, 0.32
    box("esky body", (0.98, 0.72, 0.48), (esky_x, esky_y, 0.28), blue, 0.10)
    box("esky lid gasket", (1.00, 0.74, 0.045), (esky_x, esky_y, 0.525), rust, 0.04)
    box("esky lid", (1.04, 0.78, 0.11), (esky_x, esky_y, 0.59), white, 0.06)
    for dx in (-0.35, -0.15, 0.15, 0.35):
        box("lid moulded rib", (0.035, 0.55, 0.018), (esky_x + dx, esky_y, 0.652), blue, 0.009)
    for x in (esky_x - 0.32, esky_x + 0.32):
        box("esky latch", (0.12, 0.045, 0.12), (x, esky_y - 0.375, 0.49), orange, 0.02)
    beam("esky carry grip", (esky_x - 0.18, esky_y + 0.41, 0.40), (esky_x + 0.18, esky_y + 0.41, 0.40), 0.07, 0.07, white)

    # Two rough rails and split posts make this read as roadside protection, not a fence wall.
    for x in (-2.65, 2.70):
        box("rail post", (0.17, 0.20, 0.92), (x, 1.86, 0.48), wood, 0.025)
        box("post foot plate", (0.27, 0.28, 0.08), (x, 1.86, 0.05), rust, 0.015)
        for z in (0.50, 0.84):
            cylinder("rail bolt", 0.026, 0.035, (x, 1.74, z), rust, 10, axis=(0, 1, 0))
    beam("upper split timber rail", (-2.70, 1.86, 0.80), (2.76, 1.86, 0.80), 0.17, 0.18, wood_light)
    beam("lower timber rail", (-2.70, 1.86, 0.43), (2.76, 1.86, 0.43), 0.13, 0.15, wood)
    # A worn diagonal repair brace and a couple of old repair plates tell a small material story.
    beam("rail repair brace", (-1.00, 1.74, 0.44), (-0.54, 1.74, 0.78), 0.07, 0.055, wood)
    box("rail repair plate", (0.22, 0.035, 0.10), (1.95, 1.735, 0.82), rust, 0.012)
    for x in (1.89, 2.01):
        cylinder("repair plate screw", 0.015, 0.018, (x, 1.708, 0.82), cream, 8, axis=(0, 1, 0))

    # Keep one light, multi-material mesh in the runtime instead of dozens of loose objects.
    bpy.ops.object.select_all(action="DESELECT")
    for obj in created:
        if obj.type == "MESH":
            obj.select_set(True)
    active = next(obj for obj in created if obj.type == "MESH")
    bpy.context.view_layer.objects.active = active
    bpy.ops.object.join()
    joined = bpy.context.object
    joined.name = "Smoko roadside tyre esky and rail cluster"
    joined.data.name = "Smoko roadside prop cluster mesh"
    joined.location = (0.0, 0.0, 0.0)
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    bpy.ops.export_scene.gltf(
        filepath=GLB_PATH,
        export_format="GLB",
        use_selection=True,
        export_apply=True,
        export_yup=True,
    )


def main():
    os.makedirs(os.path.dirname(TURF_PATH), exist_ok=True)
    os.makedirs(os.path.dirname(GLB_PATH), exist_ok=True)
    window = bpy.context.window
    original_scene = window.scene
    temp_scene = bpy.data.scenes.new("Codex Temporary Visual Pass Assets")
    try:
        window.scene = temp_scene
        make_heist_turf()
        build_cluster()
        print("Created", TURF_PATH, "and", GLB_PATH)
    finally:
        window.scene = original_scene
        bpy.data.scenes.remove(temp_scene)


if __name__ == "__main__":
    main()
