#!/usr/bin/env python3
"""Reproduce candidate bugs in the frozen upstream Python source."""

from __future__ import annotations

import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "reference"))

import inflection  # noqa: E402


def capture_camelize(value: str, uppercase_first_letter: bool) -> dict[str, object]:
    try:
        return {
            "input": value,
            "uppercase_first_letter": uppercase_first_letter,
            "ok": True,
            "value": inflection.camelize(value, uppercase_first_letter),
        }
    except Exception as error:
        return {
            "input": value,
            "uppercase_first_letter": uppercase_first_letter,
            "ok": False,
            "error": type(error).__name__,
            "message": str(error),
        }


def main() -> int:
    evidence = {
        "upstream": "https://github.com/jpvanhal/inflection",
        "commit": "88eefaacf7d0caaa701af7c8ab2d0ab3f17086f1",
        "python": sys.version.splitlines()[0],
        "camelize_empty": [
            capture_camelize("", True),
            capture_camelize("", False),
        ],
        "multi_character_separator": [
            {
                "input": value,
                "separator": "__sep__",
                "value": inflection.parameterize(value, "__sep__"),
            }
            for value in ("a _b", "x/_y", "_trailing _")
        ],
    }
    print(json.dumps(evidence, ensure_ascii=False, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
