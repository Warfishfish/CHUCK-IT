from PIL import Image
import numpy as np
from pathlib import Path
rng=np.random.default_rng(271026)
y,x=np.mgrid[0:128,0:128]
noise=rng.normal(0,2.5,(128,128))
strata=2*np.sin(y*.35+np.sin(x*.04)*1.5)+1.5*np.sin(y*.9)
v=np.clip(225+noise+strata,205,240).astype('uint8')
for i,c in enumerate([(198,153,107),(208,166,120),(217,177,131),(202,158,111)]):
 rgb=np.clip(v[:,:,None].astype(float)/225*np.array(c),0,255).astype('uint8')
 Image.fromarray(rgb).save(Path(__file__).with_name(f'sandstone-grain-{i}.png'))
