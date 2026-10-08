"""Builds the blob character in Blender (headless) and exports rust/crates/bbq_app/assets/models/blob_<name>.glb, one file each.

Run from the rust/ folder:   python3 tools/make_blob.py
(needs `pip install bpy`, Blender as a Python module). Open the .glb in Blender to tweak it.

Shapes and sizes are the same as the JavaScript game's createChar():
 capsule body, lighter head, white eyes + black pupils, flat dark feet, two lighter hands.
Blender is Z-up; the exporter turns that into the game's Y-up, so a point (x, up, forward)
is written to Blender as (x, -forward, up).
"""
import bpy, bmesh, math, os, random, sys

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
# (Step 2c: the singlet and the thongs have their own fixed colours and are not tinted.)
M = {
    "Body": mat("BodyMat", (0.8, 0.8, 0.8)),
    "Head": mat("HeadMat", (0.9, 0.9, 0.9)),
    "Foot": mat("FootMat", (0.5, 0.5, 0.5)),
    "White": mat("EyeWhite", (1, 1, 1)),
    "Black": mat("Pupil", (0.07, 0.07, 0.07)),
    "Singlet": mat("SingletMat", (0.93, 0.92, 0.86)),
    "ThongPear": mat("ThongPearMat", (1.0, 0.69, 0.18)),
    "ThongEgg": mat("ThongEggMat", (0.18, 0.66, 0.85)),
    "ThongGumdrop": mat("ThongGumdropMat", (0.91, 0.27, 0.23)),
    "Strap": mat("StrapMat", (0.12, 0.12, 0.12)),
    "ThongClassic": mat("ThongClassicMat", (0.30, 0.76, 0.42)),
}


def pos(x, up, fwd):
    return (x, -fwd, up)


# Body shapes. Radius at the top and at the bottom of the body, plus its height (the old capsule
# was 0.36 round everywhere and 1.42 tall). The body is egg-shaped: the radius changes smoothly
# from the bottom value to the top value, and both ends stay round.
# C9.5 (8 Oct 2026): four clearly different outlines, so you can tell who is who from across the
# yard. Looks only: hit and catch sizes are fixed in the game code and do not read these.
SHAPES = {
    "classic": dict(bottom=0.37, top=0.37, height=1.42, capsule=True),  # the original capsule: straight sides
    "pear": dict(bottom=0.56, top=0.25, height=1.34),     # a proper pear: big round bottom, narrow shoulders
    "egg": dict(bottom=0.41, top=0.33, height=1.62),      # tall and slim, round at both ends
    "gumdrop": dict(bottom=0.57, top=0.42, height=1.12),  # short, wide and squat
}
# Step 2c caricature: bigger head, hands and feet, a tummy, thongs, a wonky head. Visual only: the
# game's hit and catch sizes are fixed in code and do not read this. Classic is left as the
# original capsule (decision 4 Oct 2026).
CARICATURE = {
    # belly: how far the tummy swells forward (m), its height and how tall it is
    # Classic (updated 5 Oct 2026 at Marcus's request): still the original capsule body, now with the
    # same caricature as the others: bigger head, hands and feet, thongs, a singlet and a tummy
    "classic": dict(head=0.32, hand=0.12, foot=0.16, belly=0.12, belly_y=0.76, belly_h=0.32, tilt=0.05, thong="ThongClassic"),
    "pear": dict(head=0.30, hand=0.125, foot=0.17, belly=0.15, belly_y=0.66, belly_h=0.34, tilt=-0.07, thong="ThongPear"),
    "egg": dict(head=0.33, hand=0.125, foot=0.16, belly=0.12, belly_y=0.84, belly_h=0.34, tilt=0.09, thong="ThongEgg"),
    "gumdrop": dict(head=0.37, hand=0.14, foot=0.19, belly=0.19, belly_y=0.56, belly_h=0.32, tilt=-0.05, thong="ThongGumdrop"),
}
# Style experiment (S1 in GRAPHICS_2C.md): `--style pop` or `--style clay` writes
# blob_<name>_<style>.glb next to the normal ones, with different proportions.
STYLE = sys.argv[sys.argv.index("--style") + 1] if "--style" in sys.argv else ""
if STYLE == "pop":
    # cartoon pop: much bigger head and eyes, hands and feet; the body a little slimmer
    for k, c in CARICATURE.items():
        c["head"] *= 1.32; c["hand"] *= 1.4; c["foot"] *= 1.35; c["belly"] *= 1.0
    for k, sh in SHAPES.items():
        if not sh.get("capsule"):
            sh["bottom"] *= 0.95; sh["top"] *= 0.9
