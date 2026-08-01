# Benchmark methodology

The benchmark compares the frozen Python implementation at commit
`88eefaacf7d0caaa701af7c8ab2d0ab3f17086f1` with the Rust port through the
same newline-delimited JSON protocol.

`make bench` first rebuilds the locked release artifact, then generates a deterministic mixed workload,
runs both implementations, and refuses to measure them unless every output is
identical. It then measures:

- batch latency across 30 measured processes after three warm-up processes;
- minimum, median, p95, p99, and maximum wall-clock latency;
- median cases per second;
- one-request process startup latency across at least 30 processes; and
- peak resident memory reported by `/usr/bin/time` when that tool exposes a
supported RSS field.

The JSON evidence records SHA-256 hashes for the release binary and every
source, lock, oracle, generator, and benchmark input needed to identify the
measured implementation.

Both paths include process startup, JSON parsing, computation, and JSON output.
The Python path runs the frozen source directly; the Rust path uses a release
binary. Results describe one recorded machine and workload only. They are not a
claim about all hardware, Python versions, or input distributions.
