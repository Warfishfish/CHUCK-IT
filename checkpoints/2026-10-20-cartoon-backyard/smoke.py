from pathlib import Path
import subprocess,os
c=Path(__file__).resolve().parent;r=c.parents[1]
assert '"CGSSessionScreenIsLocked"=Yes' not in subprocess.check_output(['ioreg','-n','Root','-d1'],text=True),'Unlock Mac'
for mode in ['teams','heist','smoko','race']:
 out=c/'modes'/(mode+'.png');out.parent.mkdir(exist_ok=True)
 p=subprocess.run([str(r/'rust/target/release/bbq_app'),'--shot',str(out),'--review','--big','--look','polished','--style','pop','--mode',mode,'--bots','3','--capture-frame','600'],cwd=r/'rust',env=dict(os.environ,BEVY_ASSET_ROOT=str(r/'rust/crates/bbq_app'),RUST_LOG='warn'),capture_output=True,text=True,timeout=240)
 out.with_suffix('.log').write_text(p.stdout+p.stderr)
 assert p.returncode==0 and out.exists() and 'ERROR' not in p.stderr
 print(mode,'captured',flush=True)
