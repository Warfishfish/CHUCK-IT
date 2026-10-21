"""Guarded source rollback. Run without --apply first. Evidence and the full backup remain."""
from pathlib import Path
import hashlib,json,sys,shutil
folder=Path(__file__).resolve().parent;root=folder.parents[1]
manifest=json.loads((folder/'changed-files.json').read_text())
for name,entry in manifest.items():
 p=root/name;digest=hashlib.sha256(p.read_bytes()).hexdigest() if p.exists() else None
 if digest not in (entry['before'],entry['after']):raise SystemExit(f'Stopped: {name} has later edits. Nothing changed; request a selective revert.')
 if entry['before'] is not None and hashlib.sha256((folder/'before-source'/name).read_bytes()).hexdigest()!=entry['before']:raise SystemExit('Recovery source checksum failed. Nothing changed.')
if '--apply' not in sys.argv:print('Rollback check passed. Add --apply to restore pre-cartoon-update sources.')
else:
 for name,entry in manifest.items():
  if entry['before'] is None:(root/name).unlink(missing_ok=True)
  else:shutil.copy2(folder/'before-source'/name,root/name)
 print('Restored pre-cartoon-update sources. Earlier published features preserved. Rebuild before playing. Checkpoint evidence retained.')
