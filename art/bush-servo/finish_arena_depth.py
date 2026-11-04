"""Local, non-destructive runtime vertex tint after Blender export.
Reads the verified pre-pass archive, never compounds its own output. Geometry, UVs,
textures and materials remain byte-identical; only a local COLOR_0 is added.
Reapply after re-exporting via the same transform function with a fresh input GLB.
"""
import json, struct, tarfile, pathlib, hashlib
ROOT=pathlib.Path(__file__).resolve().parents[2]
BACKUP=ROOT.parent/'australian-bbq-before-arena-depth-2026-10-31.tar.gz'
TIMBER=(.92,.97,1.0) # less orange, sun-bleached warm neutral
SANDSTONE=(.92,.97,1.0) # quieter ochre, retain contrast against earth

def finish(data,kind):
 n=struct.unpack_from('<I',data,12)[0];doc=json.loads(data[20:20+n]);blob=bytearray(data[28+n:]);oldlen=len(blob);changed=0;outside=0
 for node in doc['nodes']:
  if 'mesh' not in node:continue
  assert 'matrix' not in node and 'rotation' not in node and 'scale' not in node
  offset=node.get('translation',[0,0,0]);offset=[*offset]
  if kind=='shelter':offset[2]+=10
  for prim in doc['meshes'][node['mesh']]['primitives']:
   name=doc['materials'][prim['material']].get('name','')
   tint=TIMBER if 'timber' in name.lower() else SANDSTONE if 'sandstone' in name.lower() else None
   if tint is None:continue
   assert 'COLOR_0' not in prim['attributes']
   a=doc['accessors'][prim['attributes']['POSITION']];v=doc['bufferViews'][a['bufferView']];assert a['componentType']==5126 and a['type']=='VEC3'
   colors=[]
   for i in range(a['count']):
    x,y,z=struct.unpack_from('<3f',blob,v.get('byteOffset',0)+a.get('byteOffset',0)+i*v.get('byteStride',12));x+=offset[0];z+=offset[2]
    # Shelter plus both picnic areas; smoothly neutral before arena boundary.
    shelter=max(0,min(1,(6-abs(x)),(z-4),(17-z)))
    picnic=max(0,min(1,(4-abs(abs(x)-8.5)),(z+5.5),(1.5-z)))
    k=max(shelter,picnic);k=k*k*(3-2*k)
    colors.extend([1+(t-1)*k for t in tint]+[1])
    if k>0:changed+=1
    else:outside+=1;assert colors[-4:]==[1,1,1,1]
   while len(blob)%4:blob.append(0)
   off=len(blob);packed=struct.pack('<'+'f'*len(colors),*colors);blob.extend(packed)
   view=len(doc['bufferViews']);doc['bufferViews'].append({'buffer':0,'byteOffset':off,'byteLength':len(packed),'target':34962})
   idx=len(doc['accessors']);doc['accessors'].append({'bufferView':view,'componentType':5126,'count':a['count'],'type':'VEC4'})
   prim['attributes']['COLOR_0']=idx
 assert bytes(blob[:oldlen])==data[28+n:]
 doc['buffers'][0]['byteLength']=len(blob)
 j=json.dumps(doc,separators=(',',':')).encode();j+=b' '*((-len(j))%4)
 out=struct.pack('<III',0x46546c67,2,28+len(j)+len(blob))+struct.pack('<II',len(j),0x4e4f534a)+j+struct.pack('<II',len(blob),0x004e4942)+blob
 return out,{'tinted_vertices':changed,'neutral_vertices':outside,'added_bytes':len(out)-len(data),'original_binary_unchanged':True}
if __name__=='__main__':
 report={}
 with tarfile.open(BACKUP) as archive:
  for kind in ['shelter','props','surroundings']:
   path=f'rust/crates/bbq_app/assets/models/smoko_{kind}.glb';data=archive.extractfile(path).read();out,report[kind]=finish(data,kind);(ROOT/path).write_bytes(out)
 (ROOT/'checkpoints/2026-10-31-arena-depth/asset-costs.json').write_text(json.dumps(report,indent=2));print(report)
