from pathlib import Path
import json,re,hashlib
from PIL import Image,ImageStat
c=Path(__file__).parent;e=c/'evidence';r=c.parents[1]
frames=sorted(e.glob('moving-[0-9][0-9][0-9].png'))
assert len(frames)>=74
images=[Image.open(p).convert('RGB') for p in frames]
for im in images: assert max(ImageStat.Stat(im.resize((64,40))).mean)>1
images[0].save(e/'moving.webp',save_all=True,append_images=images[1:],duration=67,loop=0,quality=78,method=4)
rows=[];results={}
for view in ['overview','gameplay','close','servo']:
    pair=[]
    for phase in ['before','after']:
        t=(e/f'{phase}-{view}.metrics.txt').read_text()
        pair.append({k:float(v) for k,v in re.findall(r'(mean_ms|p95_ms|image_cpu_bytes|materials|mesh_entities)=([0-9.]+)',t)})
    b,a=pair;results[view]={'before':b,'after':a}
    rows.append(f'<tr><td>{view}</td><td>{b["mean_ms"]:.3f} → {a["mean_ms"]:.3f}</td><td>{100*(a["mean_ms"]/b["mean_ms"]-1):+.1f}%</td><td>{b["p95_ms"]:.3f} → {a["p95_ms"]:.3f}</td></tr>')
(e/'comparison.json').write_text(json.dumps(results,indent=2))
html='''<!doctype html><meta charset="utf-8"><title>Bush Servo — lighting and materials</title><style>body{background:#252a24;color:#f4eee2;font:17px system-ui;max-width:1200px;margin:30px auto;padding:20px}h1,h2{color:#ffd195}img{width:100%;border-radius:10px}.pair{display:grid;grid-template-columns:1fr 1fr;gap:16px}section{margin:32px 0}td,th{padding:12px;text-align:left;border-bottom:1px solid #64705a}a{color:#a6dfff}@media(max-width:700px){.pair{grid-template-columns:1fr}}</style><h1>Bush Servo: lighting and materials</h1><p>Stage 5 · Actual implemented native game captures · Approved and published</p><p>Warmer directional sunlight, brighter shelter and veranda shade, gentler contrast and a restrained distant haze colour. Imported environment textures now have distance filtering, including colour-correct smaller texture levels, to reduce fine-pattern shimmer. Existing models, collision, gameplay and other modes are preserved.</p>'''
for name,label in [('overview','Arena'),('gameplay','Gameplay view'),('close','Shelter'),('servo','Servo and materials')]:
    html+=f'<section><h2>{label}</h2><div class="pair"><div>Before — Stage 4<img src="evidence/before-{name}.png"></div><div>After — Stage 5<img src="evidence/after-{name}.png"></div></div></section>'
html+='''<h2>Moving-camera check</h2><img src="evidence/moving.webp"><p>Scripted native Smoko delivery and camera movement. See checkpoint for measured outcome and test details.</p><h2>Performance</h2><table><tr><th>View</th><th>Mean milliseconds</th><th>Change</th><th>p95 milliseconds</th></tr>'''+''.join(rows)+'''</table><p>600 wall-clock frame intervals per view at 1280×800, matched cameras and warmup, with builds stopped. These are not GPU timestamps or browser performance measurements.</p><h2>Limits and next step</h2><p>This is a focused finishing pass, not an exact match to the concept render. The existing stylised models and foliage remain; filtering reduces texture aliasing but cannot remove silhouette shimmer. No new props or obstacles were added. Stage 5 is now published to the existing Cloudflare browser link; refresh to load the update.</p><p><a href="CHECKPOINT.md">Verification and recovery checkpoint</a> · <a href="evidence/ffa.png">Backyard regression capture</a></p><h2>Local testing</h2><pre>cd /Users/marcus/Claude/australian-bbq/rust
~/.cargo/bin/cargo run -p bbq_app --release -- --menu</pre><p>Choose Maps / Game modes → Smoko Yard.</p>'''
(c/'review.html').write_text(html)
files=['rust/crates/bbq_app/src/'+n for n in ['main.rs','lighting.rs','yard_scene.rs','servo_finish.rs']]
(c/'reviewed-sha256.json').write_text(json.dumps({p:hashlib.sha256((r/p).read_bytes()).hexdigest() for p in files},indent=2))
print(json.dumps(results,indent=2))
