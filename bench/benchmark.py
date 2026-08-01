#!/usr/bin/env python3
"""Benchmark the frozen Python implementation against the Rust port."""

from __future__ import annotations

import argparse
import hashlib
import json
import platform
import re
import statistics
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))

from fuzz.differential import (  # noqa: E402
    encode_cases,
    ensure_binary,
    generate_cases,
    run_jsonl,
)


SOURCE_SHA = "88eefaacf7d0caaa701af7c8ab2d0ab3f17086f1"
DEFAULT_BINARY = ROOT / "target" / "release" / "inflection-jsonl"
EVIDENCE_INPUTS = (
    "Cargo.lock",
    "Cargo.toml",
    "rust-toolchain.toml",
    "src/lib.rs",
    "src/main.rs",
    "fuzz/differential.py",
    "fuzz/oracle.py",
    "reference/MANIFEST.sha256",
    "reference/inflection/__init__.py",
    "bench/benchmark.py",
)


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def percentile(values: list[float], quantile: float) -> float:
    ordered = sorted(values)
    if not ordered:
        raise ValueError("cannot calculate a percentile of an empty list")
    index = (len(ordered) - 1) * quantile
    lower = int(index)
    upper = min(lower + 1, len(ordered) - 1)
    fraction = index - lower
    return ordered[lower] + ((ordered[upper] - ordered[lower]) * fraction)


def time_command(command: list[str], payload: str, iterations: int, warmups: int) -> list[float]:
    for _ in range(warmups):
        run_jsonl(command, payload)

    durations: list[float] = []
    for _ in range(iterations):
        started = time.perf_counter_ns()
        run_jsonl(command, payload)
        durations.append((time.perf_counter_ns() - started) / 1_000_000.0)
    return durations


def summarize(durations_ms: list[float], case_count: int) -> dict[str, Any]:
    median_ms = statistics.median(durations_ms)
    return {
        "iterations": len(durations_ms),
        "min_ms": min(durations_ms),
        "median_ms": median_ms,
        "p95_ms": percentile(durations_ms, 0.95),
        "p99_ms": percentile(durations_ms, 0.99),
        "max_ms": max(durations_ms),
        "median_cases_per_second": case_count / (median_ms / 1000.0),
        "raw_ms": durations_ms,
    }


def maximum_rss(command: list[str], payload: str) -> dict[str, Any]:
    time_binary = Path("/usr/bin/time")
    if not time_binary.exists():
        return {"available": False, "reason": "/usr/bin/time is unavailable"}

    if sys.platform == "darwin":
        timed_command = [str(time_binary), "-l", *command]
        pattern = re.compile(r"^\s*(\d+)\s+maximum resident set size\s*$", re.MULTILINE)
        unit = "bytes"
        multiplier = 1
    else:
        timed_command = [str(time_binary), "-v", *command]
        pattern = re.compile(r"Maximum resident set size \(kbytes\):\s*(\d+)")
        unit = "kibibytes"
        multiplier = 1024

    completed = subprocess.run(
        timed_command,
        cwd=ROOT,
        input=payload,
        text=True,
        capture_output=True,
        check=False,
    )
    if completed.returncode != 0:
        return {
            "available": False,
            "reason": f"time command failed with {completed.returncode}",
            "stderr_tail": completed.stderr[-1000:],
        }
    match = pattern.search(completed.stderr)
    if not match:
        return {"available": False, "reason": "RSS field was not recognized"}
    raw = int(match.group(1))
    return {
        "available": True,
        "raw_value": raw,
        "raw_unit": unit,
        "bytes": raw * multiplier,
    }


def tool_version(command: list[str]) -> str:
    completed = subprocess.run(command, text=True, capture_output=True, check=False)
    return (completed.stdout or completed.stderr).splitlines()[0].strip()


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cases", type=int, default=5_000)
    parser.add_argument("--iterations", type=int, default=30)
    parser.add_argument("--warmups", type=int, default=3)
    parser.add_argument("--seed", type=int, default=20_260_801)
    parser.add_argument("--binary", type=Path, default=DEFAULT_BINARY)
    parser.add_argument("--output", type=Path, default=ROOT / "bench" / "results.json")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.cases < 1 or args.iterations < 1 or args.warmups < 0:
        raise SystemExit("cases and iterations must be positive; warmups cannot be negative")

    binary = args.binary if args.binary.is_absolute() else ROOT / args.binary
    ensure_binary(binary)

    python_command = [sys.executable, str(ROOT / "fuzz" / "oracle.py")]
    rust_command = [str(binary)]
    cases = generate_cases(args.cases, args.seed)
    payload = encode_cases(cases)

    python_output = run_jsonl(python_command, payload)
    rust_output = run_jsonl(rust_command, payload)
    if python_output != rust_output:
        raise SystemExit("refusing to benchmark: Python and Rust outputs differ on the workload")

    python_batch = time_command(python_command, payload, args.iterations, args.warmups)
    rust_batch = time_command(rust_command, payload, args.iterations, args.warmups)

    startup_case = encode_cases([{"operation": "pluralize", "value": "person"}])
    startup_iterations = max(30, args.iterations)
    python_startup = time_command(python_command, startup_case, startup_iterations, args.warmups)
    rust_startup = time_command(rust_command, startup_case, startup_iterations, args.warmups)

    results = {
        "schema_version": 1,
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "source": {
            "url": "https://github.com/jpvanhal/inflection",
            "commit": SOURCE_SHA,
        },
        "evidence_inputs_sha256": {
            relative: sha256_file(ROOT / relative) for relative in EVIDENCE_INPUTS
        },
        "release_binary_sha256": sha256_file(binary),
        "environment": {
            "platform": platform.platform(),
            "machine": platform.machine(),
            "python": sys.version.splitlines()[0],
            "rustc": tool_version(["rustc", "--version"]),
            "cargo": tool_version(["cargo", "--version"]),
        },
        "workload": {
            "seed": args.seed,
            "cases_per_batch": args.cases,
            "batch_iterations": args.iterations,
            "warmup_iterations": args.warmups,
            "startup_iterations": startup_iterations,
            "outputs_equal_before_measurement": True,
        },
        "python": {
            "batch": summarize(python_batch, args.cases),
            "startup": summarize(python_startup, 1),
            "maximum_rss": maximum_rss(python_command, payload),
        },
        "rust": {
            "batch": summarize(rust_batch, args.cases),
            "startup": summarize(rust_startup, 1),
            "maximum_rss": maximum_rss(rust_command, payload),
        },
        "limitations": [
            "Measurements include JSON parsing and process I/O for both implementations.",
            "This is a single-machine result, not a universal performance claim.",
            "Python uses the interpreter installed at the recorded path; Rust uses a release build.",
        ],
    }

    output = args.output if args.output.is_absolute() else ROOT / args.output
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(results, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    python_median = results["python"]["batch"]["median_ms"]
    rust_median = results["rust"]["batch"]["median_ms"]
    ratio = python_median / rust_median
    print(f"Outputs matched for {args.cases} cases before timing")
    print(f"Python batch median: {python_median:.3f} ms")
    print(f"Rust batch median:   {rust_median:.3f} ms")
    print(f"Observed median ratio (Python/Rust): {ratio:.2f}x")
    print(f"Evidence: {output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
