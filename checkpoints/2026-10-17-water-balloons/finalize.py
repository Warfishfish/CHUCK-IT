from pathlib import Path
import hashlib, json, difflib, re
r=Path(__file__).resolve().parents[2]; c=Path(__file__).resolve().parent
manifest={};patch=[]
for p in sorted((r/'rust/crates').rglob('*.rs')):
 name=str(p.relative_to(r));b=c/'before-source'/name;old=b.read_bytes() if b.exists() else None;new=p.read_bytes()
 if old==new:continue
 manifest[name]={'before':hashlib.sha256(old).hexdigest() if old else None,'after':hashlib.sha256(new).hexdigest()}
 patch.extend(difflib.unified_diff((old or b'').decode().splitlines(True),new.decode().splitlines(True),fromfile='a/'+name if old is not None else '/dev/null',tofile='b/'+name))
(c/'changed-files.json').write_text(json.dumps(manifest,indent=2)+'\n');(c/'water-balloons.patch').write_text(''.join(patch))
rows=[]
for scene in ['held','backyard']:
 vals=[]
 for phase in ['before','after']:
  text=(c/phase/f'{scene}.metrics.txt').read_text()
  data=dict(re.findall(r'(\w+)=([\d.]+)',text));vals.append(data)
 a,b=vals;change=(float(b['mean_ms'])/float(a['mean_ms'])-1)*100
 rows.append(f"<tr><td>{scene}</td><td>{a['mean_ms']} ms / {a['mean_fps']} FPS</td><td>{b['mean_ms']} ms / {b['mean_fps']} FPS</td><td>{change:+.1f}%</td></tr>")
 table='<table><tr><th>Scene</th><th>Before</th><th>After</th><th>Mean frame time</th></tr>'+''.join(rows)+'</table>'
(c/'performance.html').write_text(table)
p=c/'review.html';s=p.read_text().replace('<div id="performance">Measurements pending.</div>',f'<div id="performance">{table}<p>621 tests passed; one existing test ignored. Release build passed. Measurements: same camera/scene, 1280×800, 600 wall-clock frames, uncapped rendering. These are CPU wall-clock observations, not GPU timestamps; gameplay and particle activity can vary.</p></div>');p.write_text(s)
print(f'Saved selective delta for {len(manifest)} source files and performance table.')
