from pathlib import Path
import re,json
p=Path(__file__).parent; e=p/'evidence'
metrics={}
for phase in ['before','after']:
 for view in ['overview','gameplay','close']:
  text=(e/f'{phase}-{view}.metrics.txt').read_text()
  metrics[phase+'-'+view]={k:float(v) for k,v in re.findall(r'(mean_ms|p95_ms|meshes|materials|mesh_entities|image_cpu_bytes)=([\d.]+)',text)}
rows=''
for v in ['overview','gameplay','close']:
 b,a=metrics['before-'+v],metrics['after-'+v]
 rows+=f"<tr><td>{v}</td><td>{b['mean_ms']:.3f}</td><td>{a['mean_ms']:.3f}</td><td>{100*(a['mean_ms']/b['mean_ms']-1):+.1f}%</td><td>{a['p95_ms']:.3f}</td></tr>"
s="""<!doctype html><meta charset="utf-8"><title>Bush Servo — ground and road</title><style>body{background:#20251f;color:#f1e6d4;font:17px system-ui;margin:0}main{max-width:1280px;margin:auto;padding:28px}h1,h2,a{color:#efbd79}img{width:100%;border-radius:10px}.pair{display:grid;grid-template-columns:1fr 1fr;gap:16px}p{line-height:1.6;max-width:1050px}td,th{padding:10px;border:1px solid #657063}table{border-collapse:collapse}</style><main><h1>Bush Servo · stage 1: ground and road</h1><p>Completed 27 October 2026. Actual running-game captures; no new concept images. Not committed, pushed or published.</p><img src="evidence/after-close.png"><p>Warmer red earth, worn asphalt, dusty uneven shoulders, fine cracks, subtle tyre wear and small embedded stones. The yellow delivery ring stays visible, while the pale circular pad is replaced with continuous earth. Existing shelter, lighting, game rules and collisions are preserved.</p>"""
for v in ['overview','gameplay','close']:
 s+=f'<h2>{v.title()} — matched cameras</h2><div class="pair"><div><p>Before</p><img src="evidence/before-{v}.png"></div><div><p>After</p><img src="evidence/after-{v}.png"></div></div>'
s+="""<h2>Moving-camera and delivery check</h2><img src="evidence/moving.webp"><p>75 actual game frames. Controlled fixture exercises existing carry/delivery rules: +100, then no carried victim remaining. This is not a human multiplayer playtest.</p><h2>Performance</h2><p>Same cameras, polished/pop settings and 1280×800 resolution; 600 wall-clock samples per view. Baseline captured 26 October, final captures 27 October; session/thermal variation limits small percentage comparisons. Final captures taken without a build running. Black interrupted captures and their timings were excluded.</p><table><tr><th>View</th><th>Before mean ms</th><th>After mean ms</th><th>Change</th><th>After p95 ms</th></tr>"""+rows+"""</table><p>One 192-triangle road mesh with no extra shader. Dirt stays 512px; road rises from 512px to 1024px. Tiny shoulder stones are decorative, below boot height. Existing mipmaps reduce distant texture noise.</p><h2>Scope</h2><p>Stage 1 only. Plants, sandstone borders, new trees, buildings, lighting overhaul and additional props remain later stages. Native release and focused collision tests passed; browser compatibility compile recorded in the checkpoint. The published browser assets remain unchanged pending review.</p><p><a href="CHECKPOINT.md">Checkpoint and rollback instructions</a></p></main>"""
(p/'review.html').write_text(s)
(p/'performance.json').write_text(json.dumps(metrics,indent=2))
print(rows)
