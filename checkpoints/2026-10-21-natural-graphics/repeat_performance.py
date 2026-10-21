from pathlib import Path
import subprocess,os,json
c=Path(__file__).resolve().parent;r=c.parents[1];b=Path(json.loads((c/'baseline.json').read_text())['backup'])
assert '"CGSSessionScreenIsLocked"=Yes' not in subprocess.check_output(['ioreg','-n','Root','-d1'],text=True),'Unlock Mac'
(c/'timing-repeats').mkdir(exist_ok=True)
for trial in range(1,3):
 for phase in (['before','after'] if trial==1 else ['after','before']):
  out=c/'timing-repeats'/f'{phase}-{trial}.png'
  binary=(b if phase=='before' else r)/'rust/target/release/bbq_app'
  p=subprocess.run([str(binary),'--shot',str(out),'--review','--big','--look','polished','--style','pop','--mode','ffa','--bots','3','--capture-frame','900','--cam','0,3,18,0,-2,90'],cwd=r/'rust',env=dict(os.environ,BEVY_ASSET_ROOT=str(r/'rust/crates/bbq_app'),RUST_LOG='warn'),capture_output=True,text=True,timeout=240)
  out.with_suffix('.log').write_text(p.stdout+p.stderr)
  assert p.returncode==0 and out.exists() and 'ERROR' not in p.stderr
  print(phase,trial,out.with_suffix('.metrics.txt').read_text().splitlines()[0],flush=True)
