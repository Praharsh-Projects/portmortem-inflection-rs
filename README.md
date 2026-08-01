# inflection-rs — Port Mortem 2026

`inflection-rs` is a Track D (Python → Rust) port of
[`jpvanhal/inflection`](https://github.com/jpvanhal/inflection), pinned to
commit [`88eefaacf7d0caaa701af7c8ab2d0ab3f17086f1`](https://github.com/jpvanhal/inflection/commit/88eefaacf7d0caaa701af7c8ab2d0ab3f17086f1).
The pinned source is MIT-licensed. The port is implemented as standalone,
safe Rust; the shipped library and executable do not invoke or link a Python
runtime.

This repository was started after the Port Mortem kickoff at
2026-07-31 18:00 UTC. The preserved source snapshot and upstream license live
under [`reference/`](reference/). Python is used only by development-time
comparison tools that run the original and the port side by side.

## Evidence status

Local evidence has been generated and reviewed, the repository is public, and
GitHub Actions reruns the complete correctness gate on `main`. The signed-in
Devfolio project is saved as a verified draft with its repository, branding,
cover image, builder, narrative, technology, and platform fields. It is not a
hackathon submission until the required demo is hosted and the owner publishes
the project.

Retained evidence:

| Evidence | Command | Output | Current claim |
| --- | --- | --- | --- |
| Rust checks and source parity | `make verify` | `docs/VERIFICATION.md` | 30 Rust tests passed; the untouched 455-test suite passed against the source and through the live-Rust bridge |
| Differential fuzzing | `make fuzz` | `fuzz/log.txt` | 2,005,000 cases over 60.108s; 0 divergences |
| Benchmark comparison | `make bench` | `bench/results.json` | Outputs matched; 3.37x observed median batch-time ratio on the recorded workload |
| Benchmark method | review `bench/METHODOLOGY.md` | methodology document | Reviewed with the local results |

Do not convert a successful command into a numerical claim without retaining
the exact output, source revision, toolchain, machine details, and methodology.
The full local record, raw metrics, hashes, and remaining gates are in
[`docs/VERIFICATION.md`](docs/VERIFICATION.md).

## Demo video

A reproducible macOS renderer captures the build, examples, native tests, and
untouched upstream parity run from a clean checkout, then creates a narrated
sub-four-minute MP4 with a revision-and-checksum manifest:

```sh
./scripts/render_demo_video.sh
```

See [`docs/DEMO_VIDEO.md`](docs/DEMO_VIDEO.md) for outputs, review checks, and
the separate human-controlled hosting and submission gates.

## Build and run

Requirements:

- Rust 1.91.1 with `cargo`, `rustfmt`, and `clippy`. `rustup` reads the pinned
  `rust-toolchain.toml` automatically.
- GNU Make (or a compatible `make`) for the documented commands.
- Python 3.14 for parity, fuzzing, and benchmark tooling only. The parity command
  uses `uv` when available; otherwise that Python environment must provide
  `pytest`.

The required one-command build is:

```sh
make build
```

It produces the standalone release executable at
`target/release/inflection-jsonl`.

The executable accepts one JSON object per input line and emits one JSON object
per output line:

```sh
printf '%s\n' \
  '{"operation":"pluralize","value":"person"}' \
  '{"operation":"underscore","value":"HTMLTidyGenerator"}' \
  '{"operation":"ordinalize","value":1003}' \
  | target/release/inflection-jsonl
```

Successful responses have the shape `{"ok":true,"value":...}`. Invalid
requests produce a structured `{"ok":false,"error":...}` response rather
than contaminating standard output with diagnostics.

## Supported operations

The Rust library and JSONL adapter cover the pinned Python module's twelve
public callables:

- String operations: `camelize`, `dasherize`, `humanize`, `parameterize`,
  `pluralize`, `singularize`, `tableize`, `titleize`, `transliterate`, and
  `underscore`.
- Integer operations: `ordinal` and `ordinalize`.
- `camelize` accepts the optional `uppercase_first_letter` boolean.
- `parameterize` accepts the optional `separator` string.

The adapter is an evaluation surface, not a Python compatibility layer. Rust
callers can use the `portmortem_inflection_rs` library directly.

## Verification commands

```sh
make test       # Rust unit and integration tests
make parity     # untouched suite against source and live Rust, plus derived cases
make unicode-sweep # 1.6M deterministic broad-Unicode comparisons
make fuzz       # >=60-second differential fuzz run; writes fuzz/log.txt
make bench      # shared-workload comparison; writes bench/results.json
make verify     # format, lint, tests, parity, Unicode sweep, and differential fuzzing
```

`make verify` deliberately excludes `make bench`: benchmark output is evidence
about a declared environment, not a deterministic correctness gate. Run and
review both commands before making a submission or recording the demo.
For a non-submission smoke check only, `FUZZ_SECONDS=0 make fuzz` overrides the
default duration; its log does not satisfy the event's 60-second bonus rule.

## Repository map

```text
src/                    safe Rust library and JSONL executable
rust-toolchain.toml     pinned Unicode-16 Rust toolchain
tests/                  Rust tests
reference/              pinned Python source, tests, README, and MIT license
scripts/parity.py       source-versus-port parity runner
scripts/unicode_sweep.py broad-Unicode parity sweep
compat/inflection.py    test-only bridge from the untouched suite to Rust
fuzz/differential.py    differential fuzz runner
fuzz/log.txt            generated fuzz evidence
bench/benchmark.py      shared-workload benchmark runner
bench/METHODOLOGY.md    benchmark protocol
bench/results.json      generated benchmark evidence
DECISIONS.md            architectural divergences and rationale
.port-mortem.toml       track and source provenance
docs/VERIFICATION.md    exact local evidence and remaining gates
docs/UPSTREAM_FINDINGS.md reproducible candidate upstream bugs
assets/devfolio/         generated project branding and media disclosure
```

## Scope and interpretation

The north-star behavior is the pinned Python implementation, including its
ordered pluralization and singularization rules. The port does not shell out to
Python, load Python extensions, or wrap the source implementation. The Python
snapshot remains in the repository solely so the evidence tools can compare
the implementations. `make parity` runs the original suite and the corresponding
Rust cases. The same untouched `reference/test_inflection.py` also runs through
the test-only `compat/inflection.py` bridge, which delegates transformations to
the live release binary and locally preserves the suite's dynamically added
`UNCOUNTABLES` value; native Rust tests separately cover all fixed built-ins.
`make fuzz` directly sends identical generated requests to the Python oracle
and Rust executable.

Known design-level differences and their rationale are recorded in
[`DECISIONS.md`](DECISIONS.md). Any behavioral divergence discovered by tests,
parity checks, fuzzing, or benchmarking must be recorded there before
submission; it must not be hidden by changing the source oracle.

Successful adapter values are compared exactly. For exceptions, differential
tools compare success/error status and the structured error code; CPython's
version-specific regex diagnostic wording and character positions are retained
for diagnosis but are not treated as transformation behavior.

Two reproducible source-level findings, including an empty-input `IndexError`,
are documented in [`docs/UPSTREAM_FINDINGS.md`](docs/UPSTREAM_FINDINGS.md).

## Competition references

- [Port Mortem event brief, rules, rubric, and timeline](https://coderesurrection.com/2026/)
- [Port Mortem terms and eligibility](https://coderesurrection.com/2026/terms/)
- [Official Devfolio event page](https://portmortem.devfolio.co/)

The event requires a public repository at submission time. Publication,
Devfolio form completion, demo upload, evidence review, and final submission are
separate human-controlled steps and are not implied by a successful local
build.

## License

The port is distributed under the MIT License. See [`LICENSE`](LICENSE). The
upstream notice is also preserved verbatim in
[`reference/UPSTREAM_LICENSE`](reference/UPSTREAM_LICENSE).
