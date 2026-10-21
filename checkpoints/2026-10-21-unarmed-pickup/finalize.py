from pathlib import Path
import hashlib,json,difflib,shutil
c=Path(__file__).resolve().parent;r=c.parents[1];base=json.loads((c/'baseline.json').read_text());b=Path(base['backup']);manifest={};patch=[]
for n,h in base['source_hashes'].items():
 after=hashlib.sha256((r/n).read_bytes()).hexdigest()
 if after==h:continue
 old=(b/n).read_bytes();assert hashlib.sha256(old).hexdigest()==h
 p=c/'before-source'/n;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(old)
 manifest[n]={'before':h,'after':after}
 patch.extend(difflib.unified_diff(old.decode().splitlines(True),(r/n).read_text().splitlines(True),fromfile='a/'+n,tofile='b/'+n))
(c/'changed-files.json').write_text(json.dumps(manifest,indent=2)+'\n');(c/'implementation.patch').write_text(''.join(patch))
s=(r/'checkpoints/2026-10-20-cartoon-backyard/revert.py').read_text().replace('pre-cartoon-update','pre-unarmed-pickup').replace('Earlier published features preserved.','Pending Cartoon Backyard work preserved.')
(c/'revert.py').write_text(s)
print('Checkpoint saved',len(manifest),'changed files; all other starting Rust sources preserved.')
