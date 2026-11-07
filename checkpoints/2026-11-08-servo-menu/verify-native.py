import os, subprocess, sys
from pathlib import Path
root=Path(__file__).resolve().parents[2]
out=Path(__file__).parent/'evidence'
if '"CGSSessionScreenIsLocked"=Yes' in subprocess.check_output(['ioreg','-n','Root','-d1'],text=True):
    raise SystemExit('Mac locked')
env=dict(os.environ,BEVY_ASSET_ROOT=str(root/'rust/crates/bbq_app'))
tests=[
 ('implemented','smoko',['--bare','--cam','22,13,21,29,-18,72']),
 ('forecourt','smoko',['--bare','--cam','-12,2.2,-9,0,-4,78']),
 ('delivery','smoko',['--motion-review','smoko','--capture-frame','480']),
 ('backyard','ffa',[]),
]
for name,mode,extra in tests:
 if len(sys.argv)>1 and name not in sys.argv[1:]: continue
 args=[str(root/'rust/target/release/bbq_app'),'--mode',mode,'--shot',str(out/(name+'.png')),
       '--big','--look','polished','--style','pop','--review','--capture-frame','420']+extra
 p=subprocess.run(args,cwd=root/'rust',env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,timeout=180)
 (out/(name+'.log')).write_text(p.stdout)
 assert p.returncode==0 and 'ERROR' not in p.stdout and ((out/(name+'.png')).exists() or any(out.glob(name+'-*.png'))),p.stdout
 if name=='delivery': assert 'final_score=100 carries=0' in p.stdout,p.stdout
 print(name,flush=True)
