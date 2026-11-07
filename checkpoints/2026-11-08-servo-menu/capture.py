import os, subprocess
from pathlib import Path
root=Path(__file__).resolve().parents[2]
out=Path(__file__).parent/'evidence'
if '"CGSSessionScreenIsLocked"=Yes' in subprocess.check_output(['ioreg','-n','Root','-d1'],text=True):
    raise SystemExit('Mac locked')
env=dict(os.environ,BEVY_ASSET_ROOT=str(root/'rust/crates/bbq_app'))
for site,flag in [('northwest','--servo-nw'),('northeast','--servo-ne')]:
    for view,cam in [('approach','22,13,21,29,-18,72'),('overhead','0,60,0,0,-89.9,72')]:
        name=site+'-'+view
        args=[str(root/'rust/target/release/bbq_app'),'--mode','smoko','--shot',str(out/(name+'.png')),
              '--big','--bare','--look','polished','--style','pop','--review','--capture-frame','420','--cam',cam,flag]
        p=subprocess.run(args,cwd=root/'rust',env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,timeout=120)
        (out/(name+'.log')).write_text(p.stdout)
        assert p.returncode==0 and (out/(name+'.png')).exists(),p.stdout
        print(name,flush=True)
