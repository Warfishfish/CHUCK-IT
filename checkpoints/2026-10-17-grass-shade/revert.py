"""Restore the pre-grass visual checkpoint, preserving existing race/Smoko changes."""
from pathlib import Path
import hashlib,json,sys,shutil
folder=Path(__file__).resolve().parent
root=folder.parents[1]
manifest=json.loads((folder/'changed-files.json').read_text())
for name,entry in manifest.items():
 path=root/name
 digest=hashlib.sha256(path.read_bytes()).hexdigest() if path.exists() else None
 if digest not in (entry['before'],entry['after']):
  raise SystemExit(f'Stopped: {name} has later edits. Nothing changed; request a selective revert.')
if '--apply' not in sys.argv:
 print('Revert check passed. Add --apply to restore the first lighting pass.')
else:
 for name,entry in manifest.items():
  if entry['before'] is None:
   (root/name).unlink(missing_ok=True)
  else:
   shutil.copy2(folder/'before-source'/name,root/name)
 print('Restored the first lighting pass. All existing race and Smoko gameplay changes remain. Rebuild before playing.')
