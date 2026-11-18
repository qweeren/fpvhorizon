"""Packages the bundle as dist/fpvdrone.hnmod, the file the Horizon launcher installs.

An .hnmod is a zip holding horizon-native.json (the id, the version, the game,
the commits and the hook points' version) and the bundle's .rs and .toml files.

    python tools/package.py              write dist/fpvdrone.hnmod
    python tools/package.py --install    also copy it into the game folder's
                                         horizon-native-mods (default instance)
"""

import argparse
import json
import os
import re
import shutil
import sys
import tomllib
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
STAMP = (2026, 1, 1, 0, 0, 0)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "--install",
        nargs="?",
        const="",
        metavar="GAME_FOLDER",
        help="copy the .hnmod into this game folder's horizon-native-mods "
        "(default: %%APPDATA%\\horizon)",
    )
    args = parser.parse_args()

    bundles = [p for p in (ROOT / "mixins").iterdir() if (p / "horizon-mixin.toml").is_file()]
    if len(bundles) != 1:
        print(f"expected one bundle in mixins/, found {len(bundles)}", file=sys.stderr)
        return 1
    bundle = bundles[0]
    manifest = tomllib.loads((bundle / "horizon-mixin.toml").read_text(encoding="utf-8"))
    compat = json.loads((ROOT / "compatibility.json").read_text(encoding="utf-8"))
    mod_id, version = manifest["mod"]["id"], manifest["mod"]["version"]
    if bundle.name != mod_id:
        print(f"mixins/{bundle.name} should be named after the mod id, {mod_id}", file=sys.stderr)
        return 1
    pinned = manifest["compatibility"]["horizon_revision"]
    if pinned != compat["horizon_revision"]:
        print("horizon_revision differs between horizon-mixin.toml and compatibility.json", file=sys.stderr)
        return 1

    native = {
        "game": compat["game"],
        "hooks": compat["hooks"],
        "horizon": compat["horizon_revision"],
        "id": mod_id,
        "mixins": compat["horizon_mixins_revision"],
        "schema": 1,
        "version": version,
    }

    stage = ROOT / "dist" / mod_id
    if stage.exists():
        shutil.rmtree(stage)
    stage.mkdir(parents=True)

    # In our project, core/ is a folder with mod.rs and submodules.
    # We flatten core/ into core.rs
    core_dir = ROOT / "core"
    if (core_dir / "mod.rs").is_file():
        text = flatten(core_dir / "mod.rs")
        (stage / "core.rs").write_text(text, encoding="utf-8")

    # Copy files from bundle directory
    for path in sorted(bundle.glob("*")):
        if path.is_file() and path.suffix in (".rs", ".toml"):
            shutil.copyfile(path, stage / path.name)

    named = [row["source"] for row in manifest.get("module", [])]
    named += [row["helper"] for row in manifest.get("injection", [])]
    missing = [name for name in named if not (stage / name).is_file()]
    if missing:
        print(f"the manifest names files that are not there: {', '.join(missing)}", file=sys.stderr)
        return 1
    files = sorted(p for p in stage.rglob("*") if p.is_file())

    out = ROOT / "dist" / f"{mod_id}.hnmod"
    out.parent.mkdir(exist_ok=True)
    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as archive:
        def add(name: str, data: bytes) -> None:
            info = zipfile.ZipInfo(name, STAMP)
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o644 << 16
            archive.writestr(info, data)

        add("horizon-native.json", (json.dumps(native, indent=2, sort_keys=True) + "\n").encode())
        for path in files:
            add(f"mixins/{mod_id}/{path.relative_to(stage).as_posix()}", path.read_bytes())
    print(f"wrote {out} ({out.stat().st_size} bytes, {len(files)} files, {mod_id} {version})")

    if args.install is not None:
        game = Path(args.install) if args.install else Path(os.environ["APPDATA"]) / "horizon"
        if not game.is_dir():
            print(f"no game folder at {game}", file=sys.stderr)
            return 1
        target = game / "horizon-native-mods"
        target.mkdir(exist_ok=True)
        shutil.copyfile(out, target / out.name)
        print(f"installed {target / out.name}")
    return 0


MOD_LINE = re.compile(r"^(\s*)(pub(?:\([^)]*\))? )?mod (\w+);\s*$")


def flatten(path: Path) -> str:
    """The module file `path` with each `mod name;` replaced by the module's
    file inline (`mod name { ... }`), recursively."""
    folder = path.parent if path.name == "mod.rs" else path.parent / path.stem
    out = []
    for line in path.read_text(encoding="utf-8").split("\n"):
        m = MOD_LINE.match(line)
        if not m:
            out.append(line)
            continue
        indent, vis, name = m.group(1), m.group(2) or "", m.group(3)
        child = folder / f"{name}.rs"
        if not child.is_file():
            child = folder / name / "mod.rs"
        if not child.is_file():
            raise SystemExit(f"{path}: no file for `mod {name};`")
        body = flatten(child).rstrip("\n").split("\n")
        out.append(f"{indent}{vis}mod {name} {{")
        out.extend(f"{indent}    {l}" if l else "" for l in body)
        out.append(f"{indent}}}")
    return "\n".join(out)


if __name__ == "__main__":
    sys.exit(main())
