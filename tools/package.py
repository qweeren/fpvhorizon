"""Packages the bundle as dist/fpvdrone.hnmod, the file the Horizon launcher installs.

An .hnmod is a zip holding horizon-native.json (the id, the version, the game,
the commits and the hook points' version) and the bundle's .rs and .toml files.

    python tools/package.py              write dist/fpvdrone.hnmod
    python tools/package.py --install    also copy it into the game folder's
                                         horizon-native-mods (default instance)
"""

import argparse
import hashlib
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
        target_dirs = []
        if args.install:
            target_dirs.append(Path(args.install))
        else:
            default_game = Path(os.environ["APPDATA"]) / "horizon"
            if default_game.is_dir():
                target_dirs.append(default_game)
            fork_game = Path(os.environ["APPDATA"]) / "horizon-fork-arismoko" / "game"
            if fork_game.is_dir():
                target_dirs.append(fork_game)

        if not target_dirs:
            print("no game folder found to install into", file=sys.stderr)
            return 1

        for game in target_dirs:
            install_to_instance(game, out, mod_id, version, compat["horizon_revision"])
    return 0


def install_to_instance(game: Path, out: Path, mod_id: str, version: str, horizon_rev: str):
    target = game / "horizon-native-mods"
    target.mkdir(exist_ok=True)
    dst = target / out.name
    shutil.copyfile(out, dst)
    print(f"installed {dst}")

    # 1. SHA-256 digest
    with open(dst, "rb") as f:
        digest_raw = hashlib.sha256(f.read()).hexdigest()
    digest = f"SHA-256:{digest_raw}"

    # 2. Write origin.json for launcher native mod discovery
    origin_path = target / f"{mod_id}.origin.json"
    origin_data = {
        "id": mod_id,
        "version": version,
        "owner": "local",
        "repo": mod_id,
        "tag": f"v{version}",
        "commit": horizon_rev,
        "asset": out.name,
        "sha256": digest_raw,
    }
    origin_path.write_text(json.dumps(origin_data, indent=2) + "\n", encoding="utf-8")
    print(f"wrote {origin_path}")

    # 3. Register in mod-browser.json if present
    mb_path = game / "mod-browser.json"
    if mb_path.is_file():
        try:
            mb = json.loads(mb_path.read_text(encoding="utf-8"))
            entries = [e for e in mb.get("entries", []) if e.get("project") != mod_id]
            entries.append({
                "source": "horizon",
                "project": mod_id,
                "slug": mod_id,
                "title": "FPV Drone",
                "kind": "mod",
                "version": version,
                "number": version,
                "file": f"horizon-native-mods/{out.name}",
                "checksum": digest,
                "page": "https://github.com/fpvhorizon",
                "dependency": False,
                "verdict": "Verified",
            })
            mb["entries"] = entries
            mb_path.write_text(json.dumps(mb, indent=2) + "\n", encoding="utf-8")
            print(f"registered in {mb_path}")
        except Exception as e:
            print(f"warning: could not update mod-browser.json: {e}", file=sys.stderr)

    # 4. Enable in horizon-mods.txt
    mods_txt = game / "horizon-mods.txt"
    lines = []
    if mods_txt.is_file():
        lines = [l.strip() for l in mods_txt.read_text(encoding="utf-8").splitlines() if l.strip()]
    toggle_key = f"mod.{mod_id}:true"
    if toggle_key not in lines:
        lines = [l for l in lines if not l.startswith(f"mod.{mod_id}:")]
        lines.append(toggle_key)
        mods_txt.write_text("\n".join(lines) + "\n", encoding="utf-8")
        print(f"enabled in {mods_txt}")


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
