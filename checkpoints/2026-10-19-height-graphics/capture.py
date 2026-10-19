from pathlib import Path
import subprocess,os,sys
c=Path(__file__).resolve().parent;r=c.parents[1];phase=sys.argv[1]
assert '"CGSSessionScreenIsLocked"=Yes' not in subprocess.check_output(['ioreg','-n','Root','-d1'],text=True),'Unlock Mac for capture'
views={'boxes':['--cam','-21,2.6,-7,35,-5,65'],'hoist':['--cam','4,2.8,10,25,-8,70'],'menu':['--menu','--tab','look'],'short':['--menu','--tab','look','--leg-length','0'],'medium':['--menu','--tab','look','--leg-length','50'],'tall':['--menu','--tab','look','--leg-length','100']}
for name,args in views.items():
 if len(sys.argv)>2 and name!=sys.argv[2]:continue
 out=c/phase/(name+'.png');out.parent.mkdir(exist_ok=True)
 binary='/tmp/bbq-before-height' if phase=='before' else str(r/'rust/target/release/bbq_app')
 p=subprocess.run([binary,'--shot',str(out),'--review','--big','--look','polished','--style','pop','--mode','ffa','--bots','0','--capture-frame','900',*args],cwd=r/'rust',env=dict(os.environ,BEVY_ASSET_ROOT=str(r/'rust/crates/bbq_app'),RUST_LOG='warn'),capture_output=True,text=True,timeout=240)
 out.with_suffix('.log').write_text(p.stdout+p.stderr)
 assert p.returncode==0 and out.exists() and 'ERROR' not in p.stderr,p.stderr[-2000:]
 print(phase,name,flush=True)
