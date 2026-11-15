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


def box_pitch(name, center, size, pitch, material, bevel=0.0):
    """A game-axis box tipped about x, used for panels sitting on the rear roof slope."""
    cx, cy, cz = center
    sx, sy, sz = size
    verts = []
    for y in (-sy / 2, sy / 2):
        for z in (-sz / 2, sz / 2):
            for x in (-sx / 2, sx / 2):
                ry = y * cos(pitch) - z * sin(pitch)
                rz = y * sin(pitch) + z * cos(pitch)
                verts.append((cx + x, cy + ry, cz + rz))
    faces = [(0, 1, 3, 2), (4, 6, 7, 5), (0, 4, 5, 1),
             (2, 3, 7, 6), (0, 2, 6, 4), (1, 5, 7, 3)]
    obj = mesh_object(name, verts, faces, material)
    if bevel:
        modifier = obj.modifiers.new("Soft panel edges", "BEVEL")
        modifier.width = bevel
        modifier.segments = 2
        obj.modifiers.new("Weighted panel normals", "WEIGHTED_NORMAL")
    return obj


def build_backyard_finish(m):
    """The next Backyard story props, authored from Blender's native mesh primitives."""
    # Three sun-faded, mismatched gnomes keep watch beside the fence.
    for index, (x, z, hat, coat) in enumerate([
            (-36.0, -20.0, m["gnome_hat_red"], m["gnome_blue"]),
            (36.0, -18.0, m["gnome_hat_green"], m["gnome_coat_yellow"]),
            (-35.5, 19.0, m["gnome_hat_yellow"], m["gnome_coat_red"])]):
        sphere("Gnome boots", (x, .10, z), (.22, .10, .25), m["gnome_boot"], 10, 6)
        cylinder("Gnome coat", (x, .12, z), (x, .60, z), .21, coat, 10, .13)
        sphere("Gnome face", (x, .68, z + .015), (.17, .18, .15), m["gnome_skin"], 12, 8)
        sphere("Gnome nose", (x, .69, z + .16), (.055, .05, .045), m["gnome_skin"], 8, 6)
        sphere("Gnome beard", (x, .51, z + .10), (.135, .13, .06), m["gnome_beard"], 10, 6)
        cylinder("Gnome hat brim", (x, .78, z), (x, .85, z), .20, m["gnome_hat_rim"], 12, .18)
        cylinder("Pointed gnome hat", (x, .83, z), (x + (.025 if index % 2 else -.02), 1.16, z), .17, hat, 10, .012)
        for eye in (-.055, .055):
            sphere("Gnome eye", (x + eye, .72, z + .145), (.018, .021, .012), m["dark"], 7, 5)
        # A different scarf or vest stripe gives each little spectator its own look.
        box("Gnome scarf", (x, .57, z + .148), (.29, .075, .025), m["gnome_scarf"], yaw=(index - 1) * .08, bevel=.015)

    # A wheelbarrow left in the middle of a garden job, with a dusty steel tray.
    wx, wz = 4.2, 15.8
    box("Wheelbarrow shadow", (wx, .012, wz), (1.45, .02, 1.55), m["shadow"])
    box("Wheelbarrow tray body", (wx, .55, wz), (1.02, .25, .80), m["barrow_steel"], yaw=-.04, bevel=.13)
    box("Wheelbarrow raised front lip", (wx, .68, wz + .35), (1.02, .22, .12), m["barrow_rust"], yaw=-.05, bevel=.035)
    box("Barrow soil mound", (wx, .70, wz - .025), (.78, .18, .57), m["soil"], yaw=.06, bevel=.16)
    # Dark earth clods and a few green cuttings make the tray read from above.
    for i, off in enumerate([(-.20, .79, -.17), (.08, .80, .04), (.24, .76, -.11), (-.07, .82, .18)]):
        sphere("Soil clod", (wx + off[0], off[1], wz + off[2]), (.12, .06, .10), m["soil_lump" if i % 2 else "soil"], 8, 5)
    cylinder("Wheelbarrow axle", (wx - .19, .31, wz + .52), (wx + .19, .31, wz + .52), .045, m["iron"], 8)
    cylinder("Wheelbarrow rubber wheel", (wx - .15, .31, wz + .52), (wx + .15, .31, wz + .52), .28, m["rubber"], 12, .28)
    cylinder("Wheelbarrow hub", (wx - .17, .31, wz + .52), (wx + .17, .31, wz + .52), .10, m["steel"], 10, .10)
    for side in (-1, 1):
        cylinder("Barrow handle rail", (wx + side*.34, .50, wz - .25), (wx + side*.34, .68, wz - 1.15), .045, m["barrow_steel"], 8, .032)
        cylinder("Barrow rubber grip", (wx + side*.34, .68, wz - 1.10), (wx + side*.34, .68, wz - 1.35), .065, m["rubber"], 8, .045)
        cylinder("Barrow leg", (wx + side*.36, .12, wz - .34), (wx + side*.32, .43, wz - .20), .045, m["iron"], 8, .035)
    # A hand trowel lies across the lip; the spade catches a small highlight.
    cylinder("Trowel timber grip", (wx + .28, .83, wz + .14), (wx + .42, .85, wz - .26), .035, m["timber"], 8, .028)
    box("Trowel blade", (wx + .45, .84, wz - .34), (.18, .025, .25), m["steel"], yaw=-.2, bevel=.045)
    for i in range(5):
        x = wx - .78 + i*.22
        cylinder("Loose garden clipping", (x, .03, wz - .82), (x + .08*(i%2-.5), .30+(i%3)*.04, wz - .78), .018, m["clipping"], 6, .006)

    # Open rough compost corner: mismatched slats, clippings, and a blue-tongue lizard.
    cx, cz = 1.0, 24.2
    for side in (-1, 1):
        for row in range(4):
            box("Compost side slat", (cx + side*.63, .18 + row*.23, cz), (.09, .20, 1.22),
                m["compost_wood_a" if row % 2 else "compost_wood_b"], yaw=(row-1.5)*.012, bevel=.018)
    for row in range(4):
        gap = .16 if row == 3 else 0.0
        box("Compost front slat", (cx, .18 + row*.23, cz + .60), (1.22-gap, .20, .08),
            m["compost_wood_b" if row % 2 else "compost_wood_a"], yaw=(row-1.5)*.009, bevel=.018)
    box("Compost back slat", (cx, .42, cz - .60), (1.22, .66, .08), m["compost_wood_a"], bevel=.02)
    for i in range(8):
        x = cx + ((i*37)%7-3)*.13
        z = cz + ((i*17)%5-2)*.11
        sphere("Compost leaves", (x, .69+(i%3)*.05, z), (.20+(i%2)*.05, .06, .11), m["clipping" if i%3 else "leaf"], 9, 5)
    sphere("Blue tongue lizard body", (cx + .95, .10, cz + .35), (.28, .095, .11), m["lizard"], 12, 6)
    sphere("Blue tongue lizard head", (cx + 1.17, .11, cz + .35), (.12, .10, .10), m["lizard_blue"], 10, 6)
    cylinder("Lizard tail", (cx + .73, .10, cz + .34), (cx + .43, .07, cz + .25), .035, m["lizard"], 7, .008)
    for side in (-1, 1):
        cylinder("Lizard leg", (cx+.99, .06, cz+.29+side*.05), (cx+.88, .04, cz+.18+side*.08), .018, m["lizard"], 6, .012)

    # Small flourishes in sidewalk chalk beyond the cricket crease markings.
    chalk = m["chalk"]
    for i in range(5):
        box("Chalk hopscotch line", (18.3, .014, 12.4+i*.32), (.92, .012, .035), chalk, yaw=.015*(i%2), bevel=.008)
    for x in (17.85, 18.75):
        box("Chalk hopscotch side", (x, .014, 13.05), (.035, .012, 1.32), chalk, bevel=.008)
    # Flower and sun doodles are just irregular linework, rather than a new objective marker.
    for i in range(8):
        a = 2*pi*i/8
        cylinder("Chalk flower petal", (20.0+.16*cos(a), .018, 12.8+.16*sin(a)),
                 (20.0+.29*cos(a), .018, 12.8+.29*sin(a)), .022, m["chalk_yellow"], 6, .018)
    sphere("Chalk flower centre", (20.0, .018, 12.8), (.09,.012,.09), m["chalk_yellow"], 10, 5)
    for i in range(5):
        box("Chalk tally mark", (20.8+(i%4)*.10, .016, 12.5+(i//4)*.22), (.027,.01,.18), chalk,
            yaw=.18 if i < 4 else -.45, bevel=.006)

    # Pool-toy rack, with the faded toys hung above splash height by the pool gate.
    rx, rz = -11.2, 9.4
    for x in (-.48, .48):
        box("Pool rack upright", (rx+x, .86, rz), (.075, 1.72, .075), m["pool_rack_wood"], bevel=.02)
        box("Rack foot", (rx+x, .07, rz), (.38,.10,.25), m["pool_rack_wood"], bevel=.025)
    box("Pool rack top rail", (rx, 1.62, rz), (1.12,.09,.09), m["pool_rack_wood"], bevel=.025)
    box("Faded kickboard", (rx-.22, 1.18, rz+.08), (.48,.62,.10), m["kickboard"], yaw=-.06, bevel=.12)
    cylinder("Pool noodle", (rx+.22, .82, rz+.14), (rx+.22, 1.54, rz+.14), .095, m["noodle"], 12, .095)
    for x in (rx-.52, rx+.50):
        box("Hanging pool flipper", (x, .83, rz+.09), (.28,.17,.06), m["flipper"], yaw=(x-rx)*.16, bevel=.06)
        box("Flipper foot pocket", (x, .98, rz+.09), (.15,.22,.07), m["flipper"], bevel=.055)

    # Rust-specked reel and kinked line by the existing hose and sprinkler.
    hx, hz = 27.1, -8.35
    box("Hose reel wall board", (hx, .72, hz), (.74, 1.20, .14), m["timber"], bevel=.045)
    for side in (-1, 1):
        cylinder("Hose reel stand", (hx+side*.33,.08,hz), (hx+side*.28,.72,hz), .045, m["iron"], 8, .035)
    cylinder("Reel axle", (hx-.42,.72,hz), (hx+.42,.72,hz), .075, m["iron"], 8, .075)
    cylinder("Hose coil", (hx-.25,.72,hz), (hx+.25,.72,hz), .40, m["hose"], 12, .40)
    cylinder("Reel hub cap", (hx-.31,.72,hz), (hx-.35,.72,hz), .12, m["steel"], 10, .12)
    for i in range(6):
        a = 2*pi*i/6
        cylinder("Reel spoke", (hx-.31,.72,hz), (hx-.31,.72+.34*cos(a),hz+.34*sin(a)), .022, m["steel"], 6, .022)
    # The hose bows down, catches a kink, then lies over the lawn toward the sprinkler.
    for a, b in [((hx+.1,.18,hz+.05),(hx+.55,.07,hz+.12)), ((hx+.55,.07,hz+.12),(hx+.42,.06,hz+.42)),
                 ((hx+.42,.06,hz+.42),(25.65,.05,-7.72)), ((25.65,.05,-7.72),(25.45,.05,-7.82))]:
        cylinder("Kinked garden hose", a, b, .045, m["hose"], 8, .045)

    # Leaf nest box on the front gum tree, with claw scores and scraps at its roots.
    bx, by, bz = 13.1, 5.4, 32.7
    box("Possum box timber body", (bx,by,bz), (.72,.86,.62), m["nest_wood"], bevel=.045)
    box("Possum box sloping lid", (bx,by+.47,bz), (.82,.12,.72), m["nest_lid"], yaw=.035, bevel=.03)
    cylinder("Nest box entrance shadow", (bx,by+.06,bz+.317), (bx,by+.06,bz+.326), .14, m["nest_dark"], 14, .14)
    cylinder("Nest box entrance inner", (bx,by+.06,bz+.328), (bx,by+.06,bz+.332), .09, m["nest_wood"], 12, .09)
    for side in (-1, 1):
        box("Nest box mounting strap", (bx+side*.25,by-.02,bz+.33), (.04,.75,.025), m["iron"], bevel=.008)
    for i in range(4):
        cylinder("Possum claw mark", (bx-.18+i*.11,by-.18,bz+.327), (bx-.15+i*.11,by+.10,bz+.33), .008, m["nest_dark"], 5, .004)
    for i in range(9):
        a = i*2.4
        sphere("Gum leaves below possum box", (12.3+.42*cos(a),.06,33.0+.30*sin(a)), (.14,.035,.07), m["leaf"], 8, 5)

    # Backyard party remains: plates and an open, empty esky on the prep table.
    for x,z in [(-9.05,-18.12),(-8.18,-18.25),(-7.75,-18.0)]:
        cylinder("Enamel snag plate", (x,.835,z), (x,.865,z), .26, m["plate"], 16, .26)
        cylinder("Plate shallow well", (x,.866,z), (x,.874,z), .19, m["plate_inner"], 16, .19)
    ex, ez = -10.8, -16.0
    box("Party esky base", (ex,.31,ez), (.92,.56,.72), m["esky_red"], bevel=.11)
    box("Party esky rim", (ex,.61,ez), (.98,.09,.78), m["esky_white"], bevel=.035)
    box("Party esky open lid", (ex,.83,ez-.28), (.96,.12,.22), m["esky_white"], yaw=-.56, bevel=.04)
    for i in range(3):
        box("Empty esky dark interior", (ex-.22+i*.22,.66,ez), (.18,.025,.31), m["esky_dark"], bevel=.02)
    # The note is a cream card with a hand-letter-like dark stroke, readable from the tabletop.
    box("Snags note paper", (-8.60,.89,-18.68), (.72,.018,.42), m["paper"], yaw=.08, bevel=.018)
    note = bpy.data.curves.new("Snags 4 later handwritten note", type="FONT")
    note.body = "Snags 4 later..."
    note.size = .074
    note.shear = .18
    note.extrude = .001
    note_obj = bpy.data.objects.new("Handwritten snag note lettering", note)
    bpy.context.collection.objects.link(note_obj)
    note_obj.location = to_blender((-8.91,.904,-18.80))
    note.materials.append(m["ink"])
    bpy.ops.object.select_all(action="DESELECT")
    note_obj.select_set(True)
    bpy.context.view_layer.objects.active = note_obj
    bpy.ops.object.convert(target="MESH")
    box("Party esky ground shadow", (-10.8,.012,-16.0), (1.2,.02,1.0), m["shadow"])

    # A low-profile solar hot-water collector follows the back side of the house roof.
    box_pitch("Solar collector dark frame", (-11.5,5.22,-32.75), (2.65,.12,1.65), -.445, m["solar_frame"], .055)
    box_pitch("Solar collector glass", (-11.5,5.30,-32.75), (2.45,.045,1.45), -.445, m["solar_glass"], .025)
    for x in (-12.25,-11.75,-11.25,-10.75):
        box_pitch("Solar collector tube", (x,5.33,-32.75), (.035,.025,1.34), -.445, m["solar_tube"])
    cylinder("Solar hot water return pipe", (-12.65,5.18,-32.15), (-12.65,4.75,-31.25), .055, m["solar_pipe"], 10, .05)
    cylinder("Solar hot water feed pipe", (-10.35,5.18,-32.15), (-10.35,4.75,-31.25), .055, m["solar_pipe"], 10, .05)


def backyard_finish_materials():
    return {
        "gnome_hat_red": make_material("Gnome sun-faded red cap", (.64,.16,.10)),
        "gnome_hat_green": make_material("Gnome moss-green cap", (.25,.40,.22)),
        "gnome_hat_yellow": make_material("Gnome old yellow cap", (.75,.55,.16)),
        "gnome_blue": make_material("Gnome faded blue coat", (.22,.34,.48)),
        "gnome_coat_red": make_material("Gnome red coat", (.59,.22,.15)),
        "gnome_coat_green": make_material("Gnome green coat", (.30,.41,.24)),
        "gnome_coat_yellow": make_material("Gnome mustard coat", (.72,.51,.16)),
        "gnome_skin": make_material("Gnome painted terracotta skin", (.78,.53,.37)),
        "gnome_beard": make_material("Gnome chalky cream beard", (.88,.82,.68)),
        "gnome_boot": make_material("Gnome muddy little boots", (.28,.21,.14)),
        "gnome_hat_rim": make_material("Gnome hat brim", (.45,.32,.20)),
        "gnome_scarf": make_material("Gnome faded scarf", (.72,.34,.23)),
        "dark": make_material("Small dark details", (.10,.12,.11)),
        "barrow_steel": make_material("Wheelbarrow scratched zinc", (.39,.43,.41), .65, .24),
        "barrow_rust": make_material("Wheelbarrow rusted rim", (.42,.23,.13), .78, .12),
        "soil": make_material("Fresh garden soil", (.22,.15,.085)),
        "soil_lump": make_material("Moist soil clods", (.30,.22,.12)),
        "rubber": make_material("Old black rubber", (.12,.13,.12), .9),
        "steel": make_material("Worn hand tool steel", (.57,.60,.57), .5, .52),
        "iron": make_material("Dark garden iron", (.22,.24,.21), .72, .35),
        "timber": make_material("Weathered garden timber", (.43,.31,.20)),
        "clipping": make_material("Fresh green clippings", (.38,.48,.20)),
        "leaf": make_material("Dry gum leaves", (.56,.50,.30)),
        "compost_wood_a": make_material("Compost rough timber", (.39,.29,.18)),
        "compost_wood_b": make_material("Compost sun-bleached slats", (.52,.42,.28)),
        "lizard": make_material("Blue-tongue mottled olive", (.43,.47,.30)),
        "lizard_blue": make_material("Lizard blue tongue", (.24,.42,.56)),
        "chalk": make_material("Old white chalk", (.88,.84,.69)),
        "chalk_yellow": make_material("Faded yellow chalk", (.85,.65,.20)),
        "pool_rack_wood": make_material("Pool toy rack timber", (.53,.41,.27)),
        "kickboard": make_material("Sun-faded orange kickboard", (.82,.36,.20)),
        "noodle": make_material("Faded turquoise pool noodle", (.19,.59,.61)),
        "flipper": make_material("Old blue pool flippers", (.20,.36,.53)),
        "hose": make_material("Olive green kinked hose", (.27,.34,.16)),
        "nest_wood": make_material("Gum tree nest box timber", (.43,.33,.22)),
        "nest_lid": make_material("Sun-bleached nest box lid", (.56,.45,.29)),
        "nest_dark": make_material("Nest entrance and claw scores", (.12,.13,.11)),
        "plate": make_material("Cream enamel plate", (.79,.76,.64), .32, .15),
        "plate_inner": make_material("Plate worn centre", (.65,.63,.54), .36, .10),
        "esky_red": make_material("Party esky sun-faded red", (.67,.22,.15)),
        "esky_white": make_material("Esky pale plastic rim", (.75,.72,.60)),
        "esky_dark": make_material("Empty esky interior", (.20,.24,.22)),
        "paper": make_material("Snags note paper", (.85,.79,.63)),
        "ink": make_material("Felt pen ink", (.20,.19,.15)),
        "shadow": make_material("Soft prop contact shadow", (.24,.27,.16)),
        "solar_frame": make_material("Solar heater dark frame", (.13,.18,.18), .42, .28),
        "solar_glass": make_material("Blue green solar glass", (.20,.39,.40), .25, .20),
        "solar_tube": make_material("Solar copper tubes", (.45,.28,.17), .38, .48),
        "solar_pipe": make_material("Solar exposed copper pipe", (.47,.31,.19), .42, .46),
    }


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
    mats.update(backyard_finish_materials())
    builders = [
        ("backyard_overflow.glb", "Backyard rainwater overflow", build_overflow),
        ("poolside_shower.glb", "Poolside outdoor shower", build_shower),
        ("side_gate.glb", "Open brick side gate", build_gate),
        ("backyard_finish_props.glb", "Backyard story props", build_backyard_finish),
    ]
    OUT.mkdir(parents=True, exist_ok=True)
    for filename, name, build in builders:
        # Clear only the temporary export scene between independent assets.
        for obj in list(temp_scene.objects):
            bpy.data.objects.remove(obj, do_unlink=True)
        build(mats)
        if filename == "backyard_finish_props.glb":
            # Keep the scene asset light: evaluate small bevels, then combine the decorations into
            # one multi-material mesh so 177 authored pieces do not become 177 draw calls.
            meshes = [obj for obj in temp_scene.objects if obj.type == "MESH"]
            for obj in meshes:
                bpy.ops.object.select_all(action="DESELECT")
                obj.select_set(True)
                bpy.context.view_layer.objects.active = obj
                for modifier in list(obj.modifiers):
                    bpy.ops.object.modifier_apply(modifier=modifier.name)
            bpy.ops.object.select_all(action="DESELECT")
            for obj in meshes:
                obj.select_set(True)
            bpy.context.view_layer.objects.active = meshes[0]
            bpy.ops.object.join()
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
