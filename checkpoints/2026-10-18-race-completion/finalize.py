from pathlib import Path
import subprocess,json,hashlib,re,shutil
c=Path(__file__).resolve().parent;r=c.parents[1]
base=(c/'starting-commit.txt').read_text().strip()
names=subprocess.check_output(['git','diff','--name-only',base,'--','rust'],cwd=r,text=True).splitlines()
manifest={}
for name in names:
 old=subprocess.check_output(['git','show',base+':'+name],cwd=r);new=(r/name).read_bytes()
 p=c/'before-source'/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(old)
 manifest[name]={'before':hashlib.sha256(old).hexdigest(),'after':hashlib.sha256(new).hexdigest()}
(c/'changed-files.json').write_text(json.dumps(manifest,indent=2)+'\n')
(c/'implementation.patch').write_bytes(subprocess.check_output(['git','diff',base,'--','rust'],cwd=r))
shutil.copy2(r/'checkpoints/2026-10-17-water-balloons/revert.py',c/'revert.py')
p=c/'revert.py';s=p.read_text().replace('water-balloon change','Race completion and splash change').replace('water-balloon','Race').replace('pre-balloon','pre-Race-completion').replace('Earlier grass, lighting, Race and Smoko work preserved.','Published water balloons, Race foundation, grass and Smoko work preserved.');p.write_text(s)
rows=[]
for scene in ['held','sideline']:
 vals=[]
 for phase in ['before','after']:
  vals.append(dict(re.findall(r'(\w+)=([\d.]+)',(c/phase/(scene+'.metrics.txt')).read_text())))
 a,b=vals;delta=(float(b['mean_ms'])/float(a['mean_ms'])-1)*100
 rows.append(f"<tr><td>{scene}</td><td>{a['mean_ms']} ms / {a['mean_fps']} FPS</td><td>{b['mean_ms']} ms / {b['mean_fps']} FPS</td><td>{delta:+.1f}%</td><td>{int(b['image_cpu_bytes'])/1e6:.1f} MB</td></tr>")
html='''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>CHUCK IT · Race and water splash</title><style>
:root{color-scheme:dark}body{margin:0;background:#11231e;color:#f6f6e9;font:16px/1.55 system-ui}main{max-width:1280px;margin:auto;padding:30px}h1{font-size:38px}h2{margin-top:40px}a{color:#9bdcfa}.grid{display:grid;grid-template-columns:1fr 1fr;gap:16px}figure{margin:0;background:#213c32;border-radius:12px;overflow:hidden}img{width:100%;display:block}figcaption{padding:12px}td,th{text-align:left;padding:10px;border-bottom:1px solid #45675a}code{background:#294339;padding:4px}.note{border-left:4px solid #ffd348;padding:14px;background:#213c32}@media(max-width:740px){.grid{grid-template-columns:1fr}main{padding:15px}}
</style><main><p>ACTUAL IMPLEMENTED GAME CAPTURES · 19 OCTOBER 2026 · AWAITING REVIEW</p><h1>Race & throw + bigger water splashes</h1><p>Three racers follow the oval while sideline players and bots throw at them. Racers and throwers have separate standings and winners. Hits pay +50, +75, +100… in launch order; a resolved miss resets the streak. Automatic supplies and score-based upgrades keep the throwing game going.</p><p><a href="CHECKPOINT.md">Rules, verification and recovery</a> · <a href="tests.log">Tests</a> · <a href="release-build.log">Release build</a> · <a href="recovery-check.txt">Rollback verification</a></p>
<h2>Choose your role</h2><figure><img src="after/menu.png"><figcaption>Maps / Game modes → Race → Racer or Sideline Thrower. Online guests choose before the host starts. Three racing seats; extra players throw.</figcaption></figure>
<h2>Both games in one arena</h2><div class="grid"><figure><img src="after/race-player.png"><figcaption>Racer: ordered gates, three laps, dodging and bounded slowdowns.</figcaption></figure><figure><img src="after/sideline-play.png"><figcaption>Thrower: score, streak, next hit value and next upgrade. Hold and release left click; Q swaps items after the teddy unlock.</figcaption></figure></div>
<figure><img src="race.webp"><figcaption>Actual race replay. Bots run and throw through the same rules; no points are fabricated for this recording.</figcaption></figure>
<h2>Separate results</h2><figure><img src="after/results.png"><figcaption>Full race completion and final throwing standings. Tied throwing leaders share the win.</figcaption></figure>
<h2>A more dramatic balloon burst</h2><div class="grid"><figure><img src="../2026-10-17-water-balloons/motion/splash.png"><figcaption>Before: the previous water-balloon splash.</figcaption></figure><figure><img src="motion/splash.png"><figcaption>After: a wider radial burst and a short water-mist puff.</figcaption></figure></div><figure><img src="water-balloon.webp"><figcaption>Actual balloon hit, wet droplets and normal movement returning. Standard modes retain 20% slowdown for four seconds.</figcaption></figure>
<h2>Online check</h2><div class="grid"><figure><img src="network/host.png"><figcaption>Host: authoritative racer progress, throws, scores and upgrades.</figcaption></figure><figure><img src="network/guest.png"><figcaption>Guest: joined as a thrower and sent an actual throw through the local relay.</figcaption></figure></div><p class="note">Protocol 11: all players need this build. Tested with two local game instances and the real relay, plus headless authority and snapshot tests. Separate machines, WAN latency and packet-loss conditions were not tested.</p>
<h2>Performance</h2><table><tr><th>Scene</th><th>Before</th><th>After</th><th>Frame time</th><th>Retained images</th></tr>ROWS</table><p>1280×800, same fixed scene/camera, 600 measured wall-clock intervals, no compiler running. Race now includes two additional thrower bots, active projectiles and effects. These measurements are not GPU timestamps. A hidden developer-text allocation issue discovered during verification was fixed; the sustained full-round capture checks retained memory too.</p>
<h2>Existing modes</h2><div class="grid"><figure><img src="after/backyard.png"><figcaption>Heist smoke check.</figcaption></figure><figure><img src="after/smoko.png"><figcaption>Smoko smoke check; standard Backyard also exercised in the balloon replay.</figcaption></figure></div>
<h2>Test</h2><pre>cd /Users/marcus/Claude/australian-bbq/rust
~/.cargo/bin/cargo run -p bbq_app --release -- --mode race --sideline</pre><p>Omit <code>--sideline</code> to race. No commit or push. Selective recovery instructions are in the checkpoint.</p></main></html>'''
(c/'review.html').write_text(html.replace('ROWS',''.join(rows)))
print('Saved source manifest, patch, recovery script and review page.')
