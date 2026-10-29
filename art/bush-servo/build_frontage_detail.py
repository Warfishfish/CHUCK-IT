"""Concept-match frontage relief. Run in Blender; original Stage 4 file is the source.

Reuses the licensed base asset/materials; creates a separate editable scene.
Everything remains beyond the playable fence (Blender y > 24 m).
The authoring .blend keeps named objects; runtime export joins copies.
"""
import bpy, math, json
from pathlib import Path
from mathutils import Vector
ROOT = Path('/Users/marcus/Claude/australian-bbq')
BEVEL = .012
DISPLAY_Y = 25.19
HOSE_RADIUS = .29
scene = bpy.data.scenes.new('Bush Servo Concept Frontage')
bpy.context.window.scene = scene
with bpy.data.libraries.load(str(ROOT/'art/bush-servo/smoko-servo.blend'), link=False) as (src, dst):
    dst.objects = src.objects
base = next(o for o in dst.objects if o and o.type == 'MESH')
scene.collection.objects.link(base)
objects = [base]
def material(prefix):
    return next(m for m in base.data.materials if m.name.startswith(prefix))
cream = material('Servo warm cream paint')
ivory = material('Servo ivory trim')
red = material('Servo faded red enamel')
dark = material('Servo recess and rubber')
wood = material('Servo porch timber')
steel = material('Servo zinc trim')
teal = material('Servo shaded blue glass')
font = bpy.data.fonts.load(str(ROOT/'rust/crates/bbq_app/assets/fonts/BowlbyOne-Regular.ttf'))
def add(o, name, mat):
    o.name = name
    o.data.materials.append(mat)
    objects.append(o)
    return o
def box(name, p, size, mat, bevel=BEVEL):
    bpy.ops.mesh.primitive_cube_add(size=1, location=p)
    o=add(bpy.context.object,name,mat);o.dimensions=size
    bpy.ops.object.transform_apply(location=False,rotation=False,scale=True)
    if bevel:
        mod=o.modifiers.new('Worn edges','BEVEL');mod.width=bevel;mod.segments=1
        bpy.ops.object.modifier_apply(modifier=mod.name)
    return o
def rod(name,a,b,r,mat,sides=8):
    a,b=Vector(a),Vector(b)
    bpy.ops.mesh.primitive_cylinder_add(vertices=sides,radius=r,depth=(b-a).length,location=(a+b)/2)
    o=add(bpy.context.object,name,mat);o.rotation_euler=(b-a).to_track_quat('Z','Y').to_euler()
    return o
def label(name,words,p,width,mat):
    cu=bpy.data.curves.new(name,'FONT');cu.body=words;cu.font=font;cu.align_x='CENTER';cu.size=1;cu.resolution_u=2;cu.extrude=0
    o=bpy.data.objects.new(name,cu);scene.collection.objects.link(o);objects.append(o)
    o.location=p;o.rotation_euler=(math.pi/2,0,0);cu.materials.append(mat)
    bpy.context.view_layer.objects.active=o;bpy.ops.object.select_all(action='DESELECT');o.select_set(True)
    bpy.ops.object.convert(target='MESH');bpy.context.view_layer.update();o.scale*=width/max(o.dimensions.x,.001)
    return o
