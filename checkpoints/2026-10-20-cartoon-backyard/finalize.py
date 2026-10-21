from pathlib import Path
import subprocess,json,hashlib
c=Path(__file__).resolve().parent;r=c.parents[1];base=json.loads((c/'baseline.json').read_text())['commit'];manifest={}
names=subprocess.check_output(['git','diff','--name-only',base,'--','rust'],cwd=r,text=True).splitlines()
names+=subprocess.check_output(['git','ls-files','--others','--exclude-standard','--','rust'],cwd=r,text=True).splitlines()
for n in sorted(set(names)):
 old=subprocess.run(['git','show',base+':'+n],cwd=r,capture_output=True)
 before=None
 if old.returncode==0:
  p=c/'before-source'/n;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(old.stdout);before=hashlib.sha256(old.stdout).hexdigest()
 manifest[n]={'before':before,'after':hashlib.sha256((r/n).read_bytes()).hexdigest()}
(c/'changed-files.json').write_text(json.dumps(manifest,indent=2)+'\n')
patch=subprocess.check_output(['git','diff',base,'--','rust'],cwd=r)
for n,e in manifest.items():
 if e['before'] is None:patch+=subprocess.run(['git','diff','--no-index','--','/dev/null',n],cwd=r,capture_output=True).stdout
(c/'implementation.patch').write_bytes(patch)
print('Saved',len(manifest),'source files for guarded recovery.')
