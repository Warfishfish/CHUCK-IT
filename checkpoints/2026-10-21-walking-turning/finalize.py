from pathlib import Path
import hashlib,json,difflib
c=Path(__file__).resolve().parent;r=c.parents[1];base=json.loads((c/'baseline.json').read_text());b=Path(base['backup']);manifest={};patch=[]
names=set(base['source_hashes'])|{str(p.relative_to(r)) for p in r.glob('rust/crates/*/src/**/*.rs')}
for n in sorted(names):
 before=base['source_hashes'].get(n);after=hashlib.sha256((r/n).read_bytes()).hexdigest() if (r/n).exists() else None
 if before==after:continue
 old=(b/n).read_bytes() if before else b''
 if before:
  assert hashlib.sha256(old).hexdigest()==before
  p=c/'before-source'/n;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(old)
 manifest[n]={'before':before,'after':after}
 patch.extend(difflib.unified_diff(old.decode().splitlines(True),(r/n).read_text().splitlines(True),fromfile='a/'+n,tofile='b/'+n))
(c/'changed-files.json').write_text(json.dumps(manifest,indent=2)+'\n');(c/'implementation.patch').write_text(''.join(patch))
print('Saved',len(manifest),'changed files against this pass’s starting state.')
