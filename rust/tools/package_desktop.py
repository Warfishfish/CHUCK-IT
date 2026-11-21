"""Make a portable Windows ZIP or a macOS app ZIP with the runtime assets and credits."""

import argparse
import plistlib
import shutil
import stat
import tempfile
import zipfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
APP_NAME = "Australian BBQ"


def package(binary, target, output):
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory() as temporary:
        folder = Path(temporary)
        if target == "macos":
            contents = folder / f"{APP_NAME}.app" / "Contents"
            install = contents / "MacOS"
            install.mkdir(parents=True)
            (contents / "Info.plist").write_bytes(plistlib.dumps({
                "CFBundleExecutable": "bbq_app",
                "CFBundleIdentifier": "au.australianbbq.game",
                "CFBundleName": APP_NAME,
                "CFBundlePackageType": "APPL",
                "NSHighResolutionCapable": True,
            }))
            executable = install / "bbq_app"
        else:
            install = folder / APP_NAME
            install.mkdir()
            executable = install / f"{APP_NAME}.exe"

        shutil.copyfile(binary, executable)
        executable.chmod(0o755)
        shutil.copytree(ROOT / "crates/bbq_app/assets", install / "assets")
        with zipfile.ZipFile(output, "w", zipfile.ZIP_DEFLATED) as archive:
            for path in sorted(folder.rglob("*")):
                if not path.is_file():
                    continue
                member = zipfile.ZipInfo.from_file(path, path.relative_to(folder).as_posix())
                member.compress_type = zipfile.ZIP_DEFLATED
                member.create_system = 3
                mode = 0o755 if path == executable else 0o644
                member.external_attr = (stat.S_IFREG | mode) << 16
                with path.open("rb") as source, archive.open(member, "w") as zipped:
                    shutil.copyfileobj(source, zipped)
    with zipfile.ZipFile(output) as archive:
        assert archive.testzip() is None
        prefix = install.relative_to(folder).as_posix()
        names = set(archive.namelist())
        assert f"{prefix}/{executable.name}" in names
        assert f"{prefix}/assets/CREDITS.txt" in names
    print(output)


if __name__ == "__main__":
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument("target", choices=["macos", "windows"])
    cli.add_argument("binary", type=Path)
    cli.add_argument("output", type=Path)
    args = cli.parse_args()
    package(args.binary, args.target, args.output)