elif STYLE == "clay":
    # dusty clay toys: wide, squat, round, a big soft belly, a smaller head, small hands
    for k, c in CARICATURE.items():
        c["head"] *= 0.95; c["hand"] *= 0.88; c["foot"] *= 1.12; c["belly"] *= 1.5; c["belly_h"] *= 1.15
    for k, sh in SHAPES.items():
        if sh.get("capsule"):
            sh["bottom"] *= 1.2; sh["top"] *= 1.2
        else:
            sh["bottom"] *= 1.2; sh["top"] *= 1.22; sh["height"] *= 0.9
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


def torus(name, major, minor, at, material, tilt=(0.0, 0.0), scale=(1, 1, 1), parent=None):
    """A thin ring (used for the thong straps). `tilt` leans it (x, y) in radians."""
    bm = bmesh.new()
    seg_major, seg_minor = 24, 8
    for i in range(seg_major):
        a = 2 * math.pi * i / seg_major
        for j in range(seg_minor):
            b = 2 * math.pi * j / seg_minor
            r = major + minor * math.cos(b)
            bm.verts.new((r * math.cos(a), r * math.sin(a), minor * math.sin(b)))
    bm.verts.ensure_lookup_table()
    for i in range(seg_major):
        for j in range(seg_minor):
            v = lambda ii, jj: bm.verts[(ii % seg_major) * seg_minor + (jj % seg_minor)]
            bm.faces.new((v(i, j), v(i + 1, j), v(i + 1, j + 1), v(i, j + 1)))
    bm.normal_update()
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    for p_ in me.polygons:
        p_.use_smooth = True
    ob = bpy.data.objects.new(name, me)
    ob.data.materials.append(material)
    ob.location = pos(*at)
    ob.rotation_euler = (tilt[0], tilt[1], 0.0)
    ob.scale = (scale[0], scale[2], scale[1])
    scn.collection.objects.link(ob)
    if parent:
        ob.parent = parent
        ob.matrix_parent_inverse = parent.matrix_world.inverted()
    return ob


def profile_radius(shape, up):
    """The real half-width of the torso at height `up` (the sphere narrows towards its ends)."""
    sh = SHAPES[shape]
    if sh.get("capsule"):
        r = 0.36 * sh["bottom"] / 0.37
        mid, half = 0.78, 0.275
        dy = max(0.0, abs(up - mid) - half)
        return math.sqrt(max(0.0, r * r - dy * dy))
    u = max(0.0, min(1.0, (up - BODY_BASE) / sh["height"]))
    w = sh["top"] + (sh["bottom"] - sh["top"]) * (1 - u)
    return w * math.sqrt(max(0.0, 1 - (2 * u - 1) ** 2))


def belly_field(c, x, y, z):
    """How a point moves for the two belly shape keys: (swell, sag). Swell pushes the front of the
    body outwards round the tummy; sag lets it droop down and forward (the game bounces the two).
    Blender coordinates: x sideways, -y forward, z up."""
    r = math.hypot(x, y)
    if r < 1e-6:
        return (0, 0, 0), (0, 0, 0)
    # a broad, round swell: it wraps well round the sides (not just straight out the front) and
    # fades out softly above and below, so it reads as a full tummy rather than a point
    cosf = -y / r
    front = max(0.0, (cosf + 0.45) / 1.45) ** 1.25
    d = (z - c["belly_y"]) / c["belly_h"]
    g = math.exp(-(d * d) ** 1.4)  # flatter top than a plain bell curve
    amt = c["belly"] * g * front
    swell = (x / r * amt, y / r * amt, -0.15 * amt)
    sag = (0.0, -0.35 * amt, -0.6 * amt)
    return swell, sag


