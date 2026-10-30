"""Alternate only the landscape asset, preserving/restoring the new version."""
from pathlib import Path
import tarfile,subprocess,os,hashlib
c=Path(__file__).resolve().parent;r=c.parents[1];p=r/'rust/crates/bbq_app/assets/models/smoko_landscape.glb'
current=p.read_bytes();digest=hashlib.sha256(current).hexdigest()
with tarfile.open(r.parent/'australian-bbq-before-arena-gums-2026-10-30.tar.gz') as t:original=t.extractfile(str(p.relative_to(r))).read()
try:
 for phase,data in [('before-repeat',original),('after-repeat',current)]:
  p.write_bytes(data)
  subprocess.run(['python3',str(c/'capture.py'),phase],env=dict(os.environ,CAPTURE_VIEWS='overview,tree'),cwd=r,check=True)
finally:
 p.write_bytes(current)
 assert hashlib.sha256(p.read_bytes()).hexdigest()==digest
