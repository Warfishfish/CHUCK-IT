"""Author a shared hero gum tree for the Australian backyard environment.

Run through Blender MCP or Blender's Python console. It builds in a disposable scene,
leaves the open artist scene untouched, and exports a game-scale Y-up GLB.
"""
from math import cos, pi, sin
from pathlib import Path
import bpy
from mathutils import Vector

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "crates/bbq_app/assets/models/gum_tree_hero.glb"
BARK = ROOT / "crates/bbq_app/assets/textures/gum_bark_overhaul.png"


def game_to_blender(p):
    return (p[0], -p[2], p[1])


def mat(name, color, rough=.82, metallic=0.0):
    m = bpy.data.materials.new(name)
    m.diffuse_color = (*color, 1)
    m.use_nodes = True
    bsdf = next(n for n in m.node_tree.nodes if n.type == "BSDF_PRINCIPLED")
    bsdf.inputs["Base Color"].default_value = (*color, 1)
    bsdf.inputs["Roughness"].default_value = rough
    bsdf.inputs["Metallic"].default_value = metallic
    return m


def cylinder_between(name, a, b, r0, r1, material, sides=12):
    a, b = Vector(game_to_blender(a)), Vector(game_to_blender(b))
    delta = b - a
    mid = (a + b) * .5
    bpy.ops.mesh.primitive_cone_add(vertices=sides, radius1=r0, radius2=r1,
                                    depth=delta.length, location=mid)
    ob = bpy.context.object
    ob.name = name
    ob.rotation_mode = "QUATERNION"
    ob.rotation_quaternion = Vector((0, 0, 1)).rotation_difference(delta.normalized())
    ob.data.materials.append(material)
    for poly in ob.data.polygons:
        poly.use_smooth = True
    return ob


def leaf_cluster(name, p, scale, material, turn=0.0):
    # Compact overlapping sprays give a varied silhouette with closed, stable game meshes.
    bpy.ops.mesh.primitive_uv_sphere_add(segments=12, ring_count=8,
                                         location=game_to_blender(p))
    ob = bpy.context.object
    ob.name = name
    ob.rotation_euler[2] = turn
    ob.scale = scale
    ob.data.materials.append(material)
    for poly in ob.data.polygons:
        poly.use_smooth = True
    return ob