def add_belly_keys(ob, c, swell_value=1.0):
    """Shape keys "Belly" (on by default) and "Sag" (off) that the game drives to make it bounce."""
    ob.shape_key_add(name="Basis")
    kb = ob.shape_key_add(name="Belly")
    ks = ob.shape_key_add(name="Sag")
    for i, v in enumerate(ob.data.vertices):
        # the field is in the body's frame; the capsule torso's own origin is up at 0.78
        sw, sg = belly_field(c, v.co.x + ob.location.x, v.co.y + ob.location.y, v.co.z + ob.location.z)
        kb.data[i].co = (v.co.x + sw[0], v.co.y + sw[1], v.co.z + sw[2])
        ks.data[i].co = (v.co.x + sg[0], v.co.y + sg[1], v.co.z + sg[2])
    kb.value = swell_value
    ks.value = 0.0


def singlet(shape, y0, y1, material, parent, seed=1, front_lift=0.0):
    """A loose singlet over the chest: a shell a little wider than the torso, with folds, a wavy
    hem that rides up (so the tummy sticks out below it) and a wonky collar."""
    rnd = random.Random(seed)
    ph = [rnd.uniform(0, 6.28) for _ in range(4)]
    n_a, n_h = 56, 9
    bm = bmesh.new()
    rows = []
    for j in range(n_h + 1):
        t = j / n_h
        row = []
        for i in range(n_a):
            a = 2 * math.pi * i / n_a
            hem = 0.035 * math.sin(3 * a + ph[0]) + 0.02 * math.sin(7 * a + ph[1])  # wavy bottom edge
            hem += front_lift * max(0.0, math.cos(a)) ** 2  # rides up over the tummy at the front
            collar = 0.03 * math.sin(2 * a + ph[2])
            y = (y0 + hem * (1 - t)) + t * ((y1 + collar) - (y0 + hem * (1 - t)))
            r = profile_radius(shape, y) * 1.045 + 0.012
            # folds: more at the bottom (where it bunches over the tummy)
            f = (0.014 * math.sin(5 * a + 9 * t + ph[2]) + 0.009 * math.sin(11 * a - 13 * t + ph[3])) * (1.0 - 0.6 * t)
            r += f
            fwd = 0.02 * max(0.0, math.cos(a)) * (1 - t)   # it hangs a bit forward over the tummy
            row.append(bm.verts.new(((r + fwd) * math.sin(a), -(r + fwd) * math.cos(a), y)))
        rows.append(row)
    for j in range(n_h):
        for i in range(n_a):
            bm.faces.new((rows[j][i], rows[j][(i + 1) % n_a], rows[j + 1][(i + 1) % n_a], rows[j + 1][i]))
    bm.normal_update()
    me = bpy.data.meshes.new("Singlet")
    bm.to_mesh(me)
    bm.free()
    for p_ in me.polygons:
        p_.use_smooth = True
    ob = bpy.data.objects.new("Singlet", me)
    ob.data.materials.append(material)
    scn.collection.objects.link(ob)
    ob.parent = parent
    ob.matrix_parent_inverse = parent.matrix_world.inverted()
    return ob


