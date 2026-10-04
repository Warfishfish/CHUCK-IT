"""Builds the blob character in Blender (headless) and exports rust/crates/bbq_app/assets/models/blob_<name>.glb, one file each.

Run from the rust/ folder:   python3 tools/make_blob.py
(needs `pip install bpy`, Blender as a Python module). Open the .glb in Blender to tweak it.

Shapes and sizes are the same as the JavaScript game's createChar():
 capsule body, lighter head, white eyes + black pupils, flat dark feet, two lighter hands.
Blender is Z-up; the exporter turns that into the game's Y-up, so a point (x, up, forward)
is written to Blender as (x, -forward, up).
"""
import bpy, bmesh, math, os, sys

OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "crates", "bbq_app", "assets", "models", "blob.glb")
PREVIEW = sys.argv[sys.argv.index("--preview") + 1] if "--preview" in sys.argv else None

bpy.ops.wm.read_factory_settings(use_empty=True)
scn = bpy.context.scene


def mat(name, rgb):
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    b = m.node_tree.nodes["Principled BSDF"]
    b.inputs["Base Color"].default_value = (*rgb, 1)
    b.inputs["Roughness"].default_value = 0.55
    return m


# Base colours are plain grey: the game tints body / head / feet / hands per player.
M = {
    "Body": mat("BodyMat", (0.8, 0.8, 0.8)),
    "Head": mat("HeadMat", (0.9, 0.9, 0.9)),
    "Foot": mat("FootMat", (0.5, 0.5, 0.5)),
    "White": mat("EyeWhite", (1, 1, 1)),
    "Black": mat("Pupil", (0.07, 0.07, 0.07)),
}


def pos(x, up, fwd):
    return (x, -fwd, up)


# Body shapes. Radius at the top and at the bottom of the body, plus its height (the old capsule
# was 0.36 round everywhere and 1.42 tall). The body is egg-shaped: the radius changes smoothly
# from the bottom value to the top value, and both ends stay round.
SHAPES = {
    "classic": dict(bottom=0.37, top=0.37, height=1.42, capsule=True),  # the original capsule
    "pear": dict(bottom=0.47, top=0.31, height=1.42),     # gentle: round bottom, slimmer shoulders
    "egg": dict(bottom=0.45, top=0.26, height=1.46),      # stronger taper, a bit taller
    "gumdrop": dict(bottom=0.52, top=0.30, height=1.34),  # wide and low, the cutest
}
BODY_BASE = 0.07  # the body's lowest point sits just off the ground (same as the old capsule)


def body_radius(shape, up):
    """Half-width of the body at height `up` (used to place hands against it)."""
    sh = SHAPES[shape]
    u = (up - BODY_BASE) / sh["height"]
    return sh["top"] + (sh["bottom"] - sh["top"]) * (1 - u)


def egg_body(name, shape, material, dx=0.0, parent=None, seg=40, rings=33):
    sh = SHAPES[shape]
    bm = bmesh.new()
    bmesh.ops.create_uvsphere(bm, u_segments=seg, v_segments=rings, radius=1.0)
    for v in bm.verts:
        y = v.co.z                      # -1 (bottom) .. +1 (top) on the unit sphere
        u = (y + 1) / 2
        w = sh["top"] + (sh["bottom"] - sh["top"]) * (1 - u)
        v.co.x *= w
        v.co.y *= w
        v.co.z = BODY_BASE + u * sh["height"]
    bm.normal_update()
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    for p_ in me.polygons:
        p_.use_smooth = True
    ob = bpy.data.objects.new(name, me)
    ob.data.materials.append(material)
    ob.location = (dx, 0, 0)
    scn.collection.objects.link(ob)
    if parent:
        ob.parent = parent
        ob.matrix_parent_inverse = parent.matrix_world.inverted()
    return ob


def sphere(name, r, at, material, scale=(1, 1, 1), parent=None, seg=32, rings=17, capsule=0.0):
    bm = bmesh.new()
    bmesh.ops.create_uvsphere(bm, u_segments=seg, v_segments=rings, radius=r)
    if capsule:
        for v in bm.verts:
            v.co.z += capsule / 2 if v.co.z > 0 else -capsule / 2
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    for p in me.polygons:
        p.use_smooth = True
    ob = bpy.data.objects.new(name, me)
    ob.data.materials.append(material)
    ob.location = pos(*at)
    # scale is given as (x, up, fwd)
    ob.scale = (scale[0], scale[2], scale[1])
    scn.collection.objects.link(ob)
    if parent:
        ob.parent = parent
        ob.matrix_parent_inverse = parent.matrix_world.inverted()
    return ob


