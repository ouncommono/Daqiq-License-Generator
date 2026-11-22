#!/usr/bin/env python3
"""Verification Script for Compiled Standalone Executable.

Executes compiled binary headlessly with:
1. --version check
2. --self-test check
Ensures zero missing DLLs, zero hidden-import crashes, and exit code 0.
"""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
from pathlib import Path


def verify(binary_path: Path) -> int:
    print(f"=== Verifying Compiled Binary: {binary_path} ===")

    if not binary_path.is_file():
        print(f"[ERROR] Target binary does not exist: {binary_path}", file=sys.stderr)
        return 1

    size_bytes = binary_path.stat().st_size
    print(f"Binary size: {size_bytes:,} bytes ({size_bytes / (1024 * 1024):.2f} MiB)")
    if size_bytes < 5 * 1024 * 1024:  # At least 5MB
        print(f"[ERROR] Binary size suspiciously small ({size_bytes} bytes)", file=sys.stderr)
        return 1

    env = os.environ.copy()
    env["QT_QPA_PLATFORM"] = "offscreen"

    # Step 1: --version check
    print("\n--- Test 1: --version check ---")
    proc_ver = subprocess.run(
        [str(binary_path), "--version"],
        capture_output=True,
        text=True,
        timeout=30,
        env=env,
    )
    print(f"STDOUT: {proc_ver.stdout.strip()}")
    print(f"STDERR: {proc_ver.stderr.strip()}")
    print(f"Exit code: {proc_ver.returncode}")

    if proc_ver.returncode != 0 or "1.0.0" not in proc_ver.stdout:
        print("[ERROR] --version check failed!", file=sys.stderr)
        return 1
    print("[PASS] --version check succeeded.")

    # Step 2: --self-test check
    print("\n--- Test 2: --self-test check ---")
    proc_test = subprocess.run(
        [str(binary_path), "--self-test"],
        capture_output=True,
        text=True,
        timeout=45,
        env=env,
    )
    print(f"STDOUT:\n{proc_test.stdout.strip()}")
    if proc_test.stderr.strip():
        print(f"STDERR:\n{proc_test.stderr.strip()}")
    print(f"Exit code: {proc_test.returncode}")

    if proc_test.returncode != 0 or "ALL SELF-TESTS PASSED SUCCESSFULLY" not in proc_test.stdout:
        print("[ERROR] --self-test check failed!", file=sys.stderr)
        return 1
    print("[PASS] --self-test check succeeded.")

    print("\n=== ALL COMPILED BINARY CHECKS PASSED SUCCESSFULLY ===")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="Verify compiled binary")
    parser.add_argument("binary", nargs="?", help="Path to executable")
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parent.parent
    if args.binary:
        binary_path = Path(args.binary).resolve()
    else:
        # Default to dist/Daqiq-License-Generator-v1.0.0.exe
        v_file = repo_root / "VERSION"
        ver = v_file.read_text(encoding="utf-8").strip() if v_file.is_file() else "1.0.0"
        binary_path = repo_root / "dist" / f"Daqiq-License-Generator-v{ver}.exe"

    return verify(binary_path)


if __name__ == "__main__":
    sys.exit(main())
