from pathlib import Path
import subprocess,json,hashlib,shutil
c=Path(__file__).resolve().parent;r=c.parents[1];base='6dbfa03'
manifest={}
for n in subprocess.check_output(['git','diff','--name-only',base,'--','rust'],cwd=r,text=True).splitlines():
 old=subprocess.check_output(['git','show',base+':'+n],cwd=r);new=(r/n).read_bytes()
 p=c/'before-source'/n;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(old)
 manifest[n]={'before':hashlib.sha256(old).hexdigest(),'after':hashlib.sha256(new).hexdigest()}
(c/'changed-files.json').write_text(json.dumps(manifest,indent=2)+'\n')
(c/'implementation.patch').write_bytes(subprocess.check_output(['git','diff',base,'--','rust'],cwd=r))
s=(r/'checkpoints/2026-10-19-animation-polish/revert.py').read_text().replace('animation polish','height and graphics update').replace('pre-animation','pre-height-update').replace('Existing Race completion and splash work preserved.','Published Race, splash and animation work preserved.')
(c/'revert.py').write_text(s)
print('Saved',len(manifest),'source files for guarded recovery.')
