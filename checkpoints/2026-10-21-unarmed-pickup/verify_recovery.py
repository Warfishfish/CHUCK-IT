from pathlib import Path
import tempfile,shutil,subprocess,json,hashlib
c=Path(__file__).resolve().parent;r=c.parents[1];m=json.loads((c/'changed-files.json').read_text())
subprocess.run(['python3',str(c/'revert.py')],check=True)
with tempfile.TemporaryDirectory(prefix='bbq-rollback-check-') as tmp:
 t=Path(tmp);d=t/'checkpoints'/c.name;shutil.copytree(c/'before-source',d/'before-source')
 for n in ['revert.py','changed-files.json']:shutil.copy2(c/n,d/n)
 for n in m:(t/n).parent.mkdir(parents=True,exist_ok=True);shutil.copy2(r/n,t/n)
 one=next(iter(m));p=t/one;p.write_bytes(p.read_bytes()+b'\n// later edit\n')
 denied=subprocess.run(['python3',str(d/'revert.py'),'--apply'],capture_output=True,text=True);assert denied.returncode!=0
 for n,e in m.items():
  if n!=one:assert hashlib.sha256((t/n).read_bytes()).hexdigest()==e['after']
 shutil.copy2(r/one,p);subprocess.run(['python3',str(d/'revert.py'),'--apply'],check=True)
 for n,e in m.items():assert (hashlib.sha256((t/n).read_bytes()).hexdigest() if (t/n).exists() else None)==e['before']
(c/'rollback-verification.txt').write_text(f'Live dry-run passed. Disposable-copy rollback restored all {len(m)} sources exactly; new files removed. Later edit caused refusal before any mutation. No rollback applied to the working project.\n')
