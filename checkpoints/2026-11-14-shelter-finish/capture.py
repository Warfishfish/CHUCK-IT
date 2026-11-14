"""Matched, actual native Smoko captures for the shelter finish review."""
import os
import pathlib
import subprocess
import sys
import time

ROOT = pathlib.Path(__file__).resolve().parents[2]
OUT = pathlib.Path(__file__).parent / "evidence"
VIEWS = {
    "sign-close": "0,2,2,180,35,65",
    "roof-raking": "7,6,17,35,-8,70",
    "standing-distance": "0,1.7,23,0,18,75",
}


def unlocked():
    return '"CGSSessionScreenIsLocked"=Yes' not in subprocess.check_output(
        ["ioreg", "-n", "Root", "-d1"], text=True
    )


for phase in sys.argv[1:] or ["before", "after"]:
    env = dict(os.environ, RUST_LOG="warn,bbq_app::servo_finish=info")
    if phase == "before":
        env["BEVY_ASSET_ROOT"] = "/tmp/australian-bbq-shelter-before"
    else:
        env["BEVY_ASSET_ROOT"] = str(ROOT / "rust/crates/bbq_app")
    for name, camera in VIEWS.items():
        if os.environ.get("CAPTURE_VIEWS") and name not in os.environ["CAPTURE_VIEWS"].split(","):
            continue
        if not unlocked():
            raise SystemExit("Mac locked; no capture attempted")
        path = OUT / f"{phase}-{name}.png"
        cmd = [
            str(ROOT / "rust/target/release/bbq_app"),
            "--mode", "smoko", "--bots", "0", "--shot", str(path), "--review", "--big",
            "--look", "polished", "--style", "pop", "--cam", camera, "--capture-frame", "360",
        ]
        started = time.time()
        result = subprocess.run(cmd, cwd=ROOT / "rust", env=env, stdout=subprocess.PIPE,
                                stderr=subprocess.STDOUT, text=True, timeout=180)
        (OUT / f"{phase}-{name}.log").write_text(result.stdout)
        assert result.returncode == 0 and path.exists() and path.stat().st_mtime >= started
        print(phase, name, "captured", flush=True)
