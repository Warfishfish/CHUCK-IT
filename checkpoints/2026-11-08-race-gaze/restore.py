#!/usr/bin/env python3
"""Check first; --restore selectively restores this milestone's tracked files."""
import argparse,hashlib,json,tarfile
from pathlib import Path
here=Path(__file__).resolve().parent
root=here.parent.parent
manifest=json.loads((here/'recovery-manifest.json').read_text())
args=argparse.ArgumentParser();args.add_argument('--restore',action='store_true');opts=args.parse_args()
changed=[]
for name,expected in manifest['after_sha256'].items():
 p=root/name
 if not p.is_file() or hashlib.sha256(p.read_bytes()).hexdigest()!=expected:changed.append(name)
if changed:raise SystemExit('Stopped: later edits must be reviewed before restoration: '+', '.join(changed))
if not opts.restore:
 print('All guarded files match the reviewed milestone. Nothing restored. Use --restore only to revert these tracked files.')
else:
 with tarfile.open(Path(manifest['backup'])/'source.tar') as archive:
  for name in manifest['after_sha256']:
   data=archive.extractfile(name).read();(root/name).write_bytes(data)
 print('Restored only the listed tracked files. Evidence, other files, assets, Git history and public browser release were preserved. Rebuild to run the restored source, or use the saved previous executable.')
