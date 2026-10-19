"""Guarded rollback of only the Race completion and splash change; preserves prior pending work."""
from pathlib import Path
import hashlib,json,sys,shutil
folder=Path(__file__).resolve().parent
root=folder.parents[1]
manifest=json.loads((folder/'changed-files.json').read_text())
for name,entry in manifest.items():
 p=root/name
 digest=hashlib.sha256(p.read_bytes()).hexdigest() if p.exists() else None
 if digest not in (entry['before'],entry['after']):
  raise SystemExit(f'Stopped: {name} has later edits. Nothing changed; request a selective revert.')
if '--apply' not in sys.argv:
 print('Rollback check passed. Add --apply to restore the exact pre-Race-completion Rust source state.')
else:
 for name,entry in manifest.items():
  if entry['before'] is None: (root/name).unlink(missing_ok=True)
  else: shutil.copy2(folder/'before-source'/name,root/name)
 print('Restored pre-Race-completion sources. Published water balloons, Race foundation, grass and Smoko work preserved. Rebuild before playing. Checkpoint evidence is retained.')
