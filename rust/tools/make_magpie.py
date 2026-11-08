"""Build the polished Backyard magpie. Run blender --background --python rust/tools/make_magpie.py."""

from math import cos, pi, sin
from pathlib import Path

import bpy
from mathutils import Vector


OUT = Path(__file__).resolve().parents[1] / "crates/bbq_app/assets/models/magpie_pop.glb"
INK = 0x181C24
BREAST = 0x353F50
CROWN = 0x2F3643
SLATE = 0x434E60
WHITE = 0xEEEADA
WARM_WHITE = 0xD9D6C8
LEG = 0x4A4546


def linear(c):
    def channel(v):
        s = v / 255.0
        return s / 12.92 if s <= 0.04045 else ((s + 0.055) / 1.055) ** 2.4
    return tuple(channel((c >> shift) & 255) for shift in (16, 8, 0)) + (1.0,)


def blender(v):
    # Vertices are authored in game space: +Y up, +Z toward the bill.
    return (v.x, -v.z, v.y)


class Section:
    """One mesh, with its origin at the joint used in the game."""

    def __init__(self, name, joint):
        self.name = name
        self.joint = Vector(joint)
        self.vertices = []
        self.colours = []
        self.faces = []

    def ring_mesh(self, rings, colour, ink_width=0):
        points = [Vector(p) for ring in rings for p in ring]
        n = len(rings[0])
        faces = []
        for r in range(len(rings) - 1):
            for i in range(n):
                j = (i + 1) % n
                faces.append((r*n+i, r*n+j, (r+1)*n+j, (r+1)*n+i))
        faces.append(tuple(range(n - 1, -1, -1)))
        faces.append(tuple(range(len(points) - n, len(points))))
        base = len(self.vertices)
        self.vertices.extend(points)
        self.colours.extend(colour(p) if callable(colour) else colour for p in points)
        self.faces.extend(tuple(base + i for i in face) for face in faces)
        if ink_width:
            # Offset the silhouette along its own normals. Only the back of this shell draws.
            normals = [Vector((0, 0, 0)) for _ in points]
            for face in faces:
                a, b, c = (points[i] for i in face[:3])
                normal = (b - a).cross(c - a).normalized()
                for i in face:
                    normals[i] += normal
            base = len(self.vertices)
            self.vertices.extend(p + normal.normalized() * ink_width for p, normal in zip(points, normals))
            self.colours.extend([INK] * len(points))
            self.faces.extend(tuple(base + i for i in reversed(face)) for face in faces)

    def spindle(self, a, b, profile, colour, segments=12, ink_width=0):
        """Cross sections in (fraction along axis, lateral radius, vertical radius)."""
        a, b = Vector(a), Vector(b)
        axis = (b - a).normalized()
        up = Vector((0, 1, 0))
        if abs(axis.dot(up)) > 0.97:
            up = Vector((0, 0, 1))
        across = up.cross(axis).normalized()
        deep = axis.cross(across)
        rings = [[a.lerp(b, t) + across * (w * cos(i * 2*pi/segments))
                  + deep * (h * sin(i * 2*pi/segments)) for i in range(segments)]
                 for t, w, h in profile]
        self.ring_mesh(rings, colour, ink_width)

    def oval(self, c, radii, colour, segments=16, ink_width=0):
        x, y, z = c
        w, h, d = radii
        # A tiny flat cap at each pole avoids duplicated vertices and zero-area faces.
        angles = [-1.55 + 3.10 * k / 9 for k in range(10)]
        profile = [((sin(t) + 1) / 2, w * cos(t), h * cos(t)) for t in angles]
        self.spindle((x, y, z-d), (x, y, z+d), profile, colour, segments, ink_width)

    def finish(self, material):
        mesh = bpy.data.meshes.new(self.name)
        mesh.from_pydata([blender(v - self.joint) for v in self.vertices], [], self.faces)
        mesh.update()
        colours = mesh.color_attributes.new(name="Plumage", type='BYTE_COLOR', domain='POINT')
        for entry, c in zip(colours.data, self.colours):
            entry.color = linear(c)
        mesh.color_attributes.active_color = colours
        for face in mesh.polygons:
            face.use_smooth = True
        mesh.materials.append(material)
        obj = bpy.data.objects.new(self.name, mesh)
        obj.location = blender(self.joint)
        bpy.context.collection.objects.link(obj)