# Shallow shop-window dioramas: opaque geometry avoids transparency cost/sorting.
# Project only a few centimetres in front of the old glazing, behind the frames.
for wi,(x,width) in enumerate([(-15.8,5.1),(-9.55,5.1),(.15,5.25)]):
    for shelf_z in [1.30,1.95,2.59]:
        box('Display shelf',(x,DISPLAY_Y,shelf_z),(width-.3,.10,.06),wood,0)
    for row,z in enumerate([1.35,2.00]):
        for k in range(11):
            xx=x-width*.43+k*width*.086
            if abs(xx-x)<.12:continue  # clear the central mullion
            h=[.30,.39,.34,.43][(k+row+wi)%4]
            colour=[teal,cream,red,ivory][(k//2+row+wi)%4]
            box('Shop tin or pantry carton',(xx,DISPLAY_Y,z+h/2),(.22,.07,h),colour,.008)
            box('Simple pack label',(xx,DISPLAY_Y-.042,z+h*.55),(.15,.008,h*.27),ivory,0)
    # Larger display goods read as shapes from normal gameplay distance.
    for k in range(5):
        xx=x-1.65+k*.81
        box('Upper shelf supplies',(xx,DISPLAY_Y,2.79),(.47,.08,.34),[cream,red,teal][(k+wi)%3],.01)
    # Repeated horizontal glazing strip breaks up the broad flat panel.
    box('Window transom',(x,25.12,1.92),(width,.08,.05),cream,0)
# Concept-style stacked service board near the left veranda corner.
box('Service board timber frame',(-19.0,24.81,2.10),(1.20,.10,2.15),wood,.025)
for i,word in enumerate(['FUEL','DRINKS','SNACKS','TYRES','MATES']):
    z=2.93-i*.40
    box('Service board ivory slat',(-19.,24.745,z),(1.06,.035,.34),cream,.01)
    label('Service board '+word,word,(-19.,24.72,z-.09),.88,teal)
# Joinery and warm sheltered recesses under the canopy.
for x in [-18.3,-15.1,-11.9,-8.7,-2.3,.9]:
    box('Veranda underside rafter',(x,24.98,3.63),(.10,1.55,.13),wood,0)
for x in [-16.8,-8.,.8]:
    box('Under awning lamp backing',(x,25.18,3.47),(.58,.27,.06),dark)
    box('Under awning lamp lens',(x,25.18,3.43),(.45,.18,.04),ivory)
# Hose reel: narrow solid rings with only 16 segments, no transparent cards.
box('Air hose wall bracket',(-3.25,25.23,1.55),(.22,.16,.20),steel)
for depth in [25.06,25.105,25.15]:
    for i in range(16):
        a=i*math.tau/16;b=(i+1)*math.tau/16
        rod('Coiled air hose',(-3.25+math.cos(a)*HOSE_RADIUS,depth,1.55+math.sin(a)*HOSE_RADIUS),(-3.25+math.cos(b)*HOSE_RADIUS,depth,1.55+math.sin(b)*HOSE_RADIUS),.018,dark,5)
rod('Hose dangling end',(-3.25,25.06,1.26),(-3.06,25.06,.71),.018,dark,5)
rod('Air chuck',(-3.06,25.06,.71),(-3.01,25.06,.63),.028,steel,6)
# Crates and oil cans kept completely outside the fence, under the veranda.
def crate(x,y,z):
    for dx in [-.40,.40]:
        for dy in [-.25,.25]:box('Crate corner',(x+dx,y+dy,z+.27),(.06,.06,.54),wood,0)
    for h in [.08,.25,.42]:
        for dy in [-.28,.28]:box('Crate long slat',(x,y+dy,z+h),(.86,.055,.12),cream,.008)
        for dx in [-.43,.43]:box('Crate end slat',(x+dx,y,z+h),(.055,.52,.12),cream,.008)
    box('Crate base',(x,y,z+.03),(.86,.56,.06),wood,0)
crate(-12.72,24.77,.25);crate(-12.72,24.77,.80)
crate(2.1,24.77,.25)
for x in [1.86,2.17,2.40]:
    box('Oil can',(x,24.77,.97),(.20,.20,.35),teal,.025)
    box('Oil can cap',(x,24.77,1.17),(.08,.08,.05),steel,.008)
    box('Oil can label',(x,24.66,.99),(.13,.012,.13),ivory,0)
# Light roof-edge fasteners and freezer vent relief; no extra textures.
for x in range(-19,5,2):
    box('Fascia fixing',(x,24.01,3.51),(.035,.018,.035),steel,0)
for i in range(6):box('Freezer vent slat',(-18.55+i*.07,24.30,.38),(.025,.018,.22),dark,0)
for x in [-18.7,-17.3]:box('Freezer lid latch',(x,24.27,1.30),(.07,.035,.14),steel,.008)

bpy.context.view_layer.update()
new_objects=[o for o in objects if o != base]
minimum_y=min((o.matrix_world@Vector(c)).y for o in new_objects for c in o.bound_box)
assert minimum_y>24.0,minimum_y
triangles=lambda obs:sum(sum(len(p.vertices)-2 for p in o.data.polygons) for o in obs)
report={'added_objects':len(new_objects),'added_triangles':triangles(new_objects),'base_triangles':triangles([base]),'minimum_added_y':minimum_y,'new_textures':0}
assert report['added_triangles']<12000,report
# Save a compact authoring scene with named details and the unchanged base mesh.
bpy.data.libraries.write(str(ROOT/'art/bush-servo/smoko-frontage-detail.blend'),{scene},fake_user=True,compress=True)
bpy.ops.object.select_all(action='DESELECT')
copies=[]
for source in objects:
    o=source.copy();o.data=source.data.copy();scene.collection.objects.link(o);o.select_set(True);copies.append(o)
bpy.context.view_layer.objects.active=copies[0];bpy.ops.object.join()
runtime=bpy.context.object;runtime.name='BushServo_DetailedFrontage'
bpy.ops.object.transform_apply(location=True,rotation=True,scale=True)
asset=ROOT/'rust/crates/bbq_app/assets/models/smoko_servo.glb'
bpy.ops.export_scene.gltf(filepath=str(asset),use_selection=True,use_active_scene=True,export_apply=True)
import runpy
report['compact']=runpy.run_path(str(ROOT/'art/bush-servo/compact_glb.py'))['compact'](asset)
report.update(asset_bytes=asset.stat().st_size,material_slots=len(runtime.data.materials))
(ROOT/'checkpoints/2026-10-29-servo-detail/frontage-report.json').write_text(json.dumps(report,indent=2))
print(json.dumps(report,indent=2))
