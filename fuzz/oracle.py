#!/usr/bin/env python3
"""JSON-lines oracle backed by the frozen upstream Python implementation."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any, Callable


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "reference"))

import inflection  # noqa: E402


STRING_OPERATIONS: dict[str, Callable[[str], str]] = {
    "dasherize": inflection.dasherize,
    "humanize": inflection.humanize,
    "pluralize": inflection.pluralize,
    "singularize": inflection.singularize,
    "tableize": inflection.tableize,
    "titleize": inflection.titleize,
    "transliterate": inflection.transliterate,
    "underscore": inflection.underscore,
}


def evaluate(request: dict[str, Any]) -> dict[str, Any]:
    """Evaluate one request using the frozen Python source."""

    operation = request.get("operation")
    value = request.get("value")

    if operation in STRING_OPERATIONS:
        if not isinstance(value, str):
            raise TypeError(f"{operation} expects a string value")
        result = STRING_OPERATIONS[operation](value)
    elif operation == "camelize":
        if not isinstance(value, str):
            raise TypeError("camelize expects a string value")
        uppercase = request.get("uppercase_first_letter", True)
        if not isinstance(uppercase, bool):
            raise TypeError("uppercase_first_letter must be a boolean")
        result = inflection.camelize(value, uppercase)
    elif operation == "parameterize":
        if not isinstance(value, str):
            raise TypeError("parameterize expects a string value")
        separator = request.get("separator", "-")
        if not isinstance(separator, str):
            raise TypeError("separator must be a string")
        result = inflection.parameterize(value, separator)
    elif operation in {"ordinal", "ordinalize"}:
        if isinstance(value, bool) or not isinstance(value, int):
            raise TypeError(f"{operation} expects an integer value")
        result = getattr(inflection, operation)(value)
    else:
        raise ValueError(f"unknown operation: {operation!r}")

    return {"ok": True, "value": result}


def main() -> int:
    for line_number, raw_line in enumerate(sys.stdin, start=1):
        if not raw_line.strip():
            continue
        try:
            request = json.loads(raw_line)
            if not isinstance(request, dict):
                raise TypeError("request must be a JSON object")
            response = evaluate(request)
        except Exception as error:  # The Rust CLI returns errors as data too.
            if isinstance(error, IndexError):
                code = "reference_index_error"
            elif isinstance(error, TypeError):
                code = "invalid_value_type"
            elif isinstance(error, ValueError):
                code = "unknown_operation"
            else:
                code = "oracle_error"
            response = {
                "ok": False,
                "error": {"code": code, "message": str(error)},
            }
        print(json.dumps(response, ensure_ascii=False, separators=(",", ":")))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
