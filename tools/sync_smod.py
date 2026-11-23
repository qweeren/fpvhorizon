"""Regenerates the Sunrise package (smod/) from the Horizon one (hmod/).

The mod is the same code on both games. Sunrise renamed the game's crates and the mod API's
attribute, and a package can't reach files outside its own folder, so smod/ carries its own
copy of src/, assets/ and LICENSE. Edit hmod/, then run this to carry the change over:

    python tools/sync_smod.py          rewrite smod/src, smod/assets and smod/LICENSE
    python tools/sync_smod.py --check  change nothing; exit 1 if smod/ is out of date

smod/sunrise-mod.toml and smod/Cargo.toml are not generated: they hold the Sunrise pin, and
the mixin targets and module file in the manifest name Sunrise's crate.
"""

import argparse
import shutil
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "hmod"
DST = ROOT / "smod"

# What Sunrise renamed in the code the mod is built from. The mod's own crate and ids
# (`mod_fpvhorizon`, `fpvhorizon:...`) stay.
RENAMES = [
    ("horizon_client::", "sunrise_client::"),
    ("mod_api::horizon_mod", "mod_api::sunrise_mod"),
]
COPIED = ["src", "assets", "LICENSE"]


def converted(path: Path) -> bytes:
    data = path.read_bytes()
    if path.suffix == ".rs":
        text = data.decode("utf-8")
        for old, new in RENAMES:
            text = text.replace(old, new)
        data = text.encode("utf-8")
    return data


def wanted() -> dict[str, bytes]:
    files: dict[str, bytes] = {}
    for name in COPIED:
        root = SRC / name
        paths = sorted(root.rglob("*")) if root.is_dir() else [root]
        for path in paths:
            if path.is_file():
                files[path.relative_to(SRC).as_posix()] = converted(path)
    return files


def present() -> set[str]:
    found: set[str] = set()
    for name in COPIED:
        root = DST / name
        paths = root.rglob("*") if root.is_dir() else [root]
        for path in paths:
            if path.is_file():
                found.add(path.relative_to(DST).as_posix())
    return found


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--check", action="store_true", help="only report whether smod/ is current")
    args = parser.parse_args()

    files = wanted()
    stale = sorted(
        [rel for rel, data in files.items() if not (DST / rel).is_file() or (DST / rel).read_bytes() != data]
        + [rel for rel in present() - files.keys()]
    )
    if args.check:
        for rel in stale:
            print(f"smod/{rel} is out of date", file=sys.stderr)
        return 1 if stale else 0

    for rel in present() - files.keys():
        (DST / rel).unlink()
    for rel, data in files.items():
        target = DST / rel
        target.parent.mkdir(parents=True, exist_ok=True)
        if not target.is_file() or target.read_bytes() != data:
            target.write_bytes(data)
    for directory in sorted((DST / "src").rglob("*"), reverse=True):
        if directory.is_dir() and not any(directory.iterdir()):
            shutil.rmtree(directory)
    print(f"smod/ is current ({len(files)} files, {len(stale)} changed)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