def build():
    bark = mat("Ribbon gum pale peeling bark", (.58, .49, .36), .91)
    bark_img = bpy.data.images.load(str(BARK), check_existing=True)
    bark_img.colorspace_settings.name = "sRGB"
    nodes = bark.node_tree.nodes
    links = bark.node_tree.links
    bsdf = next(n for n in nodes if n.type == "BSDF_PRINCIPLED")
    tex = nodes.new("ShaderNodeTexImage")
    tex.image = bark_img
    uv = nodes.new("ShaderNodeTexCoord")
    links.new(uv.outputs["UV"], tex.inputs["Vector"])
    links.new(tex.outputs["Color"], bsdf.inputs["Base Color"])
    noise = nodes.new("ShaderNodeTexNoise")
    noise.inputs["Scale"].default_value = 14.0
    noise.inputs["Detail"].default_value = 4.0
    bump = nodes.new("ShaderNodeBump")
    bump.inputs["Strength"].default_value = .20
    bump.inputs["Distance"].default_value = .065
    links.new(noise.outputs["Fac"], bump.inputs["Height"])
    links.new(bump.outputs["Normal"], bsdf.inputs["Normal"])

    greens = [
        mat("Gum canopy eucalyptus grey green", (.16,.25,.14), .92),
        mat("Gum canopy sunlit olive", (.25,.31,.16), .92),
        mat("Gum canopy blue sage", (.20,.30,.27), .92),
        mat("Gum canopy dry leaf tips", (.36,.32,.19), .94),
    ]
    blossom = mat("Scattered pale gum blossom", (.89,.79,.65), .86)

    # A slight S bend, flared roots and long high forks give the trunk a grown shape.
    trunk = [((0,0,0),(.10,2.1,.02),.53,.43),
             ((.10,2.1,.02),(-.12,4.3,.08),.43,.31),
             ((-.12,4.3,.08),(.10,6.3,.28),.31,.22),
             ((.10,6.3,.28),(.52,8.55,.55),.22,.105)]
    for a,b,r0,r1 in trunk:
        cylinder_between("Gum trunk with taper",a,b,r0,r1,bark,18)
    for i in range(7):
        a = i * 2*pi/7 + .18
        root_end = (cos(a)*1.22,.08,sin(a)*.94)
        cylinder_between("Exposed spreading gum root",(0,.46,0),root_end,.24,.055,bark,10)

    branch_ends=[]
    for i in range(7):
        a=i*2*pi/7+.37
        start=(.02,3.7,.05)
        mid=(cos(a)*1.45,5.0,sin(a)*1.3)
        end=(cos(a)*3.2,6.8+(.35 if i%2 else -.15),sin(a)*2.8)
        tip=(end[0]+cos(a+.35)*1.25,end[1]+.9,end[2]+sin(a+.35)*1.1)
        cylinder_between("Gum bough",start,mid,.22,.15,bark,14)
        cylinder_between("Gum outer limb",mid,end,.15,.074,bark,12)
        cylinder_between("Fine high branch",end,tip,.074,.024,bark,9)
        branch_ends.extend([mid,end,tip])

    # Overlapping leaf sprays form airy clusters with small gaps, instead of round topiary balls.
    for i, center in enumerate(branch_ends):
        cx,cy,cz=center
        for k in range(5):
            angle=2*pi*k/5 + i*.61
            spread=.55 + (k%3)*.15
            p=(cx+cos(angle)*spread,cy+(.10 if k%2 else -.12),cz+sin(angle)*spread)
            leaf_cluster("Layered eucalyptus leaf spray",p,
                         (.39+(k%2)*.10,.23,.58+(k%3)*.08),greens[(i+k)%len(greens)],angle)
        # Long hanging leaflets at branch tips break the canopy edge against the sky.
        if i%2==0:
            for k in range(3):
                angle=2*pi*k/3+i
                p=(cx+cos(angle)*.24,cy-.52,cz+sin(angle)*.24)
                leaf_cluster("Drooping gum leaf cluster",p,(.22,.44,.36),greens[(i+k+1)%3],angle)
    # A restrained scattering of cream blossoms is visible at normal camera distance.
    for i in range(16):
        a=i*2.399
        r=1.2+(i%4)*.46
        y=6.0+(i%5)*.42
        x=cos(a)*r
        z=sin(a)*r
        for k in range(4):
            t=a+k*pi/2
            leaf_cluster("Small gum blossom",(x+cos(t)*.10,y+(k%2)*.08,z+sin(t)*.10),
                         (.09,.13,.09),blossom,t)


original_scene = bpy.context.window.scene if bpy.context.window else bpy.context.scene
temp_scene = bpy.data.scenes.new("Gum Hero Export")
try:
    if not bpy.context.window:
        raise RuntimeError("Blender window unavailable; leaving artist scene untouched")
    bpy.context.window.scene = temp_scene
    build()
    objects = [o for o in temp_scene.objects if o.type == "MESH"]
    bpy.ops.object.select_all(action="DESELECT")
    for obj in objects:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = objects[0]
    bpy.ops.object.join()
    tree = bpy.context.object
    tree.name = "Ribbon gum tree"
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    OUT.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.export_scene.gltf(filepath=str(OUT), export_format="GLB", export_yup=True,
                              use_selection=True, use_active_scene=True,
                              export_cameras=False, export_lights=False,
                              export_apply=True, export_materials="EXPORT")
    print(f"Exported {OUT} as one mesh from {len(objects)} authored components with {len(tree.data.polygons)} faces")
finally:
    bpy.context.window.scene = original_scene
    bpy.data.scenes.remove(temp_scene)
