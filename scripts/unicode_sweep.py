#!/usr/bin/env python3
"""Compare a deterministic broad-Unicode sample with the frozen oracle."""

from __future__ import annotations

import argparse
import json
import random
import sys
import unicodedata
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
from fuzz.differential import DEFAULT_BINARY, compare_batch


STRING_OPERATIONS = (
    "dasherize",
    "humanize",
    "pluralize",
    "singularize",
    "tableize",
    "titleize",
    "transliterate",
    "underscore",
)

MANDATORY_SCALARS = (
    0x0085,
    0x019B,
    0x0264,
    0x02B0,
    0x0345,
    0x200C,
    0x2028,
    0x2029,
    0x24B6,
    0x24D0,
    0x16EB5,
)


def scalar_points(count: int, seed: int) -> list[int]:
    rng = random.Random(seed)
    points = list(MANDATORY_SCALARS[:count])
    seen = set(points)
    while len(points) < count:
        point = rng.randrange(0x110000)
        if not 0xD800 <= point <= 0xDFFF and point not in seen:
            points.append(point)
            seen.add(point)
    return points


def cases_for(point: int) -> list[dict[str, object]]:
    value = chr(point)
    cases: list[dict[str, object]] = [
        {"operation": operation, "value": value} for operation in STRING_OPERATIONS
    ]
    cases.extend(
        (
            {"operation": "camelize", "value": value, "uppercase_first_letter": True},
            {"operation": "camelize", "value": value, "uppercase_first_letter": False},
            {"operation": "parameterize", "value": value, "separator": "-"},
            {"operation": "titleize", "value": f"{value}x"},
            {"operation": "titleize", "value": f"a{value}b"},
            {"operation": "titleize", "value": f"a'{value}"},
            {"operation": "titleize", "value": f"a {value}"},
            {"operation": "singularize", "value": f"{value}species"},
        )
    )
    return cases


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--scalars", type=int, default=100_000)
    parser.add_argument("--batch-scalars", type=int, default=500)
    parser.add_argument("--seed", type=int, default=20_260_801)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.scalars < 1 or args.batch_scalars < 1:
        raise SystemExit("scalars and batch-scalars must be positive")

    points = scalar_points(args.scalars, args.seed)
    checked = 0
    for offset in range(0, len(points), args.batch_scalars):
        cases = [
            case
            for point in points[offset : offset + args.batch_scalars]
            for case in cases_for(point)
        ]
        divergences = compare_batch(cases, DEFAULT_BINARY, 10)
        if divergences:
            print(json.dumps(divergences, ensure_ascii=True, indent=2))
            return 1
        checked += len(cases)

    special_cases = [
        {"operation": "camelize", "value": "", "uppercase_first_letter": False},
        {"operation": "parameterize", "value": "x y", "separator": "Ö"},
        {"operation": "parameterize", "value": "x y", "separator": "İ"},
        {"operation": "parameterize", "value": "a b", "separator": r"\n"},
        {"operation": "parameterize", "value": "a b", "separator": r"\g<0>"},
        {"operation": "parameterize", "value": "abc", "separator": r"\1"},
        {"operation": "parameterize", "value": "a b", "separator": r"\q"},
        {"operation": "parameterize", "value": "a b", "separator": r"\400"},
        {"operation": "parameterize", "value": "a b", "separator": r"\777"},
        {"operation": "parameterize", "value": "a b", "separator": r"\08"},
        {"operation": "parameterize", "value": "a b", "separator": r"\1234"},
        {"operation": "parameterize", "value": "a b", "separator": r"\g<1>"},
        {"operation": "parameterize", "value": "a b", "separator": r"\g<x>"},
        {"operation": "parameterize", "value": "a b", "separator": r"\g<"},
        {"operation": "parameterize", "value": "a b", "separator": "\\"},
        {"operation": "parameterize", "value": "a b", "separator": r"\&"},
        {"operation": "parameterize", "value": "abc", "separator": "a" * 2_000_000},
    ]
    divergences = compare_batch(special_cases, DEFAULT_BINARY, 10)
    if divergences:
        print(json.dumps(divergences, ensure_ascii=True, indent=2))
        return 1
    checked += len(special_cases)

    print(
        "PASS: "
        f"{checked} broad-Unicode cases matched "
        f"(Python UCD {unicodedata.unidata_version}, seed {args.seed})"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
