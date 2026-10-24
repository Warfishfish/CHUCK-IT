"""Recover to a new directory, never overwrite the working project."""
from pathlib import Path
import tarfile,sys
archive=Path('/Users/marcus/Claude/australian-bbq-backup-2026-10-25-animation-graphics.tar.gz')
if len(sys.argv)!=2: raise SystemExit('Usage: python3 restore-backup.py NEW_EMPTY_DIRECTORY')
dest=Path(sys.argv[1]).expanduser().resolve()
if dest.exists(): raise SystemExit('Destination already exists; choose a new directory to preserve work.')
with tarfile.open(archive,'r:gz') as t:
    members=[m for m in t.getmembers() if not m.name.startswith('rust/.venv/')]
    for m in members:
        if m.name.startswith('/') or '..' in Path(m.name).parts:
            raise SystemExit('Unsafe archive entry; no extraction performed: '+m.name)
        tarfile.data_filter(m, str(dest))
    dest.mkdir(parents=True)
    t.extractall(dest,members=members,filter='data')
print('Recovered backup into',dest,'; current project untouched.')
