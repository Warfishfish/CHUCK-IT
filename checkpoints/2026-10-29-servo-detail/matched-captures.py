"""Temporarily load archived baseline assets, always restore current assets, then capture after."""
from pathlib import Path
import os, subprocess, tarfile, hashlib
r=Path(__file__).resolve().parents[2]
cp=Path(__file__).parent
assets=["rust/crates/bbq_app/assets/models/smoko_servo.glb","rust/crates/bbq_app/assets/models/smoko_landscape.glb"]
current={name:(r/name).read_bytes() for name in assets}
env=dict(os.environ,CAPTURE_VIEWS="overview,servo,frontage")
try:
    with tarfile.open("/Users/marcus/Claude/australian-bbq-before-servo-detail-2026-10-29.tar.gz") as t:
        for name in assets:(r/name).write_bytes(t.extractfile(name).read())
    subprocess.run(["python3",str(cp/"capture.py"),"before"],env=env,check=True)
finally:
    for name,data in current.items():(r/name).write_bytes(data)
subprocess.run(["python3",str(cp/"capture.py"),"after"],env=env,check=True)
for name,data in current.items():assert hashlib.sha256((r/name).read_bytes()).digest()==hashlib.sha256(data).digest()
print("Matched captures complete; updated runtime assets restored")
