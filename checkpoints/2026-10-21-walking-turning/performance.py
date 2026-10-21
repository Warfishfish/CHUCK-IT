from pathlib import Path
import subprocess,os,sys
c=Path(__file__).resolve().parent;r=c.parents[1];phase=sys.argv[1]
assert '"CGSSessionScreenIsLocked"=Yes' not in subprocess.check_output(['ioreg','-n','Root','-d1'],text=True),'Unlock Mac for capture'
views={'gameplay':['--cam','0,3,18,0,-2,90'],'overview':['--cam','-10,9,18,0,-16,100'],'close':['--cam','-21,2.6,-7,35,-5,65'],'building':['--cam','-9,3,-17,0,8,75'],'character':['--pose','polish','--cam','-3,2.3,6,0,0,75'],'face-short':['--menu','--tab','look','--face-length','0'],'face-default':['--menu','--tab','look','--face-length','50'],'face-long':['--menu','--tab','look','--face-length','100']}
views={k:v for k,v in views.items() if k in ('gameplay',)}
for name,args in views.items():
 if len(sys.argv)>2 and name!=sys.argv[2]:continue
 if phase=='before' and name.startswith('face-'):continue
 out=c/('timing-'+phase)/(name+'.png');out.parent.mkdir(exist_ok=True)
 binary='/Users/marcus/Claude/australian-bbq-backup-2026-10-21-220805-walking-turning/rust/target/release/bbq_app' if phase=='before' else str(r/'rust/target/release/bbq_app')
 p=subprocess.run([binary,'--shot',str(out),'--review','--big','--look','polished','--style','pop','--mode','ffa','--bots','3','--capture-frame','900',*args],cwd=r/'rust',env=dict(os.environ,BEVY_ASSET_ROOT=str(r/'rust/crates/bbq_app'),RUST_LOG='warn'),capture_output=True,text=True,timeout=240)
 out.with_suffix('.log').write_text(p.stdout+p.stderr)
 assert p.returncode==0 and out.exists() and 'ERROR' not in p.stderr,p.stderr[-2000:]
 print(phase,name,flush=True)
