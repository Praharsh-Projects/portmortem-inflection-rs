# Demo script — maximum 5 minutes

Status: **recording plan; no hosted demo URL is committed.** A local candidate
may be rendered with `scripts/render_demo_video.sh`, but it must be reviewed and
hosted by the owner before submission. Do not record or render from a checkout
until `make verify` and `make bench` have completed there and their outputs have
been reviewed. Replace no bracketed cue with a claim that is not visible in the
current terminal or evidence file.

## Preflight (before recording)

1. Confirm the checkout and source hash shown in `.port-mortem.toml`.
2. Run `make verify` and preserve `fuzz/log.txt`.
3. Run `make bench` and review `bench/METHODOLOGY.md` plus
   `bench/results.json`.
4. Check `git status` so the video accurately describes committed versus local
   evidence.
5. Prepare a terminal wide enough to show complete JSON lines. Do not expose
   credentials, unrelated files, or account pages.

## 0:00–0:25 — What this is

**Show:** `README.md` title and `.port-mortem.toml`.

**Say:**

> This is `inflection-rs`, a Port Mortem Track D port from Python to Rust. The
> source is `jpvanhal/inflection`, pinned at commit
> `88eefaacf7d0caaa701af7c8ab2d0ab3f17086f1` under the MIT License. All port
> code was written after kickoff.

## 0:25–0:50 — Standalone architecture

**Show:** the top of `src/lib.rs`, the top of `src/main.rs`, and the repository
tree.

**Say:**

> The transformations live in a native Rust library. A thin JSON-lines binary
> gives the Python oracle and Rust port the same evaluation interface. The
> release artifact does not invoke or link Python, and both crate roots forbid
> unsafe Rust. Python remains only in the comparison tooling.

Only say the unsafe sentence if both visible crate roots still contain
`#![forbid(unsafe_code)]`.

## 0:50–1:20 — One-command build

**Run:**

```sh
make build
ls -lh target/release/inflection-jsonl
```

**Say:**

> The documented build is one command and produces this standalone release
> executable.

If the command fails, stop the recording; do not cut directly to a prebuilt
binary.

## 1:20–2:00 — Behavior through the evaluation adapter

**Run:**

```sh
printf '%s\n' \
  '{"operation":"pluralize","value":"person"}' \
  '{"operation":"parameterize","value":"Malmö & Rust"}' \
  '{"operation":"underscore","value":"HTMLTidyGenerator"}' \
  '{"operation":"ordinalize","value":1003}' \
  | target/release/inflection-jsonl
```

**Say:**

> Each input line receives one structured response. These examples exercise an
> irregular inflection, Unicode parameterization, acronym-aware underscore
> conversion, and ordinal formatting.

Read the displayed outputs; do not narrate expected values that differ from the
terminal.

## 2:00–2:35 — Original tests and parity

**Run:**

```sh
make test
make parity
```

**Say after both commands finish:**

> The Rust tests just completed [state the exact visible result]. The parity
> command verifies the frozen source hashes, runs the untouched 455-test suite
> first against the Python source and then through a bridge that delegates its
> transformations to the live Rust release binary, apart from preserving the
> suite's dynamically added uncountable value. It also runs the corresponding
> native Rust cases. Its current result is [read the exact visible summary]. The
> preserved test file and oracle are not edited to make a mismatch disappear.

Do not state a pass rate unless the command prints its numerator, denominator,
and failure count.

## 2:35–3:55 — Differential fuzz evidence

**Run:**

```sh
make fuzz
sed -n '1,120p' fuzz/log.txt
```

**Say after the run:**

> This runner compares both implementations on generated shared inputs. The log
> records the duration, corpus or case count, seed, and every divergence. In
> this run, [read the recorded facts exactly].

Do not claim the Differential Fuzz Survivor bonus unless the displayed log
proves at least 60 continuous seconds with zero divergences under the official
criterion.

## 3:55–4:30 — Benchmark evidence

**Show:** `bench/METHODOLOGY.md`, then `bench/results.json`.

**Say:**

> Benchmarks are separate from the correctness gate because they depend on the
> environment. This report compares the same workload, records its methodology
> and environment, and currently shows [read only the metrics and units in the
> reviewed file]. A regression is reported as a regression, not hidden.

Do not use the word “faster” unless the reviewed result and methodology support
it for the named workload.

## 4:30–4:55 — Engineering decisions and close

**Show:** headings in `DECISIONS.md`.

**Say:**

> The decision log explains the standalone boundary, ordered rule semantics,
> Unicode handling, immutable shared state, JSON error contract, and evidence
> limits. The submission is designed to be reproducible and candid about any
> remaining divergence. The repository records the evidence and exact revision
> for the entry.

Stop by 4:55 to leave five seconds of margin. A successful local run does not
by itself mean the repository is public, the video is uploaded, or the Devfolio
submission is complete.
