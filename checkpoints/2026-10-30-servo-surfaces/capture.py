"""Comparable actual native captures; abort if the Mac is locked."""
import pathlib,subprocess,os,sys,time
r=pathlib.Path(__file__).resolve().parents[2];out=pathlib.Path(__file__).parent/'evidence'
if '"CGSSessionScreenIsLocked"=Yes' in subprocess.check_output(['ioreg','-n','Root','-d1'],text=True):raise SystemExit('Mac locked')
env=dict(os.environ,BEVY_ASSET_ROOT=str(r/'rust/crates/bbq_app'),RUST_LOG='warn,bbq_app::servo_finish=info')
for phase in sys.argv[1:] or ['before','after']:
 for name,cam in [('overview','24,19,32,35,-29,70'),('gameplay','13,2.4,23,38,-3,75'),('close','10,7,22,40,-18,55'),('servo','7,6,-13,38,-8,65'),('frontage','-8,3.4,-18,0,-2,72')]:
  if os.environ.get('CAPTURE_VIEWS') and name not in os.environ['CAPTURE_VIEWS'].split(','):continue
  if '"CGSSessionScreenIsLocked"=Yes' in subprocess.check_output(['ioreg','-n','Root','-d1'],text=True):raise SystemExit('Mac locked; no capture attempted')
  binary=str(r/'rust/target/release/bbq_app')
  args=[binary,'--mode','smoko','--shot',str(out/(phase+'-'+name+'.png')),'--review','--big','--look','polished','--style','pop','--cam',cam,'--capture-frame','360']
  started=time.time()
  p=subprocess.run(args,cwd=r/'rust',env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,timeout=180)
  (out/(phase+'-'+name+'.log')).write_text(p.stdout);print(phase,name,p.returncode,flush=True);assert p.returncode==0 and 'ERROR' not in p.stdout

  for suffix in ['.png','.metrics.txt']:
   path=out/(phase+'-'+name+suffix)
   assert path.exists() and path.stat().st_mtime>=started, 'Incomplete/stale capture: '+str(path)

  # An interrupted/sleeping display can return all-black PNGs despite exit code zero.
  validator="from PIL import Image,ImageStat;import sys;s=ImageStat.Stat(Image.open(sys.argv[1]).convert('RGB').resize((64,40)));assert max(s.mean)>1,'Invalid black capture'"
  subprocess.run(['/Users/marcus/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3','-c',validator,str(out/(phase+'-'+name+'.png'))],check=True)