def feather(section, a, b, width, colour):
    section.spindle(a, b, [(0, width*.27, .004), (.32, width, .011), (.73, width*.75, .008),
                          (.98, width*.16, .002), (1, .001, .001)], colour, 10)


def make_bird():
    body = Section("MagpieBody", (0, 0, 0))
    body.spindle((0, .21, -.23), (0, .31, .16),
                 [(0, .025, .035), (.20, .105, .105), (.50, .144, .158),
                  (.72, .146, .15), (.92, .096, .095), (1, .023, .022)], BREAST, 24, .0035)
    # White rump and individually tapered tail feathers, with longer centre pair.
    body.oval((0, .266, -.154), (.097, .058, .126), WARM_WHITE)
    for lane in range(-2, 3):
        x = lane * .025
        feather(body, (x*.48, .245, -.13), (x*1.25, .142, -.445 + abs(lane)*.023),
                .025, [SLATE, CROWN, BREAST, CROWN, SLATE][lane+2])
    for side in (-1, 1):
        x = side * .053
        body.spindle((x, .012, .044), (x, .164, -.005),
                     [(0, .011, .011), (.43, .013, .012), (.52, .016, .015), (1, .016, .016)], LEG, 9)
        # Three forward toes and one rear toe curl over the paling at sole height.
        for dx, dz in [(-.027, .042), (0, .054), (.026, .038), (.008, -.041)]:
            body.spindle((x, .016, .044), (x+dx, .004, .044+dz),
                         [(0, .011, .009), (.63, .008, .006), (1, .002, .002)], LEG, 7)
    head = Section("MagpieHead", (0, .36, .08))

    def plumage(p):
        # The Australian magpie's white nape wraps behind the crown, above a black throat.
        if p.z < -.007 + .045 * (p.y-.48) and p.y > .38:
            return WHITE
        return CROWN

    head.oval((0, .479, .072), (.159, .164, .179), plumage, 32, .003)
    for side in (-1, 1):
        head.oval((side*.121, .50, .162), (.038, .044, .036), INK)
        head.oval((side*.140, .505, .179), (.024, .034, .025), 0xAE582D)
        head.oval((side*.147, .508, .192), (.017, .024, .019), 0x17191D)
        head.oval((side*.157, .517, .200), (.007, .008, .006), 0xFFF2D7, 10)
    # A broad ivory wedge narrows to a dark, slightly downturned hook.
    head.spindle((0, .464, .192), (0, .453, .415),
                 [(0, .047, .032), (.21, .056, .044), (.52, .037, .032),
                  (.78, .019, .018), (1, .002, .003)],
                 lambda p: CROWN if p.z > .345 else (WARM_WHITE if p.y < .458 else 0xE1E2D6), 12, .002)
    for side in (-1, 1):
        head.oval((side*.048, .472, .263), (.004, .004, .009), 0x575657, 8)
    sections = [body, head]
    for side, name in [(-1, "MagpieWingL"), (1, "MagpieWingR")]:
        wing = Section(name, (side*.111, .332, .025))
        wing.spindle((side*.119, .319, .071), (side*.103, .181, -.249),
                     [(0, .014, .028), (.23, .052, .075), (.51, .047, .080),
                      (.78, .027, .045), (1, .005, .008)], SLATE, 18, .0025)
        for i in range(4):
            y = .30 - i*.026
            feather(wing, (side*(.133+i*.003), y, -.020),
                    (side*(.106+i*.013), y-.075, -.28+i*.022), .034, [BREAST, SLATE, CROWN, BREAST][i])
        # Pale scapular patches form a broad flash on both folded wings.
        wing.oval((side*.150, .315, -.042), (.028, .047, .093), WHITE, 18)
        sections.append(wing)
    return sections


bpy.ops.wm.read_factory_settings(use_empty=True)
material = bpy.data.materials.new("Painted plumage")
material.use_nodes = True
material.use_backface_culling = True
shader = material.node_tree.nodes.get("Principled BSDF")
shader.inputs["Roughness"].default_value = .69
colours = material.node_tree.nodes.new("ShaderNodeVertexColor")
colours.layer_name = "Plumage"
material.node_tree.links.new(colours.outputs["Color"], shader.inputs["Base Color"])
for section in make_bird():
    section.finish(material)
bpy.ops.export_scene.gltf(filepath=str(OUT), export_format="GLB", export_yup=True,
                          export_vertex_color='ACTIVE', export_cameras=False, export_lights=False)
