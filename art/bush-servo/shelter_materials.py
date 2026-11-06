"""Small baked texture maps for the shelter; ordinary PNGs usable by glTF/WebGPU.
No paid assets or new dependencies. Colours are authored in linear light.
"""
import bpy, math, struct, zlib
import numpy as np
from pathlib import Path
SIZE=(128,512)
GRAIN_STRENGTH=.075
RUST_COVERAGE=(.75,.60,.20,.40,.12,.85)
NORMAL_STRENGTH=.08

def png(path,linear,raw=False):
 c=np.clip(linear,0,1)
 if not raw:c=np.where(c<=.0031308,12.92*c,1.055*np.power(c,1/2.4)-.055)
 a=np.full((*c.shape[:2],1),255,dtype=np.uint8);rgba=np.concatenate(((c*255+.5).astype(np.uint8),a),axis=2)
 def chunk(k,v):return struct.pack('>I',len(v))+k+v+struct.pack('>I',zlib.crc32(k+v)&0xffffffff)
 h,w=c.shape[:2];data=b''.join(b'\0'+row.tobytes() for row in rgba[::-1]);path.write_bytes(b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',w,h,8,6,0,0,0))+chunk(b'IDAT',zlib.compress(data,9))+chunk(b'IEND',b''))
 image=bpy.data.images.load(str(path),check_existing=False)
 if raw:image.colorspace_settings.name='Non-Color'
 image.pack();return image

def attach(mat,image,socket='Base Color'):
 bs=next(n for n in mat.node_tree.nodes if n.type=='BSDF_PRINCIPLED')
 for link in list(bs.inputs[socket].links):mat.node_tree.links.remove(link)
 n=mat.node_tree.nodes.new('ShaderNodeTexImage');n.image=image;mat.node_tree.links.new(n.outputs['Color'],bs.inputs[socket]);return n

def apply(wood,roof,end,root):
 folder=Path(root)/'art/bush-servo/shelter-textures';folder.mkdir(exist_ok=True)
 w,h=SIZE;y,x=np.mgrid[0:h,0:w];x=x/w;y=y/h
 rng=np.random.default_rng(41104)
 # Flow bends around broad knots; long fibres remain legible after mip filtering.
 bend=.018*np.sin(y*math.tau*2)+.025*np.sin(y*math.tau)
 for cx,cy in [(.32,.31),(.74,.78)]:bend+=.065*np.exp(-((x-cx)/.19)**2-((y-cy)/.09)**2)
 flow=(x+bend)*math.tau
 grain=np.sin(flow*17)+.28*np.sin(flow*37)+.18*np.sin(flow*61)
 weather=.065*np.sin(flow*3+y*8)+rng.uniform(-.015,.015,(h,w))
 split=np.maximum(0,np.sin(flow*23))**24*.09
 tone=1+GRAIN_STRENGTH*grain+weather-split
 for k,m in enumerate(wood):
  base=np.array([.38+k*.016,.29+k*.013,.205+k*.010]);colour=tone[:,:,None]*base
  attach(m,png(folder/f'timber-{k}.png',colour))
  bs=next(n for n in m.node_tree.nodes if n.type=='BSDF_PRINCIPLED');bs.inputs['Roughness'].default_value=.86
 # One shared, restrained tangent normal map for all structural timber.
 gx=np.cos(flow*17)*NORMAL_STRENGTH;gy=np.sin(y*math.tau)*.015
 normal=np.stack((-gx,-gy,np.ones((h,w))),axis=2);normal/=np.linalg.norm(normal,axis=2)[:,:,None]
 ni=png(folder/'timber-normal.png',normal*.5+.5,raw=True)
 for m in wood:
  tex=m.node_tree.nodes.new('ShaderNodeTexImage');tex.image=ni
  node=m.node_tree.nodes.new('ShaderNodeNormalMap');node.inputs['Strength'].default_value=.45
  m.node_tree.links.new(tex.outputs['Color'],node.inputs['Color']);m.node_tree.links.new(node.outputs['Normal'],next(n for n in m.node_tree.nodes if n.type=='BSDF_PRINCIPLED').inputs['Normal'])
 r=np.sqrt(((x-.46)*1.05)**2+((y-.51)*.92)**2);rings=1+.10*np.sin(r*135)+.025*np.sin(r*281)
 attach(end,png(folder/'endgrain.png',rings[:,:,None]*np.array([.35,.265,.18])))
 # Mottled contiguous oxidation and rain streaks, rather than random face colours.
 for k,m in enumerate(roof):
  broad=(np.sin(x*12+y*5+k)+np.sin(x*27-y*8+k*.73)+.5*np.sin(x*7+y*17))/2.5
  rust=np.clip((broad+.40)*1.3,0,1)*RUST_COVERAGE[k]
  rust+=.12*np.exp(-((y-.92)/.10)**2)*(1+np.sin(x*32+k))*.5
  metal=np.array([.43,.45,.42])*(.88+k*.025);oxide=np.array([.38,.13,.045])
  c=metal*(1-rust[:,:,None])+oxide*rust[:,:,None]
  c*=1+(.035*np.sin(x*80+np.sin(y*12))+.018*rng.uniform(-1,1,(h,w)))[:,:,None]
  attach(m,png(folder/f'roof-{k}.png',c))
  bs=next(n for n in m.node_tree.nodes if n.type=='BSDF_PRINCIPLED');bs.inputs['Roughness'].default_value=.80;bs.inputs['Metallic'].default_value=.20
 print('Baked shelter maps',len(list(folder.glob('*.png'))))
