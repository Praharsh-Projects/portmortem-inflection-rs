#!/usr/bin/env python3
"""Run deterministic differential testing against the frozen Python oracle."""

from __future__ import annotations

import argparse
import hashlib
import json
import random
import string
import subprocess
import sys
import time
import unicodedata
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Iterable


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_BINARY = ROOT / "target" / "release" / "inflection-jsonl"
ORACLE = ROOT / "fuzz" / "oracle.py"
SOURCE_SHA = "88eefaacf7d0caaa701af7c8ab2d0ab3f17086f1"
SOURCE_INPUTS = (
    "Cargo.lock",
    "Cargo.toml",
    "rust-toolchain.toml",
    "src/lib.rs",
    "src/main.rs",
    "fuzz/differential.py",
    "fuzz/oracle.py",
    "reference/MANIFEST.sha256",
    "reference/inflection/__init__.py",
)

STRING_OPERATIONS = (
    "camelize",
    "dasherize",
    "humanize",
    "parameterize",
    "pluralize",
    "singularize",
    "tableize",
    "titleize",
    "transliterate",
    "underscore",
)

EDGE_STRINGS = (
    "",
    "person",
    "people",
    "CamelOctopus",
    "HTMLTidyGenerator",
    "node_child",
    "funky jeans",
    "Donald E. Knuth",
    "Random text with *(bad)* characters",
    "__leading_and_trailing__",
    "Malmö",
    "Garçons",
    "Ærøskøbing",
    "Aßlar",
    "ana índia",
    "Japanese: 日本語",
    "emoji🙂word",
    "x-men: the last stand",
    "david's code",
    "status_code",
    "passerby",
    "line before\nline after",
    "next\u0085line",
    "line\u2028separator",
    "paragraph\u2029separator",
    "trailing_id\n",
    "İıſK ǳǄǅ ﬁ",
    "e\u0301 decomposed",
    "Ελληνικά Кириллица العربية",
    "null\0byte",
)

ALPHABET = (
    string.ascii_letters
    + string.digits
    + string.punctuation
    + " \t\n\r"
    + "äöüéèçñßøÆÍİıſKǳǄǅﬁ"
    + "αβγΣςЖя日本語العربية🙂"
    + "\u0301\u0308"
)


def random_text(rng: random.Random) -> str:
    if rng.random() < 0.16:
        return rng.choice(EDGE_STRINGS)
    length = rng.randint(0, 80)
    return "".join(rng.choice(ALPHABET) for _ in range(length))


def generate_cases(count: int, seed: int, start_index: int = 0) -> list[dict[str, Any]]:
    """Generate reproducible cases without relying on Python hash ordering."""

    rng = random.Random(seed + (start_index * 1_000_003))
    cases: list[dict[str, Any]] = []
    ordinal_edges = (-10_000, -113, -112, -111, -21, -1, 0, 1, 2, 3, 11, 12, 13, 21, 22, 23, 1001, 10_000)

    for _ in range(count):
        if rng.random() < 0.15:
            operation = rng.choice(("ordinal", "ordinalize"))
            value = rng.choice(ordinal_edges) if rng.random() < 0.45 else rng.randint(-2_000_000, 2_000_000)
            cases.append({"operation": operation, "value": value})
            continue

        operation = rng.choice(STRING_OPERATIONS)
        request: dict[str, Any] = {"operation": operation, "value": random_text(rng)}
        if operation == "camelize":
            request["uppercase_first_letter"] = bool(rng.getrandbits(1))
        elif operation == "parameterize":
            request["separator"] = rng.choice(
                ("-", "_", "", "__sep__", ".", "Ö", "İ", r"\n", r"\g<0>", r"\1", r"\q")
            )
        cases.append(request)

    return cases


def encode_cases(cases: Iterable[dict[str, Any]]) -> str:
    return "".join(json.dumps(case, ensure_ascii=False, separators=(",", ":")) + "\n" for case in cases)


def run_jsonl(command: list[str], payload: str) -> list[dict[str, Any]]:
    completed = subprocess.run(
        command,
        cwd=ROOT,
        input=payload,
        text=True,
        capture_output=True,
        check=False,
    )
    if completed.returncode != 0:
        raise RuntimeError(
            f"command failed ({completed.returncode}): {' '.join(command)}\n{completed.stderr[-4000:]}"
        )

    responses: list[dict[str, Any]] = []
    # JSONL is delimited only by LF. str.splitlines() also treats valid JSON
    # string content such as U+0085, U+2028, and U+2029 as separators.
    for line_number, line in enumerate(completed.stdout.split("\n"), start=1):
        if not line.strip():
            continue
        try:
            value = json.loads(line)
        except json.JSONDecodeError as error:
            raise RuntimeError(
                f"invalid JSON from {' '.join(command)} on line {line_number}: {line!r}"
            ) from error
        if not isinstance(value, dict):
            raise RuntimeError(f"non-object response on line {line_number}: {value!r}")
        responses.append(value)
    return responses


