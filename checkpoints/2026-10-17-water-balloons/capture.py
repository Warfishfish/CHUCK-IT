import os,subprocess,sys
from pathlib import Path
folder=Path(__file__).resolve().parent
root=folder.parents[1]
phase=sys.argv[1]
if '"CGSSessionScreenIsLocked"=Yes' in subprocess.check_output(['ioreg','-n','Root','-d1'],text=True):
 raise SystemExit('Unlock the Mac before captures.')
binary='/tmp/bbq-water-balloons-before' if phase=='before' else str(root/'rust/target/release/bbq_app')
views={'held':['--mode','ffa','--bots','0','--give','gnome'], 'race-player':['--mode','race'], 'backyard':['--mode','heist','--bots','11','--cam','0,5,20,0,-10,78'], 'smoko':['--mode','smoko','--bots','5','--cam','0,7,23,0,-17,78'], 'grass-close':['--mode','heist','--bots','11','--cam','0,1.8,18,0,-13,78'], 'race':['--mode','race','--sideline','--cam','0,35,64,0,-27,78']}
for name,args in views.items():
 if len(sys.argv)>2 and name!=sys.argv[2]:continue
 out=folder/phase/(name+'.png');out.parent.mkdir(exist_ok=True)
 cmd=[binary,'--shot',str(out),'--review','--big','--look','polished','--style','pop','--capture-frame','900',*args]
 p=subprocess.run(cmd,cwd=root/'rust',env=dict(os.environ,BEVY_ASSET_ROOT=str(root/'rust/crates/bbq_app'),RUST_LOG='warn'),stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,timeout=240)
 out.with_suffix('.log').write_text(p.stdout)
 assert p.returncode==0 and 'ERROR' not in p.stdout,p.stdout[-2000:]
 assert out.exists() and out.with_suffix('.metrics.txt').exists()
 print(phase,name,'captured',flush=True)
