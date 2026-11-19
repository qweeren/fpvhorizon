#!/usr/bin/env python3
"""Build and package script for FPV Horizon Horizon mod.

Usage:
    python build.py             # Run tests, build .hmod, and install to Horizon launcher
    python build.py --test      # Run unit tests only
    python build.py --no-test   # Skip tests and package/install
    python build.py --dist-only # Package .hmod without installing
"""

import os
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent

RUST_BIN_DEFAULT = Path(r"C:\Users\baban\AppData\Roaming\horizon-fork-arismoko\launcher\rust\1.99.1-x86_64-pc-windows-gnu\bin")
MINGW_BIN_DEFAULT = Path(r"C:\Users\baban\AppData\Roaming\horizon-fork-arismoko\launcher\rust\1.99.1-x86_64-pc-windows-gnu\mingw64\bin")


def get_env_and_cargo():
    env = os.environ.copy()
    if RUST_BIN_DEFAULT.is_dir() and str(RUST_BIN_DEFAULT) not in env.get("PATH", ""):
        env["PATH"] = f"{RUST_BIN_DEFAULT};{MINGW_BIN_DEFAULT};{env.get('PATH', '')}"

    cargo_exe = shutil.which("cargo", path=env.get("PATH"))
    if not cargo_exe and (RUST_BIN_DEFAULT / "cargo.exe").is_file():
        cargo_exe = str(RUST_BIN_DEFAULT / "cargo.exe")
    return env, cargo_exe


def run_tests():
    print("==> Running unit tests (cargo test --lib)...")
    env, cargo_exe = get_env_and_cargo()
    if not cargo_exe:
        print("WARNING: cargo executable not found, skipping tests.", file=sys.stderr)
        return True

    res = subprocess.run([cargo_exe, "test", "--lib"], cwd=ROOT, env=env)
    if res.returncode != 0:
        print("ERROR: Unit tests failed!", file=sys.stderr)
        return False
    print("==> All unit tests passed!")
    return True


def package_and_install(install=True, game_dir=None):
    print("==> Packaging mod into .hmod...")
    package_script = ROOT / "tools" / "package.py"
    args = [sys.executable, str(package_script)]
    if install:
        args.append("--install")
        if game_dir:
            args.append(game_dir)
    res = subprocess.run(args, cwd=ROOT)
    if res.returncode != 0:
        print("ERROR: Packaging failed!", file=sys.stderr)
        return False
    return True


def main():
    args = sys.argv[1:]
    if "--test" in args:
        return 0 if run_tests() else 1

    if "--no-test" not in args:
        if not run_tests():
            return 1

    install = "--dist-only" not in args
    success = package_and_install(install=install)
    if success and install:
        print("\nSuccessfully built and installed fpvhorizon.hmod into Horizon Launcher!")
    return 0 if success else 1


if __name__ == "__main__":
    sys.exit(main())
