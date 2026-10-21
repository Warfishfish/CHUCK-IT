from pathlib import Path
import re,json
c=Path(__file__).resolve().parent
views=['gameplay','overview','building','character']
rows=[];data={}
for view in views:
 pair={}
 for phase in ['before','after']:
  text=(c/phase/(view+'.metrics.txt')).read_text()
  pair[phase]={k:float(v) for k,v in re.findall(r'(\w+)=([\d.]+)',text)}
 data[view]=pair
 a,b=pair['before'],pair['after']
 rows.append(f"<tr><td>{view.title()}</td><td>{a['mean_ms']:.3f} ms</td><td>{b['mean_ms']:.3f} ms</td><td>{(b['mean_ms']/a['mean_ms']-1)*100:+.1f}%</td><td>{a['p95_ms']:.2f} → {b['p95_ms']:.2f} ms</td></tr>")
(c/'performance.json').write_text(json.dumps(data,indent=2)+'\n')
sections=''.join(f'''<section><h2>{v.title()}</h2><div class="pair"><figure><img src="before/{v}.png"><figcaption>Before</figcaption></figure><figure><img src="after/{v}.png"><figcaption>Implemented update</figcaption></figure></div></section>''' for v in views)
(c/'review.html').write_text('''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>Australian BBQ — Natural graphics</title><style>body{margin:0 auto;max-width:1600px;padding:32px;background:#17251f;color:#edf2e7;font:17px/1.5 system-ui}h1{font-size:36px}h2{margin-top:36px}p{max-width:1000px}a{color:#b8dca7}.pair{display:grid;grid-template-columns:1fr 1fr;gap:18px}figure{margin:0}img{width:100%;border-radius:10px}figcaption{padding:8px}table{border-collapse:collapse}th,td{padding:10px 20px;border-bottom:1px solid #49604c;text-align:left}.badge{background:#36523a;padding:7px 12px;border-radius:7px;display:inline-block} @media(max-width:900px){.pair{grid-template-columns:1fr}body{padding:16px}}</style>
<h1>Australian BBQ · Natural graphics pass</h1><p class="badge">Actual running-game captures · 21 October 2026 · Awaiting review</p>
<p>A calmer green lawn, less yellow sunlight, softer timber and roof detail, rounded gum foliage and fuller clouds. Character designs, collision and gameplay remain unchanged. No generated concept images.</p>
<p>These comparisons use identical camera positions, scene seed, simulation frame, 1280×800 resolution and polished Pop settings.</p>'''+sections+'''<h2>Moving camera</h2><p>Actual 60-frame capture drifting across the lawn. This is a short visual check, not a comprehensive shimmer or performance test.</p><img src="drift.webp" style="max-width:1000px">
<h2>Performance</h2><p>600 wall-clock frame intervals per view, after warmup; uncapped rendering and fixed 60 Hz simulation. Gameplay uses the second alternating before/after pair. Its initial run was 9.00 → 13.00 ms, but two repeat pairs were 9.58 → 9.26 and 9.66 → 9.45 ms; the initial slowdown did not reproduce. The other views are single-run comparisons. An overview capture with a different round state was replaced. Original captures and all timing logs are retained. These are not GPU timings or a guarantee for other hardware.</p><table><tr><th>View</th><th>Before mean</th><th>After mean</th><th>Change</th><th>95th percentile</th></tr>'''+''.join(rows)+'''</table><p><a href="CHECKPOINT.md">Verification, exact changed files and rollback</a> · <a href="performance.json">Raw comparison</a></p></html>''')
print(json.dumps(data,indent=2))
