#!/usr/bin/env python3
"""Verify the frozen source snapshot and both upstream-derived test suites."""

from __future__ import annotations

import hashlib
import os
import shutil
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "reference" / "MANIFEST.sha256"


def verify_manifest() -> None:
    failures: list[str] = []
    for raw_line in MANIFEST.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        expected, relative = line.split(maxsplit=1)
        target = ROOT / "reference" / relative
        if not target.is_file():
            failures.append(f"missing: reference/{relative}")
            continue
        actual = hashlib.sha256(target.read_bytes()).hexdigest()
        if actual != expected:
            failures.append(f"hash mismatch: reference/{relative}: {actual}")
    if failures:
        raise SystemExit("\n".join(failures))
    print("PASS: frozen upstream snapshot hashes match")


def run(command: list[str], *, env: dict[str, str] | None = None) -> None:
    print("+", " ".join(command), flush=True)
    subprocess.run(command, cwd=ROOT, env=env, check=True)


def pytest_command() -> list[str]:
    uv = shutil.which("uv")
    if uv:
        return [
            uv,
            "run",
            "--python",
            "3.14",
            "--with",
            "pytest==9.1.1",
            "pytest",
        ]
    return [sys.executable, "-m", "pytest"]


def run_upstream_python_tests() -> None:
    environment = os.environ.copy()
    environment["PYTHONPATH"] = str(ROOT / "reference")
    run([*pytest_command(), "-q", "reference/test_inflection.py"], env=environment)


def run_upstream_suite_against_rust() -> None:
    environment = os.environ.copy()
    environment["PYTHONPATH"] = str(ROOT / "compat")
    run(
        [
            *pytest_command(),
            "-q",
            "--import-mode=importlib",
            "reference/test_inflection.py",
        ],
        env=environment,
    )


def main() -> int:
    if not MANIFEST.is_file():
        raise SystemExit(f"missing manifest: {MANIFEST}")
    verify_manifest()
    run_upstream_python_tests()
    run_upstream_suite_against_rust()
    run(["cargo", "test", "--locked", "--test", "upstream_cases"])
    print("PASS: frozen suite passed against source and through live-Rust bridge; Rust mappings completed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
