#!/usr/bin/env python3
"""Guarded rollback to the verified pre-update state, preserving the pending menu work."""
import argparse,hashlib,json,pathlib,tarfile,tempfile,os
cp=pathlib.Path(__file__).resolve().parent
root=cp.parent.parent
backup=root.parent/'australian-bbq-backup-2026-10-24-race-shirt.tar.gz'
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--restore',action='store_true',help='restore only the listed update files after all guards pass')
a=p.parse_args()
expected=json.loads((cp/'after-sha256.json').read_text())
changed=[name for name,digest in expected.items() if not (root/name).is_file() or hashlib.sha256((root/name).read_bytes()).hexdigest()!=digest]
if changed:raise SystemExit('Stopped: later edits detected; compare manually before recovery:\n'+'\n'.join(changed))
if not backup.is_file():raise SystemExit('Missing verified backup: '+str(backup))
print('Current files match the reviewed update. Backup:',backup)
if not a.restore:raise SystemExit('Check only; nothing changed. Use --restore to restore the pre-update versions.')
original={}
with tarfile.open(backup,'r|gz') as archive:
 for entry in archive:
  if entry.name in expected and entry.isfile():original[entry.name]=archive.extractfile(entry).read()
if set(original)!=set(expected):raise SystemExit('Stopped: incomplete backup; nothing changed')
for name,data in original.items():
 target=root/name
 with tempfile.NamedTemporaryFile(dir=target.parent,delete=False) as f:f.write(data);tmp=f.name
 os.replace(tmp,target)
print('Restored',len(original),'files. Pending main-menu work is preserved; rebuild the native game before testing.')