def compare_batch(
    cases: list[dict[str, Any]],
    binary: Path,
    max_divergences: int,
) -> list[dict[str, Any]]:
    payload = encode_cases(cases)
    expected = run_jsonl([sys.executable, str(ORACLE)], payload)
    actual = run_jsonl([str(binary)], payload)

    if len(expected) != len(cases) or len(actual) != len(cases):
        raise RuntimeError(
            f"response count mismatch: cases={len(cases)} python={len(expected)} rust={len(actual)}"
        )

    divergences: list[dict[str, Any]] = []
    for index, (case, python_value, rust_value) in enumerate(zip(cases, expected, actual, strict=True)):
        if not responses_equivalent(python_value, rust_value):
            divergences.append(
                {
                    "index": index,
                    "case": case,
                    "python": python_value,
                    "rust": rust_value,
                }
            )
            if len(divergences) >= max_divergences:
                break
    return divergences


def responses_equivalent(expected: dict[str, Any], actual: dict[str, Any]) -> bool:
    """Compare values exactly and exceptions by their stable classification.

    CPython's regex diagnostic text and byte positions are implementation
    details that can change between interpreter patches. The transformation
    result, success/error status, and structured error code remain normative.
    """

    if expected == actual:
        return True
    if expected.get("ok") is not False or actual.get("ok") is not False:
        return False
    return expected.get("error", {}).get("code") == actual.get("error", {}).get("code")


def ensure_binary(binary: Path) -> None:
    if binary.resolve() == DEFAULT_BINARY.resolve():
        subprocess.run(["cargo", "build", "--release", "--locked"], cwd=ROOT, check=True)
    if not binary.exists():
        raise FileNotFoundError(f"release binary was not created: {binary}")


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def source_inputs_sha256() -> str:
    digest = hashlib.sha256()
    for relative in SOURCE_INPUTS:
        digest.update(relative.encode("utf-8"))
        digest.update(b"\0")
        digest.update((ROOT / relative).read_bytes())
        digest.update(b"\0")
    return digest.hexdigest()


def tool_version(command: list[str]) -> str:
    completed = subprocess.run(command, text=True, capture_output=True, check=False)
    return (completed.stdout or completed.stderr).splitlines()[0].strip()


def write_log(
    path: Path,
    *,
    seed: int,
    duration: float,
    case_count: int,
    divergences: list[dict[str, Any]],
    binary: Path,
) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    status = "PASS" if not divergences else "FAIL"
    lines = [
        "Port Mortem differential fuzz session",
        f"status: {status}",
        f"generated_at_utc: {datetime.now(timezone.utc).isoformat()}",
        f"upstream: https://github.com/jpvanhal/inflection@{SOURCE_SHA}",
        f"python: {sys.version.splitlines()[0]}",
        f"python_ucd: {unicodedata.unidata_version}",
        f"rustc: {tool_version(['rustc', '--version'])}",
        f"binary_sha256: {sha256_file(binary)}",
        f"source_inputs_sha256: {source_inputs_sha256()}",
        f"seed: {seed}",
        f"duration_seconds: {duration:.3f}",
        f"cases_compared: {case_count}",
        f"divergences: {len(divergences)}",
    ]
    if divergences:
        lines.extend(("", "first_divergences:"))
        lines.extend(json.dumps(item, ensure_ascii=False, sort_keys=True) for item in divergences)
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cases", type=int, default=10_000, help="minimum cases to compare")
    parser.add_argument("--seconds", type=float, default=0.0, help="continue for at least this many seconds")
    parser.add_argument("--batch-size", type=int, default=5_000)
    parser.add_argument("--seed", type=int, default=20_260_801)
    parser.add_argument("--max-divergences", type=int, default=25)
    parser.add_argument("--binary", type=Path, default=DEFAULT_BINARY)
    parser.add_argument("--log", type=Path, default=ROOT / "fuzz" / "log.txt")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.cases < 1 or args.batch_size < 1 or args.seconds < 0:
        raise SystemExit("cases and batch-size must be positive; seconds cannot be negative")

    binary = args.binary if args.binary.is_absolute() else ROOT / args.binary
    ensure_binary(binary)

    started = time.monotonic()
    total = 0
    all_divergences: list[dict[str, Any]] = []
    batch_index = 0

    while total < args.cases or (time.monotonic() - started) < args.seconds:
        remaining = max(args.cases - total, 0)
        count = max(1, min(args.batch_size, remaining or args.batch_size))
        cases = generate_cases(count, args.seed, batch_index)
        divergences = compare_batch(cases, binary, args.max_divergences - len(all_divergences))
        all_divergences.extend(divergences)
        total += len(cases)
        batch_index += 1
        if all_divergences:
            break

    duration = time.monotonic() - started
    log_path = args.log if args.log.is_absolute() else ROOT / args.log
    write_log(
        log_path,
        seed=args.seed,
        duration=duration,
        case_count=total,
        divergences=all_divergences,
        binary=binary,
    )

    if all_divergences:
        print(f"FAIL: {len(all_divergences)} divergence(s) found across {total} cases")
        print(f"Details: {log_path}")
        return 1

    print(f"PASS: {total} Python/Rust cases matched in {duration:.3f}s")
    print(f"Evidence: {log_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
