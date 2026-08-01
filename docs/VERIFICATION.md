# Verification record

This record summarizes the local evidence generated on 2026-08-01. It is tied
to the working tree that contains this file; the final submitted Git revision
must be added after the repository is published.

## Frozen source

- Source: `https://github.com/jpvanhal/inflection`
- Commit: `88eefaacf7d0caaa701af7c8ab2d0ab3f17086f1`
- Five vendored reference-file hashes: verified by
  `reference/MANIFEST.sha256`
- Original Python test command: run by `python3 scripts/parity.py`
- Original Python result against the frozen source: 455 passed, 0 failed
- The same untouched test file run through `compat/inflection.py`, which
  delegates transformations to the live release Rust binary and locally handles
  its dynamically added `UNCOUNTABLES` value: 455 passed, 0 failed
- Rust integration mapping: all 725 expanded upstream assertions represented;
  722 direct crate assertions and 3 explicitly labeled caller-owned adapter
  assertions for Python's mutable `UNCOUNTABLES` extension point

## Rust verification

`make verify` exited successfully after:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets -- -D warnings`
- 30 Rust tests passed: 6 library, 2 JSONL adapter, and 22 upstream-case groups
- the untouched 455-test suite passed against the frozen source and through the
  live-Rust bridge; the Rust upstream-case mapping also completed successfully
- 1,600,017 deterministic broad-Unicode and separator cases matched under Python Unicode
  16.0 and the pinned Rust 1.91.1 toolchain

The crate roots contain `#![forbid(unsafe_code)]`; the compiler rejects unsafe
blocks in the port.

## Differential fuzz session

The retained `fuzz/log.txt` records:

- seed `20260801`;
- 60.108 seconds;
- 2,005,000 shared Python/Rust cases;
- zero divergences; and
- SHA-256 `2c8694c8989533d433288a41f24606ef7dbfbd36b39b66f6ee7d1fc8aa1d5ab8`
  for the retained log at the time of this record.

The generator includes upstream examples, random ASCII, punctuation, embedded
line separators and NULs, decomposed accents, scripts without case, Unicode
titlecase characters, Python's four documented non-ASCII `re.IGNORECASE`
additions, and valid plus invalid separator replacement templates.
Earlier failing runs exposed Unicode-table drift, Python word-boundary and
titlecase semantics, separator replacement-template behavior, and JSONL line
splitting. The fixes are now regression-tested. The retained log also records
the exact release-binary and combined source-input hashes.

## Benchmark

`bench/results.json` records an arm64 macOS run using Python 3.14.6 and the
pinned Rust 1.91.1. Outputs matched on all 5,000 workload cases before
measurement.

| Metric | Frozen Python | Rust port |
| --- | ---: | ---: |
| Batch median, 5,000 cases | 89.283 ms | 26.463 ms |
| Batch p99 | 92.390 ms | 28.244 ms |
| One-case process median | 26.832 ms | 6.125 ms |
| Maximum resident set size | 18,268,160 bytes | 7,618,560 bytes |

For this one recorded workload, the median batch-time ratio was 3.37x in the
Rust port's favor. This is not a universal speed claim. Both paths include
process startup, JSON parsing, computation, and output. See
`bench/METHODOLOGY.md` and the full raw samples in `bench/results.json`.

The benchmark file SHA-256 at the time of this record is
`c1830ea7dcd0be66a0489d0944f80944043410e4f855923332d5457c8c465f3b`.
The JSON also records the measured release-binary hash and each source, lock,
oracle, generator, and harness input hash.

## Remaining non-software gates

- Confirm registration, team state, and the submission form in the signed-in
  Devfolio dashboard.
- Publish the final GitHub repository and rerun CI at the submitted SHA.
- Record and upload the five-minute demo from the final checkout.
- Review all claims, accept the rules, and complete the final submission.
