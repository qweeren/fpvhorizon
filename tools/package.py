"""Packages the mod as dist/fpvhorizon.hmod, the package the Horizon launcher installs.

A native .hmod is the mod's source with a flat layout (playhorizon.dev/docs/mod-manifest):
horizon-mod.toml, Cargo.toml and src/ at the root, no folder named after the mod. The launcher
compiles it on the player's computer against the game build [native] pins.

The package is hmod/ (manifest, crate, hooks, mixin and support module) with the game-agnostic
drone core/ added as src/drone/.

    python tools/package.py              write dist/fpvhorizon.hmod
    python tools/package.py --install    also copy it into the game folder's
                                         horizon-native-mods (default instance)
"""

import argparse
import os
import shutil
import sys
import tomllib
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MOD = ROOT / "hmod"
STAMP = (2026, 1, 1, 0, 0, 0)
# Build output and the SDK `cargo horizon-mod` fetches stay out of the package.
SKIP = {".horizon", "target", "Cargo.lock"}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "--install",
        nargs="?",
        const="",
        metavar="GAME_FOLDER",
        help="copy the .hmod into this game folder's horizon-native-mods "
        "(default: %%APPDATA%%\\horizon)",
    )
    args = parser.parse_args()

    manifest = tomllib.loads((MOD / "horizon-mod.toml").read_text(encoding="utf-8"))
    mod_id, version = manifest["id"], manifest["version"]

    named = [row["helper"] for row in manifest.get("native", {}).get("mixin", [])]
    named += [row["source"] for row in manifest.get("native", {}).get("module", [])]
    missing = [name for name in named if not (MOD / name).is_file()]
    if missing:
        print(f"horizon-mod.toml names files that are not there: {', '.join(missing)}", file=sys.stderr)
        return 1

    files: dict[str, bytes] = {}
    for path in sorted(MOD.rglob("*")):
        rel = path.relative_to(MOD)
        if path.is_file() and not (set(rel.parts) & SKIP):
            files[rel.as_posix()] = path.read_bytes()
    for path in sorted((ROOT / "core").rglob("*.rs")):
        files[f"src/drone/{path.relative_to(ROOT / 'core').as_posix()}"] = path.read_bytes()

    out = ROOT / "dist" / f"{mod_id}.hmod"
    out.parent.mkdir(exist_ok=True)
    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as archive:
        for name, data in files.items():
            info = zipfile.ZipInfo(name, STAMP)
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o644 << 16
            archive.writestr(info, data)
    print(f"wrote {out} ({out.stat().st_size} bytes, {len(files)} files, {mod_id} {version})")

    if args.install is not None:
        if args.install:
            targets = [Path(args.install)]
        else:
            targets = [
                g
                for g in (
                    Path(os.environ["APPDATA"]) / "horizon",
                    Path(os.environ["APPDATA"]) / "horizon-fork-arismoko" / "game",
                )
                if g.is_dir()
            ]
        if not targets:
            print("no game folder found to install into", file=sys.stderr)
            return 1
        for game in targets:
            install_to_instance(game, out, mod_id)
    return 0


def install_to_instance(game: Path, out: Path, mod_id: str) -> None:
    target = game / "horizon-native-mods"
    target.mkdir(exist_ok=True)
    dst = target / out.name
    shutil.copyfile(out, dst)
    print(f"installed {dst}")

    # A mod copied in by hand is off until horizon-mods.txt turns it on.
    mods_txt = game / "horizon-mods.txt"
    lines = []
    if mods_txt.is_file():
        lines = [l.strip() for l in mods_txt.read_text(encoding="utf-8").splitlines() if l.strip()]
    toggle = f"mod.{mod_id}:true"
    if toggle not in lines:
        lines = [l for l in lines if not l.startswith(f"mod.{mod_id}:")]
        lines.append(toggle)
        mods_txt.write_text("\n".join(lines) + "\n", encoding="utf-8")
        print(f"enabled in {mods_txt}")


if __name__ == "__main__":
    sys.exit(main())
