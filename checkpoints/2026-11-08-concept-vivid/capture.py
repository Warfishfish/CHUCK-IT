import os,subprocess,sys
from pathlib import Path
r=Path(__file__).resolve().parents[2]
out=Path(__file__).parent/'evidence'
if '"CGSSessionScreenIsLocked"=Yes' in subprocess.check_output(['ioreg','-n','Root','-d1'],text=True):
 raise SystemExit('Mac is locked')
env=dict(os.environ,BEVY_ASSET_ROOT=str(r/'rust/crates/bbq_app'))
binary=os.environ.get('CAPTURE_BIN',str(r/'rust/target/release/bbq_app'))
for phase in sys.argv[1:]:
 for view,cam in [('picnic','13,2.45,-7.8,141,-13,64'),('shelter','6.8,2.05,-0.4,144,-10,65')]:
  if os.environ.get('CAPTURE_VIEW') and view!=os.environ['CAPTURE_VIEW']:continue
  args=[binary,'--mode','smoko','--shot',str(out/(phase+'-'+view+'.png')),'--big','--look','polished','--style','pop','--review','--capture-frame','360','--cam',cam]
  if phase=="before":args.append("--servo-baseline")
  p=subprocess.run(args,cwd=r/'rust',env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,timeout=180)
  (out/(phase+'-'+view+'.log')).write_text(p.stdout)
  assert p.returncode==0 and (out/(phase+'-'+view+'.png')).is_file(),p.stdout
  print(phase,view,flush=True)
