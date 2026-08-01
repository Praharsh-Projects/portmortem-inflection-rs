"""Test-only Python surface that forwards the upstream API to the Rust port."""

from __future__ import annotations

import atexit
import json
import re
import subprocess
import threading
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
BINARY = ROOT / "target" / "release" / "inflection-jsonl"

UNCOUNTABLES: set[str] = {
    "equipment",
    "fish",
    "information",
    "jeans",
    "money",
    "rice",
    "series",
    "sheep",
    "species",
}
_BUILTIN_UNCOUNTABLES = frozenset(UNCOUNTABLES)

_PROCESS: subprocess.Popen[str] | None = None
_LOCK = threading.Lock()


def _process() -> subprocess.Popen[str]:
    global _PROCESS
    if _PROCESS is None or _PROCESS.poll() is not None:
        if not BINARY.is_file():
            raise RuntimeError(f"Rust release binary is missing: {BINARY}")
        _PROCESS = subprocess.Popen(
            [str(BINARY)],
            cwd=ROOT,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            text=True,
            encoding="utf-8",
            bufsize=1,
        )
    return _PROCESS


@atexit.register
def _close_process() -> None:
    global _PROCESS
    process = _PROCESS
    _PROCESS = None
    if process is None or process.poll() is not None:
        return
    if process.stdin is not None:
        process.stdin.close()
    try:
        process.wait(timeout=2)
    except subprocess.TimeoutExpired:
        process.terminate()
        process.wait(timeout=2)


def _call(operation: str, value: str | int, **options: Any) -> str:
    request = {"operation": operation, "value": value, **options}
    with _LOCK:
        process = _process()
        if process.stdin is None or process.stdout is None:
            raise RuntimeError("Rust adapter pipes are unavailable")
        process.stdin.write(json.dumps(request, ensure_ascii=False, separators=(",", ":")) + "\n")
        process.stdin.flush()
        raw_response = process.stdout.readline()
    if not raw_response:
        raise RuntimeError("Rust adapter exited without a response")
    response = json.loads(raw_response)
    if response.get("ok") is True:
        return str(response["value"])

    error = response.get("error", {})
    message = str(error.get("message", "Rust adapter error"))
    if error.get("code") == "reference_index_error":
        raise IndexError(message)
    raise RuntimeError(message)


def camelize(string: str, uppercase_first_letter: bool = True) -> str:
    return _call(
        "camelize",
        string,
        uppercase_first_letter=uppercase_first_letter,
    )


def dasherize(word: str) -> str:
    return _call("dasherize", word)


def humanize(word: str) -> str:
    return _call("humanize", word)


def ordinal(number: int) -> str:
    return _call("ordinal", number)


def ordinalize(number: int) -> str:
    return _call("ordinalize", number)


def parameterize(string: str, separator: str = "-") -> str:
    return _call("parameterize", string, separator=separator)


def pluralize(word: str) -> str:
    if word.lower() in UNCOUNTABLES - _BUILTIN_UNCOUNTABLES:
        return word
    return _call("pluralize", word)


def singularize(word: str) -> str:
    for inflection in UNCOUNTABLES - _BUILTIN_UNCOUNTABLES:
        if re.search(r"(?i)\b(%s)\Z" % inflection, word):
            return word
    return _call("singularize", word)


def tableize(word: str) -> str:
    return _call("tableize", word)


def titleize(word: str) -> str:
    return _call("titleize", word)


def transliterate(string: str) -> str:
    return _call("transliterate", string)


def underscore(word: str) -> str:
    return _call("underscore", word)
