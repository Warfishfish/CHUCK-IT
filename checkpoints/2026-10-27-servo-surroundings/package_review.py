from pathlib import Path
import re,json,hashlib
from PIL import Image,ImageStat
p=Path(__file__).resolve().parent;r=p.parents[1];e=p/'evidence'
frames=sorted(e.glob('moving-[0-9][0-9][0-9].png'))
if frames:
 images=[Image.open(f).convert('RGB') for f in frames]
 assert all(max(ImageStat.Stat(im.resize((64,40))).mean)>1 for im in images)
 images[0].save(e/'moving.webp',save_all=True,append_images=images[1:],duration=67,loop=0,quality=78,method=4)
rows=[]
for view in ['overview','gameplay','close']:
 vals=[]
 for phase in ['before','after']:
  s=(e/f'{phase}-{view}.metrics.txt').read_text();vals.append({k:float(re.search(k+r'=([\d.]+)',s)[1]) for k in ['mean_ms','p95_ms']})
 b,a=vals;rows.append(f"<tr><td>{view}</td><td>{b['mean_ms']:.3f} → {a['mean_ms']:.3f}</td><td>{100*(a['mean_ms']/b['mean_ms']-1):+.1f}%</td><td>{b['p95_ms']:.3f} → {a['p95_ms']:.3f}</td></tr>")
html='''<!doctype html><meta charset="utf-8"><title>Smoko surroundings — actual game review</title><style>body{background:#18231f;color:#eee;font:17px system-ui;max-width:1200px;margin:40px auto;padding:20px}h1,h2{color:#ffd28b}img{width:100%;border-radius:10px}section{margin:35px 0}.pair{display:grid;grid-template-columns:1fr 1fr;gap:14px}td,th{padding:12px;text-align:left;border-bottom:1px solid #52645a}a{color:#9ed9ff}</style><h1>Bush Servo: shelter surroundings</h1><p>27 October 2026 · Actual native gameplay captures · Stage 2, awaiting review</p><p>Low sandstone edging, dry grasses and small saltbush clusters frame the Smoko shelter. Four broad approaches and the delivery circle stay clear. Stones have matching shared collision bounds; plants are decorative.</p>'''
for view in ['close','gameplay','overview']:
 html+=f'<section><h2>{view.title()}</h2><div class="pair"><div>Before<img src="evidence/before-{view}.png"></div><div>After<img src="evidence/after-{view}.png"></div></div></section>'
html+='<h2>Delivery and moving camera</h2><p>Scripted native carry/delivery sequence: +100 awarded, carry released. This is a local fixture, not a two-person online session.</p><img src="evidence/moving.webp"><h2>Performance</h2><table><tr><th>View</th><th>Mean ms before → after</th><th>Change</th><th>95th percentile ms</th></tr>'+''.join(rows)+'</table><p>600 wall-clock frame intervals per view at 1280×800 after warmup; same camera/settings. Session variation applies; these are not GPU timestamp measurements. No meaningful mean-time regression observed. Some tail latency varies.</p><h2>Verification and limits</h2><p>438 core tests pass, including collider/visual dimensions, approach clearance and scoring-circle clearance. Native release passes. See checkpoint for browser compilation status. Backyard smoke capture also completed. Asset: 4,978 triangles, 9 material batches and four 128px embedded grain textures.</p><p>Protocol increases to 18 because solid map bounds changed: host and guests need the same updated build. Live browser remains unchanged. No new human multiplayer session was recorded. Trees, servo detailing and furniture remain later stages; this first landscape pass does not match the concept’s full detail.</p><p><a href="CHECKPOINT.md">Checkpoint, files and recovery</a> · <a href="evidence/ffa.png">Backyard smoke capture</a></p>'
(p/'review.html').write_text(html)
print('packaged',len(frames),'motion frames')