def make_blob(shape, dx=0.0):
    """One whole blob. Parts are named so the game can find them (Torso, Head, HandR, HandL...)."""
    body = bpy.data.objects.new("Body", None)
    body.location = (dx, 0, 0)
    scn.collection.objects.link(body)
    if SHAPES[shape].get("capsule"):
        sphere("Torso", 0.36, (0, 0.78, 0), M["Body"], parent=body, capsule=0.55)
    else:
        egg_body("Torso", shape, M["Body"], parent=body)
    sphere("Head", 0.30, (0, 1.50, 0), M["Head"], parent=body)
    for side, sg in (("L", -1), ("R", 1)):
        sphere("Eye" + side, 0.085, (sg * 0.11, 1.56, 0.24), M["White"], parent=body, seg=20, rings=11)
        sphere("Pupil" + side, 0.042, (sg * 0.11, 1.56, 0.315), M["Black"], parent=body, seg=16, rings=9)
        sphere("Foot" + side, 0.13, (sg * 0.2, 0.09, 0.05), M["Foot"], scale=(1, 0.6, 1.4), parent=body, seg=24, rings=13)
    # HandR is the one the game animates (it swings), HandL is the other side.
    # Hands rest just outside the body at arm height (0.88 up).
    hx = body_radius(shape, 0.88) + 0.1
    sphere("HandR", 0.10, (-hx, 0.88, 0.05), M["Head"], parent=body, seg=24, rings=13)
    sphere("HandL", 0.10, (hx, 0.88, 0.05), M["Head"], parent=body, seg=24, rings=13)
    return body


SHEET = "--sheet" in sys.argv
ORDER = ["classic", "pear", "egg", "gumdrop"]

if SHEET:
    for i, name in enumerate(ORDER):
        make_blob(name, dx=(i - 1.5) * 1.3)
elif not PREVIEW:
    pass  # exporting happens below, one file per character
else:
    make_blob("gumdrop")

if PREVIEW:
    # a quick picture so the model can be checked without a game window (CPU render)
    for ob in list(scn.objects):
        if ob.type == "MESH":
            ob.data.materials[0] = ob.data.materials[0]
    tint = {"BodyMat": (0.9, 0.2, 0.2), "HeadMat": (0.93, 0.5, 0.5), "FootMat": (0.54, 0.12, 0.12)}
    for m in bpy.data.materials:
        if m.name in tint:
            m.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = (*tint[m.name], 1)
    w = bpy.data.worlds.new("w"); w.use_nodes = True
    w.node_tree.nodes["Background"].inputs[0].default_value = (0.55, 0.75, 0.95, 1)
    scn.world = w
    sun = bpy.data.objects.new("sun", bpy.data.lights.new("sun", "SUN"))
    sun.data.energy = 4; sun.rotation_euler = (math.radians(50), 0, math.radians(-30))
    scn.collection.objects.link(sun)
    cam = bpy.data.objects.new("cam", bpy.data.cameras.new("cam"))
    cam.location = ((0.0, -8.2, 2.2) if SHEET else (2.0, -3.6, 1.7)); scn.collection.objects.link(cam)
    d = cam.constraints.new("TRACK_TO"); d.track_axis = "TRACK_NEGATIVE_Z"; d.up_axis = "UP_Y"
    t = bpy.data.objects.new("t", None); t.location = (0, 0, 0.85); scn.collection.objects.link(t); d.target = t
    scn.camera = cam
    scn.render.engine = "CYCLES"; scn.cycles.device = "CPU"; scn.cycles.samples = 32
    scn.render.resolution_x, scn.render.resolution_y = ((1400, 560) if SHEET else (600, 700))
    scn.render.filepath = PREVIEW
    bpy.ops.render.render(write_still=True)
    sys.exit(0)

out_dir = os.path.dirname(os.path.abspath(OUT))
os.makedirs(out_dir, exist_ok=True)
for shape in ORDER:
    for ob in list(bpy.data.objects):
        bpy.data.objects.remove(ob)
    make_blob(shape)
    path = os.path.join(out_dir, "blob_%s.glb" % shape)
    bpy.ops.export_scene.gltf(filepath=path, export_format="GLB", export_yup=True,
                              export_apply=True, export_cameras=False, export_lights=False)
    print("wrote", path)
