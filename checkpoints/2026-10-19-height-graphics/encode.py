from pathlib import Path
from PIL import Image
c=Path(__file__).resolve().parent
frames=[Image.open(c/f'climb-{i:03}.png').convert('RGB').resize((960,600),Image.Resampling.LANCZOS) for i in range(75)]
assert all(max(f.getextrema()[0])>10 for f in frames)
frames[0].save(c/'climb.webp',save_all=True,append_images=frames[1:],duration=[67,67,66]*25,loop=0,quality=85)
print('Encoded actual five-second climbing replay.')
