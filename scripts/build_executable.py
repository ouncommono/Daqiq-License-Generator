#!/usr/bin/env python3
"""Unified Build Driver for Daqiq License Generator.

Bundles Daqiq License Generator into a standalone Windows executable.
Supports PyInstaller (standard and reliable for PySide6 + cryptography).
Outputs to dist/Daqiq-License-Generator-v1.0.0.exe and generates dist/SHA256SUMS.txt.
"""

from __future__ import annotations

import argparse
import hashlib
import os
import shutil
import subprocess
import sys
from pathlib import Path


def get_version(repo_root: Path) -> str:
    v_file = repo_root / "VERSION"
    if v_file.is_file():
        return v_file.read_text(encoding="utf-8").strip()
    return "1.0.0"


def compute_sha256(file_path: Path) -> str:
    h = hashlib.sha256()
    with open(file_path, "rb") as f:
        while chunk := f.read(65536):
            h.update(chunk)
    return h.hexdigest().lower()


def build(repo_root: Path, tool: str = "pyinstaller") -> int:
    version = get_version(repo_root)
    dist_dir = repo_root / "dist"
    build_dir = repo_root / "build"
    dist_dir.mkdir(parents=True, exist_ok=True)

    exe_name = f"Daqiq-License-Generator-v{version}.exe"
    icon_ico = repo_root / "assets" / "icon.ico"
    icon_png = repo_root / "assets" / "icon.png"
    icon_svg = repo_root / "assets" / "icon.svg"

    print("==================================================")
    print("Daqiq License Generator - Standalone Build Engine")
    print(f"Target Version: {version}")
    print(f"Bundler Tool:   {tool}")
    print(f"Repository:     {repo_root}")
    print(f"Output Binary:  {exe_name}")
    print("==================================================")

    if tool == "pyinstaller":
        cmd = [
            sys.executable,
            "-m",
            "PyInstaller",
            "--noconfirm",
            "--clean",
            "--onefile",
            f"--name=Daqiq-License-Generator-v{version}",
            f"--icon={str(icon_ico)}",
            f"--add-data={str(icon_png)}{os.pathsep}assets",
            f"--add-data={str(icon_ico)}{os.pathsep}assets",
            f"--add-data={str(icon_svg)}{os.pathsep}assets",
            "--collect-all=PySide6",
            "--collect-all=cryptography",
            "--collect-all=jdatetime",
            "--hidden-import=packaging.version",
            "--hidden-import=packaging.specifiers",
            "--hidden-import=pyperclip",
            "--hidden-import=openpyxl",
            str(repo_root / "main.py"),
        ]

        print("Executing command:")
        print(" ".join(cmd))
        print("\n--- Packaging in progress ---")

        proc = subprocess.run(cmd, cwd=str(repo_root))
        if proc.returncode != 0:
            print(f"[ERROR] Packaging failed with exit code {proc.returncode}", file=sys.stderr)
            return proc.returncode

        target_exe = dist_dir / exe_name
        if not target_exe.is_file():
            print(f"[ERROR] Output executable not found at {target_exe}", file=sys.stderr)
            return 1

        exe_size = target_exe.stat().st_size
        print(f"\nSuccessfully generated {target_exe.name} ({exe_size:,} bytes)")

        # Compute SHA256SUMS.txt
        sums_file = dist_dir / "SHA256SUMS.txt"
        sha = compute_sha256(target_exe)
        sums_content = f"{sha}  {target_exe.name}\n"
        sums_file.write_text(sums_content, encoding="utf-8")
        print(f"Generated {sums_file}:\n{sums_content.strip()}")

        return 0

    else:
        print(f"[ERROR] Unknown bundler tool: {tool}", file=sys.stderr)
        return 1


def main() -> int:
    parser = argparse.ArgumentParser(description="Build Standalone Windows Executable")
    parser.add_argument("--tool", choices=["pyinstaller", "nuitka"], default="pyinstaller")
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parent.parent
    return build(repo_root, tool=args.tool)


if __name__ == "__main__":
    sys.exit(main())
