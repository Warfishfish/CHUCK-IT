"""Comparable actual native captures; abort if the Mac is locked."""
import pathlib,subprocess,os,sys
r=pathlib.Path(__file__).resolve().parents[2];out=pathlib.Path(__file__).parent/'evidence'
if '"CGSSessionScreenIsLocked"=Yes' in subprocess.check_output(['ioreg','-n','Root','-d1'],text=True):raise SystemExit('Mac locked')
env=dict(os.environ,BEVY_ASSET_ROOT=str(r/'rust/crates/bbq_app'),RUST_LOG='warn')
for phase in sys.argv[1:] or ['before','after']:
 for name,cam in [('overview','24,19,32,35,-29,70'),('gameplay','13,2.4,23,38,-3,75')]:
  binary='/tmp/bbq-before-bush-servo' if phase=='before' else str(r/'rust/target/release/bbq_app')
  args=[binary,'--mode','smoko','--shot',str(out/(phase+'-'+name+'.png')),'--review','--big','--look','polished','--style','pop','--cam',cam,'--capture-frame','360']
  p=subprocess.run(args,cwd=r/'rust',env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,timeout=180)
  (out/(phase+'-'+name+'.log')).write_text(p.stdout);print(phase,name,p.returncode,flush=True);assert p.returncode==0 and 'ERROR' not in p.stdout
