"""Create six continuous toy surfaces in Blender without touching scene objects.
Game-native coordinates: Y up. Shared mesh data, no textures or paid generation.
"""
import bpy, math, json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
profile=[(-.205,.0),(-.2,.155),(-.185,.185),(-.16,.19),(-.135,.175),(-.105,.135),(-.06,.103),(0,.098),(.25,.104),(.55,.096),(.78,.084),(.84,.081),(.885,.1),(.94,.108),(.99,.103),(1.045,.086),(1.09,.054),(1.12,0)]
# Smooth monotone interpolation between profile keys keeps the silhouette rounded.
def radius(y):
 for (a,ra),(b,rb) in zip(profile,profile[1:]):
  if y<=b:
   t=max(0,min(1,(y-a)/(b-a))); t=t*t*(3-2*t)
   return ra+(rb-ra)*t
 return 0
result={}
for skin in ['Smooth','Studded','Ribbed','Rilled','Spiral','Veiny']:
 vertices=[];faces=[]
 ys=sorted(set([a for a,r in profile]+[-.205+i*1.325/96 for i in range(97)]))
 for y in ys:
  envelope=max(0,min(1,(y+.04)/.09,(.83-y)/.1))
  for j in range(24):
   a=j*math.tau/24;r=radius(y)
   if skin=='Studded':r+=envelope*.004*max(0,math.cos(a*6+math.sin(y*14)))**6*max(0,math.cos(y*55))**4
   elif skin=='Ribbed':r+=envelope*.0045*(.5+.5*math.cos(y*65))
   elif skin=='Rilled':r+=envelope*.0025*math.cos(a*12)
   elif skin=='Spiral':r+=envelope*.004*max(0,math.cos(a-y*24))**4
   elif skin=='Veiny':r+=envelope*.003*max(0,math.cos(a*3+.8*math.sin(y*9)))**8
   vertices.append((r*math.cos(a),y,r*math.sin(a)))
 for i in range(len(ys)-1):
  for j in range(24):
   a=i*24+j;b=i*24+(j+1)%24;c=(i+1)*24+(j+1)%24;d=(i+1)*24+j
   faces.append((a,d,c,b))
 mesh=bpy.data.meshes.new('BBQ Silicone export '+skin)
 mesh.from_pydata(vertices,[],faces);mesh.validate();mesh.update()
 for f in mesh.polygons:f.use_smooth=True
 mesh.calc_loop_triangles()
 result[skin]={'positions':[[round(x,6) for x in v.co] for v in mesh.vertices], 'normals':[[round(x,6) for x in v.normal] for v in mesh.vertices], 'indices':[i for t in mesh.loop_triangles for i in t.vertices]}
 bpy.data.meshes.remove(mesh)
out=ROOT/'rust/crates/bbq_app/assets/models/silicone.json';out.write_text(json.dumps(result,separators=(',',':')))
print('Exported',len(result),'surfaces,',len(vertices),'vertices and',len(result['Smooth']['indices'])//3,'triangles each,',out.stat().st_size,'bytes')
