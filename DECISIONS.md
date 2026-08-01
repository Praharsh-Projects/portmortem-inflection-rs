# Architectural decisions

This log records intentional choices in the Track D Python → Rust port of
`jpvanhal/inflection` at source commit
`88eefaacf7d0caaa701af7c8ab2d0ab3f17086f1`. It describes design and scope; it
does not assert a parity percentage, fuzz result, or performance result.

## 1. Pin one upstream revision

**Decision.** Treat the named commit as the behavioral and licensing reference,
and preserve its relevant source, tests, README, and license under `reference/`.

**Rationale.** A moving branch cannot be a reproducible oracle. Pinning also
makes source-test edits visible during review.

**Consequence.** Later upstream fixes are out of scope unless they are disclosed
as a separately documented change. Evidence must identify this same commit.

## 2. Port the public transformation surface, not Python module mechanics

**Decision.** Implement the twelve public callables exposed by the pinned
module: ten string transformations plus `ordinal` and `ordinalize`. The private
rule-construction helper `_irregular` is an implementation detail, not a public
operation.

**Rationale.** Users depend on transformation behavior. Reproducing Python's
module loader, object model, or private initialization helper would not improve
behavioral equivalence.

**Consequence.** Public mutable module constants are not presented as mutable
Rust globals. The fixed built-in rule set is covered through function behavior.

## 3. Use a native library with a thin JSONL evaluation adapter

**Decision.** Put transformation logic in the `portmortem_inflection_rs`
library and keep `inflection-jsonl` as a serialization and dispatch boundary.

**Rationale.** Rust consumers should not need a subprocess. The line-oriented
adapter gives parity, fuzz, benchmark, and demo tools a stable language-neutral
surface without coupling the core library to Python.

**Consequence.** Adapter failures are not silently treated as transformation
results; they use the structured error envelope described below.

## 4. Leave the source-language runtime behind

**Decision.** The Rust library and release executable neither invoke nor link
Python. Python is allowed only in development-time oracle tooling.

**Rationale.** A wrapper would violate the competition's standalone-port rule
and would not demonstrate a migration.

**Consequence.** `make build` requires Rust but not Python. `make parity`,
`make fuzz`, and `make bench` require Python because they intentionally execute
the preserved source as a comparison oracle.

## 5. Forbid unsafe Rust at the crate boundary

**Decision.** Both the library and binary crate roots use
`#![forbid(unsafe_code)]`.

**Rationale.** This domain does not require manual memory operations or foreign
interfaces. A compiler-enforced rule is stronger and easier to audit than an
informal statement that unsafe code is absent.

**Consequence.** A future implementation that needs `unsafe` must first remove
an explicit compiler gate and document that change here; it cannot arrive as an
unnoticed local block.

## 6. Preserve rule order as behavior

**Decision.** Plural and singular transformations retain the pinned source's
ordered first-match semantics, including irregular and uncountable handling.

**Rationale.** Inflection rules overlap. Reordering logically similar patterns
can change outputs even if every individual pattern remains present.

**Consequence.** Refactors may change representation but must not sort, dedupe,
or otherwise reorder rules without parity evidence and a documented reason.

## 7. Translate regex intent instead of embedding Python

**Decision.** Express source patterns with Rust-native regular expressions and
explicit Rust replacement logic where Python-specific regex behavior cannot be
used directly.

**Rationale.** Native regex execution keeps the port standalone and idiomatic.
Blindly copying a pattern is unsafe when regex engines differ in supported
features or replacement semantics.

**Consequence.** The source pattern and expected examples remain the oracle.
Any deliberate regex rewrite must be judged by observable outputs, not textual
similarity to Python.

## 8. Share immutable compiled transformation state

**Decision.** Rule data and any compiled regex state are initialized once and
shared immutably across calls.

**Rationale.** The source builds a fixed rule table during module import.
Repeatedly compiling rules on every call would add avoidable work, while mutable
global state would complicate thread safety.

**Consequence.** The Rust API does not reproduce Python callers mutating
`PLURALS`, `SINGULARS`, or `UNCOUNTABLES` at runtime. That extension mechanism is
outside this port's callable compatibility scope.

## 9. Reproduce transliteration with native Unicode processing

**Decision.** Implement the pinned source's compatibility decomposition and
ASCII-filtering intent with Rust Unicode facilities rather than a Python call or
locale-dependent system utility.

**Rationale.** `transliterate` feeds `parameterize`, so Unicode behavior affects
multiple public operations. A native, deterministic path is portable and keeps
the artifact standalone.

**Consequence.** The preserved Unicode examples and differential inputs are
required evidence. Uncovered script- or version-specific differences must be
reported rather than described as exact parity.

## 10. Use owned UTF-8 outputs and bounded JSON integers

**Decision.** String transformations accept borrowed UTF-8 text and return
owned Rust strings. The JSONL boundary accepts integer values representable by
its Rust integer type for ordinal operations.

**Rationale.** Returned strings are newly transformed values, so ownership is
clear and avoids lifetime coupling. JSON has no portable arbitrary-precision
integer contract, unlike Python's integer type.

