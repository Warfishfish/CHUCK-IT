"""Restore only this lighting update; refuse to overwrite later edits."""
from pathlib import Path
import hashlib,json,sys,shutil
folder=Path(__file__).resolve().parent
root=folder.parents[1]
manifest=json.loads((folder/'changed-files.json').read_text())
for name,entry in manifest.items():
 path=root/name
 digest=hashlib.sha256(path.read_bytes()).hexdigest()
 if digest not in (entry['before'],entry['after']):
  raise SystemExit(f'Stopped: {name} has later edits. Ask for a selective revert; nothing was changed.')
if '--apply' not in sys.argv:
 print('All files match the saved versions. Add --apply to restore the pre-update lighting/materials.')
else:
 for name,entry in manifest.items():
  shutil.copy2(folder/'before-source'/Path(name).name,root/name)
 print('Restored lighting/materials only. Race and Smoko gameplay work is preserved. Rebuild before playing.')
