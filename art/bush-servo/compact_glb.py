"""Remove unused UV streams from textureless static environment primitives.

Preserves retained accessor/image bytes; no quantisation, mesh simplification,
compression extension or runtime decoder is required.
"""
import json, struct
from pathlib import Path

def compact(path):
    path=Path(path);raw=path.read_bytes()
    assert raw[:4]==b'glTF'
    size=struct.unpack_from('<I',raw,12)[0]
    g=json.loads(raw[20:20+size]);binary=raw[28+size:]
    assert not g.get('animations') and not g.get('skins')
    assert len(g['buffers'])==1 and 'uri' not in g['buffers'][0]
    def textured(v):
        if isinstance(v,dict):return any(k.endswith('Texture') or textured(x) for k,x in v.items())
        if isinstance(v,list):return any(textured(x) for x in v)
        return False
    removed=0
    for mesh in g['meshes']:
        for p in mesh['primitives']:
            if not textured(g['materials'][p['material']]):
                for attr in list(p['attributes']):
                    if attr.startswith('TEXCOORD_'):
                        del p['attributes'][attr];removed+=1
    used=set()
    for mesh in g['meshes']:
        for p in mesh['primitives']:
            used.update(p['attributes'].values());used.add(p['indices'])
            assert not p.get('targets')
    accessors=[g['accessors'][i] for i in sorted(used)]
    amap={old:new for new,old in enumerate(sorted(used))}
    for a in accessors:assert 'sparse' not in a
    views={a['bufferView'] for a in accessors}|{i['bufferView'] for i in g.get('images',[])}
    newviews=[];out=bytearray();vmap={}
    for old in sorted(views):
        v=dict(g['bufferViews'][old]);begin=v.get('byteOffset',0)
        data=binary[begin:begin+v['byteLength']]
        out.extend(b'\0'*((-len(out))%4));v['byteOffset']=len(out)
        out.extend(data);vmap[old]=len(newviews);newviews.append(v)
        assert bytes(out[v['byteOffset']:v['byteOffset']+v['byteLength']])==data
    for a in accessors:a['bufferView']=vmap[a['bufferView']]
    for i in g.get('images',[]):i['bufferView']=vmap[i['bufferView']]
    for mesh in g['meshes']:
        for p in mesh['primitives']:
            p['indices']=amap[p['indices']]
            p['attributes']={k:amap[v] for k,v in p['attributes'].items()}
    g['accessors']=accessors;g['bufferViews']=newviews;g['buffers'][0]['byteLength']=len(out)
    j=json.dumps(g,separators=(',',':')).encode();j+=b' '*((-len(j))%4);out+=b'\0'*((-len(out))%4)
    result=struct.pack('<4sII',b'glTF',2,28+len(j)+len(out))+struct.pack('<I4s',len(j),b'JSON')+j+struct.pack('<I4s',len(out),b'BIN\0')+out
    path.write_bytes(result)
    return {'unused_uv_streams_removed':removed,'bytes_saved':len(raw)-len(result),'final_bytes':len(result)}