**Consequence.** Integers outside the adapter's accepted range are rejected as
invalid requests rather than rounded. This is a type-boundary divergence from
Python's arbitrary-precision integers and must not be advertised as unlimited
numeric parity.

## 11. Make protocol errors explicit and keep stdout machine-readable

**Decision.** Read one request per line and return either
`{"ok":true,"value":...}` or a structured `{"ok":false,"error":...}` object.
Diagnostics stay off standard output.

**Rationale.** Differential tools need to distinguish a valid transformed value
from malformed input, an unknown operation, or a type error. One response per
line permits streaming and reproducible corpus replay.

**Consequence.** The adapter is deliberately stricter than calling Python
functions interactively. Error-envelope behavior belongs to the evaluation
interface, not the upstream function contract. Successful values are compared
exactly; exception comparisons require the same success/error status and stable
error code, while CPython patch-specific regex message wording and positions
remain diagnostic rather than normative.

## 12. Keep the oracle immutable during evaluation

**Decision.** Parity tooling verifies the frozen snapshot, runs its original
Python tests once against the source and again unchanged against the live Rust
release binary through a test-only Python bridge, and runs the corresponding
native Rust cases. Differential fuzzing sends identical generated requests to
the preserved Python source and Rust executable. Neither path patches source
tests or expected values to make the port pass.

**Rationale.** The competition scores behavioral equivalence and weighs edits to
the original suite. An unchanged oracle makes failures useful evidence.

**Consequence.** `compat/inflection.py` exists only to let the unmodified Python
test surface drive the Rust JSONL process; it is not part of the shipped crate
or executable. Transformations delegate to Rust, while the bridge locally
preserves the test's dynamically added `UNCOUNTABLES` value. Native Rust tests
cover all fixed built-ins. A mismatch is reported with its input and both
outputs. Known flakes or justified source-test edits, if any arise, must be
named here before submission.

## 13. Separate deterministic verification from benchmark evidence

**Decision.** `make verify` runs formatting, linting, Rust tests, parity, the
broad-Unicode sweep, and a differential fuzz session. `make bench` is a separate
command with a written methodology and machine-readable results.

**Rationale.** Correctness checks should be repeatable gates. Benchmark numbers
depend on hardware, operating system, toolchain, warm-up, sample size, and
background load.

**Consequence.** CI success is not a performance claim. Any reported benchmark
must cite `bench/METHODOLOGY.md`, `bench/results.json`, and the environment that
produced it.

## 14. Prefer honest partial evidence to silent exclusions

**Decision.** Parity, fuzz, and benchmark tools must retain failures,
divergences, unsupported cases, and regressions in their output.

**Rationale.** The event rubric explicitly rewards reproducible, candid evidence
and scores partial parity proportionally. Hiding a difficult input would make
the result less defensible.

**Consequence.** No numerical claim belongs in README, the demo, or a submission
form until the corresponding generated output has been reviewed. The currently
reviewed local results and their limitations are recorded in
`docs/VERIFICATION.md`; public CI and submission evidence remain pending.

## 15. Preserve upstream attribution and license obligations

**Decision.** Keep the upstream copyright and MIT notice in both the reference
snapshot and the repository license, and link the exact upstream revision.

**Rationale.** This is a derivative port of an open-source project; competition
participation does not replace the source license.

**Consequence.** Distributing the port must retain the notices. Project code can
remain MIT-licensed without claiming ownership of upstream work.

## 16. Keep submission actions outside build automation

**Decision.** Build and evidence commands do not publish the repository, upload
the video, edit Devfolio, or submit the entry.

**Rationale.** Those actions require account context, final evidence review,
rules acceptance, and human authorization.

**Consequence.** A green `make verify` is software evidence, not proof that the
competition entry is complete or submitted.

## 17. Make empty lower-camel input total in Rust

**Decision.** `camelize("", false)` returns an empty string in the Rust API.

**Rationale.** The pinned Python function indexes the first character and raises
`IndexError` for this one empty-input mode. A Rust string transformation that
returns `String` has no useful transformed value other than the empty string,
and deliberately panicking would make the safe library less robust.

**Consequence.** This is a disclosed library-level behavioral divergence. The
JSONL evaluation adapter returns the source-equivalent structured
`reference_index_error` for this exact request, so differential evidence does
not conceal the source behavior while ordinary Rust callers retain a total
function.

## 18. Pin Unicode behavior instead of following Rust stable

**Decision.** Build with Rust 1.91.1 and exact Unicode case, category, and
normalization dependencies whose tables are version 16.0, matching the Python
3.14 oracle used for the retained local evidence. A deterministic
broad-Unicode sweep is part of `make verify`.

**Rationale.** Rust 1.92 updated the standard library to Unicode 17 while
Python 3.14 still uses Unicode 16. That changes casing for newly assigned code
points and can silently change `camelize`, `underscore`, `tableize`, and other
composed operations even when this source tree is unchanged.

**Consequence.** `rust-toolchain.toml`, exact Unicode dependency versions, and
`Cargo.lock` are behavioral inputs. Updating them requires rerunning parity,
the Unicode sweep, fuzzing, and benchmarks and reviewing any new divergence.
