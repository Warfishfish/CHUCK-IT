from pathlib import Path
import json,hashlib,shutil,re
r=Path(__file__).resolve().parents[2];p=Path(__file__).parent
for name in ['tests','native','web']:shutil.copy2('/tmp/bbq-tyre-'+name+'.log',p/(name+'.log'))
rows=[]
for v in ['close','gameplay']:
 vals=[]
 for phase in ['before','after']:
  s=(p/'evidence'/f'{phase}-{v}.metrics.txt').read_text();vals.append(re.search(r'mean_ms=(\S+).*p95_ms=(\S+)',s).groups())
 rows.append(f'| {v} | {vals[0][0]} / {vals[0][1]} | {vals[1][0]} / {vals[1][1]} |')
text='''# Arena tyre and timber — ready for review

7 November 2026. Starting commit a9b001bab68d2ffaa1608390ebe8613063a2351d. No commit, push or publication for this change.

## Implemented

One Blender-built prop group beside the eastern arena gum: two worn tyres at (13,-0.3), footprint1.1×1.1m and height0.72m, plus a three-plank timber barrier at (14,1.2), footprint2×0.28m and height0.8m. Existing timber, rubber and metal materials reused. Tyres are static low cover, not throwable items. Shared collision appended to PROP_SOLIDS; protocol25 rejects older layouts. Update host and guests together when publishing.

All 713 original Blender prop objects preserved by geometry/material/transform signatures; all68,612 original runtime triangles preserved within float32 export tolerance. Outside scenery and other models unchanged. Carrying routes, spawns, road and scoring circle checks pass. No scoring, controls or movement changes.

## Verification

446 core +246 app tests passed;1 ignored. Native release and browser web builds passed with existing warnings. Actual Backyard smoke and Smoko carry/delivery motion check passed (+100, no player left carried). git diff --check passed.

Matched native cameras,1280×800,300warmup frames,600wall-clock samples. Mean/p95 milliseconds:

| View | Before | After |
|---|---|---|
'''+ '\n'.join(rows)+'''

Browser stationary results are in browser-metrics.json with actual screenshots. Same size and zero bots, but random spawn views differ; mouse-capture overlay remains visible. These are rendering/load checks, not a full multiplayer or browser movement playtest. Native same-camera measurements provide the controlled comparison. No claim of identical speed on every device.

Runtime props68,612→71,504triangles (+2,892),4,750,752→4,889,840bytes (+139,088). Same14material batches,13images; no new materials/textures. Local edit only, not an exact reproduction of the concept. Gameplay screenshots and75-frame motion capture in review.html.

## Changed files

- art/bush-servo/build_tyre_timber.py
- art/bush-servo/arena-tyre-timber.blend
- art/bush-servo/tyre-timber-bounds.json
- rust/crates/bbq_app/assets/models/smoko_props.glb
- rust/crates/bbq_core/src/bush_servo.rs
- rust/crates/bbq_core/src/net.rs
- checkpoints/LATEST.md and this directory

Run build_tyre_timber.py after refine_entrance.py. It loads arena-crates.blend, preserves existing props, saves a separate author scene, exports and applies the existing fresh-byte depth finish. No paid resources; Blender library integrations report off, so existing materials reused.

## Recovery

Verified pre-edit archive: /Users/marcus/Claude/australian-bbq-before-tyre-timber-20261107.tar.gz. All2083file hashes verified; starting-state.json lists them. Extract to a NEW empty directory, never over ongoing work. For selective rollback compare the three changed existing source/runtime files above with the extraction before restoring; preserve subsequent edits. Restore both collision code and GLB together, then rebuild. New Blender/script files may remain unused. Unrelated concept art preserved.

Public browser remains protocol24 and unchanged. Private staged protocol25 build used localhost:3018. Review before committing, pushing or publishing.

Local test:
```sh
cd /Users/marcus/Claude/australian-bbq/rust
~/.cargo/bin/cargo run -p bbq_app --release -- --menu
```
Solo game → Bush Servo → Play Smoko. Find the group beside the eastern gum.
'''
(p/'CHECKPOINT.md').write_text(text)
html='''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>Arena tyre and timber</title><style>body{background:#202a27;color:#f4ebd8;font:18px system-ui;max-width:1300px;margin:30px auto;padding:20px}h1,h2{color:#ffd06a}img{width:100%;border-radius:12px}section{margin:32px 0}.pair{display:grid;grid-template-columns:1fr 1fr;gap:18px}figure{margin:0}figcaption{padding:8px}a{color:#9ddad9}@media(max-width:800px){.pair{grid-template-columns:1fr}}</style><h1>Arena tyre and timber group</h1><p>Actual implemented game ·7 November2026· Ready for review, not published.</p><p>Two worn tyres and a low timber barrier beside the eastern gum. Existing materials reused, shared collision added, roads and carrying lanes clear.</p>'''
for v,label in [('close','Eastern gum close view'),('gameplay','Gameplay view')]:
 html+=f'<section><h2>{label}</h2><div class="pair"><figure><img src="evidence/before-{v}.png"><figcaption>Before</figcaption></figure><figure><img src="evidence/after-{v}.png"><figcaption>After — identical camera</figcaption></figure></div></section>'
html+='<section><h2>Moving view and delivery</h2><img src="evidence/moving.webp"></section><h2>Verification</h2><p>692 tests passed; native and browser builds passed. Original props preserved. +2,892 triangles, +139 KB; no new textures or material batches.</p><pre>'+ '\n'.join(rows)+'</pre><p>Columns: view, before mean/p95 ms, after mean/p95 ms. See checkpoint for methods and limits.</p><div class="pair"><figure><img src="evidence/browser-before.png"><figcaption>Browser before</figcaption></figure><figure><img src="evidence/browser-after.png"><figcaption>Browser after. Different spawn camera; stationary smoke check.</figcaption></figure></div><p><a href="CHECKPOINT.md">Checkpoint and recovery instructions</a></p></html>'
(p/'review.html').write_text(html)
files=['art/bush-servo/build_tyre_timber.py','art/bush-servo/arena-tyre-timber.blend','art/bush-servo/tyre-timber-bounds.json','rust/crates/bbq_app/assets/models/smoko_props.glb','rust/crates/bbq_core/src/bush_servo.rs','rust/crates/bbq_core/src/net.rs']
(p/'final-hashes.json').write_text(json.dumps({f:hashlib.sha256((r/f).read_bytes()).hexdigest() for f in files},indent=2))
latest=r/'checkpoints/LATEST.md';latest.write_text('# Arena tyre and timber — ready for review (7 November 2026)\n\n[Actual comparisons and motion](2026-11-07-tyre-timber/review.html) · [Checkpoint and recovery](2026-11-07-tyre-timber/CHECKPOINT.md). One tyre stack and timber barrier beside eastern arena gum, with shared collision. Tests and native/browser builds passed. Protocol25; public browser remains24. No commit, push or publication.\n\n'+latest.read_text())
