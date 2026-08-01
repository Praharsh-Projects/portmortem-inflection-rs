# Upstream findings

These findings were reproduced against the frozen Python source at commit
`88eefaacf7d0caaa701af7c8ab2d0ab3f17086f1`. They are candidate upstream bugs,
not claims that the upstream maintainers have confirmed them. Run:

```sh
python3 scripts/reproduce_findings.py
```

## 1. Empty lower-camel input raises `IndexError`

`camelize("", True)` returns `""`, while `camelize("", False)` raises
`IndexError: string index out of range`. The lower-camel branch indexes
`string[0]` before it handles the empty string.

The Rust library deliberately makes this transformation total and returns an
empty string. The JSONL evaluation adapter returns a structured
`reference_index_error` for the exact request so differential testing still
represents the pinned source behavior. This divergence is recorded in
`DECISIONS.md`.

Suggested upstream regression test:

```python
def test_camelize_lower_empty_string():
    assert inflection.camelize("", False) == ""
```

## 2. A multi-character separator can consume source underscores

For a separator of `"__sep__"`, these frozen-source outputs are observable:

```text
parameterize("a _b", "__sep__")       -> "a__sep__b"
parameterize("x/_y", "__sep__")       -> "x__sep__y"
parameterize("_trailing _", "__sep__") -> "_trailing"
```

The squeeze pattern is constructed as `escaped_separator + "{2,}"` without a
non-capturing group. For a multi-character separator, the quantifier therefore
applies only to its final regex atom. An adjacent underscore already present in
the input can be consumed as if it were part of a repeated separator.

The Rust port preserves this behavior for compatibility and locks it with a
unit test. A possible upstream fix would group the escaped separator before
applying the repetition quantifier, but that would be a behavior change and
should be evaluated by the maintainers.

## Reporting status

No upstream issue has been filed from this repository. Filing an issue is a
separate external action that should happen only after the project owner
reviews the wording and confirms that the behavior is unintended.