def make_blob(shape, dx=0.0):
    """One whole blob. Parts are named so the game can find them (Torso, Head, HandR, HandL...)."""
    body = bpy.data.objects.new("Body", None)
    body.location = (dx, 0, 0)
    scn.collection.objects.link(body)
    if SHAPES[shape].get("capsule"):
        sphere("Torso", 0.36 * SHAPES[shape]["bottom"] / 0.37, (0, 0.78, 0), M["Body"], parent=body, capsule=0.55)
    else:
        egg_body("Torso", shape, M["Body"], parent=body)
    c = CARICATURE.get(shape)
    k = c["head"] / 0.30 if c else 1.0           # how much bigger the head and eyes are
    # the head sits on top of the body, wherever its top is (taller bodies, higher heads)
    lift = SHAPES[shape]["height"] - 1.42
    head = sphere("Head", c["head"] if c else 0.30, (0, 1.50 + lift + (0.03 if c else 0.0), 0), M["Head"], parent=body)
    if c:
        head.rotation_euler = (0.0, c["tilt"], 0.0)   # a wonky head: leaning to one side
    hy = 1.56 + lift + (0.03 if c else 0.0) + (0.02 * (k - 1.0) if c else 0.0)
    for side, sg in (("L", -1), ("R", 1)):
        er = 0.085 * (k if c else 1.0)
        sphere("Eye" + side, er, (sg * 0.11 * k, hy, 0.24 * k), M["White"], parent=body, seg=20, rings=11)
        sphere("Pupil" + side, 0.042 * (k if c else 1.0), (sg * 0.11 * k, hy, 0.24 * k + er * 0.88), M["Black"], parent=body, seg=16, rings=9)
        if c:
            # big feet that stick out the front, and a thong under each (its own colour)
            fr = c["foot"]
            sphere("Foot" + side, fr, (sg * 0.2, 0.10, 0.12), M["Foot"], scale=(1, 0.5, 1.7), parent=body, seg=24, rings=13)
            sphere("Thong" + side, fr * 1.02, (sg * 0.2, 0.035, 0.13), M[c["thong"]], scale=(1.04, 0.16, 1.82), parent=body, seg=24, rings=9)
            # the strap: a thin dark ring over the toes
            torus("Strap" + side, 0.075, 0.014, (sg * 0.2, 0.15, 0.2), M["Strap"], tilt=(math.radians(90), 0.0), parent=body)
        else:
            sphere("Foot" + side, 0.13, (sg * 0.2, 0.09, 0.05), M["Foot"], scale=(1, 0.6, 1.4), parent=body, seg=24, rings=13)
    if c:
        # a tummy that sticks out the front, and the wrinkles of a singlet round it
        # the tummy is part of the body (shape keys), not a ball stuck on the front
        torso = bpy.data.objects["Torso"] if "Torso" in bpy.data.objects else None
        for ob in scn.collection.objects:
            if ob.name.startswith("Torso"):
                torso = ob
        add_belly_keys(torso, c)
        sing = singlet(shape, c["belly_y"] - 0.04, 1.22 + (SHAPES[shape]["height"] - 1.42), M["Singlet"], body, seed=len(shape), front_lift=0.12)
        add_belly_keys(sing, c)
    # HandR is the one the game animates (it swings), HandL is the other side.
    # Hands rest just outside the body at arm height (0.88 up).
    hr = c["hand"] if c else 0.10
    hx = body_radius(shape, 0.88) + 0.1
    sphere("HandR", hr, (-hx, 0.88, 0.05), M["Head"], parent=body, seg=24, rings=13)
    sphere("HandL", hr, (hx, 0.88, 0.05), M["Head"], parent=body, seg=24, rings=13)
    measure(shape, body, head, hx)
    return body


def measure(shape, body, head, hx):
    """Print where things sit on this model, for `Character::measure` in bbq_core."""
    bpy.context.view_layer.update()
    torso = [ob for ob in body.children if ob.name.startswith("Torso")][0]
    ox = body.location.x
    tv = [torso.matrix_world @ v.co for v in torso.data.vertices]
    hv = [head.matrix_world @ v.co for v in head.data.vertices]
    print("MEASURE %-8s %-6s hand_x %.3f head_top %.3f body_top %.3f body_w %.3f" % (
        shape, STYLE or "plain", hx, max(v.z for v in hv), max(v.z for v in tv),
        max(abs(v.x - ox) for v in tv)))


SHEET = "--sheet" in sys.argv
ORDER = ["classic", "pear", "egg", "gumdrop"]

if SHEET:
    for i, name in enumerate(ORDER):
        b = make_blob(name, dx=(i - 1.5) * 1.3)
        if "--side" in sys.argv:
            b.rotation_euler = (0, 0, math.radians(-90))  # facing right, to see the tummy
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
    path = os.path.join(out_dir, "blob_%s%s.glb" % (shape, ("_" + STYLE) if STYLE else ""))
    bpy.ops.export_scene.gltf(filepath=path, export_format="GLB", export_yup=True,
                              export_apply=True, export_cameras=False, export_lights=False)
    print("wrote", path)
