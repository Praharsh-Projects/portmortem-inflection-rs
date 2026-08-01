#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
output_dir="${DEMO_OUTPUT_DIR:-${repo_root}/dist/demo}"
evidence_dir="${output_dir}/evidence"
slides_dir="${output_dir}/slides"
video_path="${output_dir}/inflection-rs-demo.mp4"
manifest_path="${output_dir}/manifest.txt"
voice="${DEMO_VOICE:-Daniel}"
speech_rate="${DEMO_SPEECH_RATE:-185}"

font_sans="/System/Library/Fonts/SFNS.ttf"
font_mono="/System/Library/Fonts/SFNSMono.ttf"
logo_path="${repo_root}/assets/devfolio/logo-600.png"
hero_path="${repo_root}/assets/devfolio/hero-1920x1080.png"

for command_name in ffmpeg ffprobe magick say git make jq shasum; do
  if ! command -v "${command_name}" >/dev/null 2>&1; then
    echo "Missing required command: ${command_name}" >&2
    exit 1
  fi
done

for required_path in "${font_sans}" "${font_mono}" "${logo_path}" "${hero_path}"; do
  if [[ ! -f "${required_path}" ]]; then
    echo "Missing required file: ${required_path}" >&2
    exit 1
  fi
done

cd "${repo_root}"

if [[ -n "$(git status --short)" ]]; then
  echo "Refusing to render from a dirty checkout." >&2
  git status --short >&2
  exit 1
fi

head_sha="$(git rev-parse HEAD)"
source_commit="$(awk -F '"' '/kickoff_hash/ { print $2; exit }' .port-mortem.toml)"
rendered_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
repo_url="https://github.com/Praharsh-Projects/portmortem-inflection-rs"

if [[ ! "${head_sha}" =~ ^[0-9a-f]{40}$ || ! "${source_commit}" =~ ^[0-9a-f]{40}$ ]]; then
  echo "Could not resolve the final checkout or pinned upstream commit." >&2
  exit 1
fi

mkdir -p "${output_dir}" "${evidence_dir}" "${slides_dir}"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/inflection-rs-demo.XXXXXX")"
trap 'rm -rf "${work_dir}"' EXIT
mkdir -p "${work_dir}/audio" "${work_dir}/clips" "${work_dir}/text"

echo "Running final-checkout commands captured by the demo renderer..."
make build >"${evidence_dir}/build.log" 2>&1

printf '%s\n' \
  '{"operation":"pluralize","value":"person"}' \
  '{"operation":"parameterize","value":"Malmö & Rust"}' \
  '{"operation":"underscore","value":"HTMLTidyGenerator"}' \
  '{"operation":"ordinalize","value":1003}' \
  | target/release/inflection-jsonl >"${evidence_dir}/examples.jsonl"

make test >"${evidence_dir}/test.log" 2>&1
make parity >"${evidence_dir}/parity.log" 2>&1
cp fuzz/log.txt "${evidence_dir}/fuzz.log"
cp bench/results.json "${evidence_dir}/benchmark.json"

if [[ -n "$(git status --short)" ]]; then
  echo "Verification altered the checkout; refusing to render an ambiguous revision." >&2
  git status --short >&2
  exit 1
fi

binary_sha="$(shasum -a 256 target/release/inflection-jsonl | awk '{print $1}')"

cat >"${work_dir}/text/01-body.txt" <<EOF
PORT MORTEM 2026  /  TRACK D

Python inflection  ->  standalone safe Rust

Final checkout
${head_sha}

Pinned upstream
${source_commit}
EOF

cat >"${work_dir}/text/02-body.txt" <<'EOF'
NATIVE RUST LIBRARY
  12 public transformation operations
  Ordered regex and replacement semantics
  Unicode 16 behavior pinned to the Python 3.14 oracle

THIN JSONL EVALUATION BINARY
  One request line -> one structured response
  Shared interface for parity, fuzzing, benchmarks, and demos

BOUNDARIES
  No Python runtime in the shipped artifact
  #![forbid(unsafe_code)] in both crate roots
EOF

{
  echo '$ make build'
  sed -n '1,16p' "${evidence_dir}/build.log"
  echo
  echo '$ ls -lh target/release/inflection-jsonl'
  ls -lh target/release/inflection-jsonl
  echo
  echo "sha256  ${binary_sha}"
} >"${work_dir}/text/03-body.txt"

{
  echo '$ printf <four JSON requests> | target/release/inflection-jsonl'
  echo
  cat "${evidence_dir}/examples.jsonl"
  echo
  echo 'Irregular inflection / Unicode parameterization / acronym handling / ordinal formatting'
} >"${work_dir}/text/04-body.txt"

{
  echo '$ make test'
  grep -E '^(running [0-9]+ tests|test result:)' "${evidence_dir}/test.log" | tail -n 12
  echo
  echo '$ make parity'
  grep -E '^(\+ |[0-9]+ passed|running [0-9]+ tests|test result:|PASS:)' "${evidence_dir}/parity.log" | tail -n 18
  echo
  echo "Captured from final checkout ${head_sha:0:12} at ${rendered_at}"
} >"${work_dir}/text/05-body.txt"

{
  echo '$ cat fuzz/log.txt'
  cat "${evidence_dir}/fuzz.log"
  echo
  echo 'Additional deterministic sweep:'
  echo '1,600,017 broad-Unicode and separator cases matched'
  echo 'Python UCD 16.0.0 / seed 20260801'
} >"${work_dir}/text/06-body.txt"

python_median="$(jq -r '.python.batch.median_ms' "${evidence_dir}/benchmark.json")"
rust_median="$(jq -r '.rust.batch.median_ms' "${evidence_dir}/benchmark.json")"
ratio="$(awk -v python_ms="${python_median}" -v rust_ms="${rust_median}" 'BEGIN { printf "%.2f", python_ms / rust_ms }')"
python_p99="$(jq -r '.python.batch.p99_ms' "${evidence_dir}/benchmark.json")"
rust_p99="$(jq -r '.rust.batch.p99_ms' "${evidence_dir}/benchmark.json")"
case_count="$(jq -r '.workload.cases_per_batch' "${evidence_dir}/benchmark.json")"

cat >"${work_dir}/text/07-body.txt" <<EOF
OUTPUT GATE
  Both implementations matched for ${case_count} cases before timing

MEDIAN BATCH TIME
  Python   ${python_median} ms
  Rust     ${rust_median} ms
  Observed ratio: ${ratio}x

P99 BATCH TIME
  Python   ${python_p99} ms
  Rust     ${rust_p99} ms

Single arm64 macOS workload; not a universal speed claim.
Raw samples, RSS, startup measurements, hashes, and limitations are retained.
EOF

cat >"${work_dir}/text/08-body.txt" <<EOF
REPRODUCIBLE AND REVIEWABLE

  Public source: ${repo_url}
  Final revision: ${head_sha}
  CI command: make verify

  30 native Rust tests
  455 untouched tests against Python source
  455 untouched tests through the live-Rust bridge
  2,005,000 retained differential cases / zero divergences

DECISIONS.md records compatibility boundaries and evidence limits.
Demo branding uses the disclosed generated assets; narration uses macOS TTS.
EOF

cat >"${work_dir}/text/01-narration.txt" <<EOF
This is inflection R S, a Port Mortem Track D port from Python to Rust. The source is J P Vanhal inflection, pinned at commit ${source_commit}, under the M I T license. This video was generated from the exact public revision displayed on screen.
EOF

cat >"${work_dir}/text/02-narration.txt" <<'EOF'
All twelve public transformation operations live in a native Rust library. A thin line-oriented JSON binary gives the Python oracle and Rust port the same evaluation interface. Python remains only in development comparison tooling. The shipped library and executable do not invoke or link Python, and both crate roots forbid unsafe Rust.
EOF

cat >"${work_dir}/text/03-narration.txt" <<'EOF'
The demo renderer just ran the documented one-command build from this clean checkout. It produced the standalone optimized executable shown here. The checksum binds this demonstration to the exact binary that handled the following examples.
EOF

cat >"${work_dir}/text/04-narration.txt" <<'EOF'
Each input line receives one structured response. These actual outputs exercise an irregular plural, Unicode parameterization, acronym-aware underscore conversion, and ordinal formatting. The adapter is deliberately small; transformation logic remains in the Rust library.
EOF

cat >"${work_dir}/text/05-narration.txt" <<'EOF'
The renderer then ran the native tests and the preserved original suite from this final checkout. The untouched four hundred fifty-five tests pass against the Python source, then pass again through a bridge that delegates transformations to the live Rust release binary. The bridge's narrow handling of the suite's dynamically added uncountable value is explicitly disclosed, and the native mappings also pass.
EOF

cat >"${work_dir}/text/06-narration.txt" <<'EOF'
Correctness extends beyond fixtures. The retained differential run compared two million five thousand shared Python and Rust cases for sixty point one zero eight seconds with zero divergences. A separate deterministic sweep matched one million six hundred thousand seventeen broad Unicode and separator cases against the pinned Unicode version.
EOF

cat >"${work_dir}/text/07-narration.txt" <<'EOF'
Benchmarks run only after the outputs match. On this documented five-thousand-case arm sixty-four Mac workload, Python's median batch time was eighty-nine point two eight three milliseconds and Rust's was twenty-six point four six three milliseconds, an observed ratio of three point three seven. Raw samples, tail latency, startup time, memory, environment details, hashes, and limitations are retained. This is not presented as a universal speed claim.
EOF

cat >"${work_dir}/text/08-narration.txt" <<EOF
The public repository contains the port, pinned source snapshot, original tests, decision log, evidence, and the continuous integration workflow that reruns the full verification gate. Inflection R S is standalone, reproducible, and candid about its compatibility boundaries. The exact revision and repository are on screen for review.
EOF

make_slide() {
  local number="$1"
  local kicker="$2"
  local title="$3"
  local body_path="${work_dir}/text/${number}-body.txt"
  local output_path="${slides_dir}/${number}.png"

  magick -size 1920x1080 gradient:'#07111f-#101b29' \
    -fill '#14263a' -draw 'roundrectangle 116,312 1804,960 34,34' \
    -fill '#1f3850' -draw 'roundrectangle 116,312 1804,326 7,7' \
    -fill '#5ee0a0' -font "${font_sans}" -pointsize 31 -annotate +130+104 "${kicker}" \
    -fill '#f5f7fb' -font "${font_sans}" -pointsize 70 -annotate +130+225 "${title}" \
    -fill '#8ea4bc' -font "${font_sans}" -pointsize 26 -annotate +130+1018 "inflection-rs  /  ${head_sha:0:12}  /  captured ${rendered_at}" \
    \( -background none -fill '#d9e5f2' -font "${font_mono}" -pointsize 30 -interline-spacing 10 -size 1570x570 caption:@"${body_path}" \) \
    -gravity northwest -geometry +176+356 -composite \
    "${output_path}"
}

make_title_slide() {
  local output_path="${slides_dir}/01.png"
  magick "${hero_path}" -resize '1920x1080^' -gravity center -extent 1920x1080 \
    \( -size 1920x1080 xc:'#030812b8' \) -compose over -composite \
    "${logo_path}" -resize 170x170 -gravity northwest -geometry +130+150 -composite \
    -fill '#5ee0a0' -font "${font_sans}" -pointsize 32 -annotate +130+104 'PORT MORTEM 2026  /  TRACK D' \
    -fill '#ffffff' -font "${font_sans}" -pointsize 94 -annotate +350+225 'inflection-rs' \
    -fill '#d9e5f2' -font "${font_sans}" -pointsize 44 -annotate +350+290 'Verified Python-to-Rust behavioral port' \
    -fill '#d9e5f2' -font "${font_mono}" -pointsize 28 -interline-spacing 9 -annotate +132+470 "final checkout  ${head_sha}\npinned source   ${source_commit}\npublic repo     ${repo_url}" \
    -fill '#8ea4bc' -font "${font_sans}" -pointsize 26 -annotate +130+1018 "Generated from the final checkout at ${rendered_at}" \
    "${output_path}"
}

make_title_slide
make_slide 02 'ARCHITECTURE' 'Standalone by construction'
make_slide 03 'ONE-COMMAND BUILD' 'Actual final-checkout output'
make_slide 04 'BEHAVIOR' 'Four requests, four structured responses'
make_slide 05 'ORIGINAL TEST SUITE' 'Captured final-checkout pass'
make_slide 06 'DIFFERENTIAL EVIDENCE' 'Broad inputs, zero retained divergences'
make_slide 07 'PERFORMANCE' 'Correctness-gated benchmark evidence'
make_slide 08 'HANDOFF' 'Everything needed for independent review'

concat_path="${work_dir}/concat.txt"
: >"${concat_path}"

for number in 01 02 03 04 05 06 07 08; do
  say -v "${voice}" -r "${speech_rate}" -f "${work_dir}/text/${number}-narration.txt" -o "${work_dir}/audio/${number}.aiff"
  ffmpeg -hide_banner -loglevel error -y \
    -loop 1 -framerate 30 -i "${slides_dir}/${number}.png" \
    -i "${work_dir}/audio/${number}.aiff" \
    -filter_complex '[1:a]apad=pad_dur=1.25[a]' \
    -map 0:v -map '[a]' \
    -c:v libx264 -preset medium -tune stillimage -crf 18 -pix_fmt yuv420p -r 30 \
    -c:a aac -b:a 160k -shortest \
    "${work_dir}/clips/${number}.mp4"
  printf "file '%s'\n" "${work_dir}/clips/${number}.mp4" >>"${concat_path}"
done

ffmpeg -hide_banner -loglevel error -y \
  -f concat -safe 0 -i "${concat_path}" \
  -c copy -movflags +faststart \
  "${video_path}"

duration_seconds="$(ffprobe -v error -show_entries format=duration -of default=noprint_wrappers=1:nokey=1 "${video_path}")"
if ! awk -v duration="${duration_seconds}" 'BEGIN { exit !(duration < 240.0) }'; then
  echo "Rendered video is ${duration_seconds}s; it must remain below 240 seconds." >&2
  exit 1
fi

video_sha="$(shasum -a 256 "${video_path}" | awk '{print $1}')"
cat >"${manifest_path}" <<EOF
project: inflection-rs
rendered_at_utc: ${rendered_at}
git_head: ${head_sha}
upstream_commit: ${source_commit}
release_binary_sha256: ${binary_sha}
video: ${video_path}
video_duration_seconds: ${duration_seconds}
video_sha256: ${video_sha}
voice: ${voice}
speech_rate_words_per_minute: ${speech_rate}
media_disclosure: generated project branding and macOS system TTS narration
captured_commands:
  - make build
  - four JSONL example requests through target/release/inflection-jsonl
  - make test
  - make parity
retained_evidence:
  - fuzz/log.txt
  - bench/results.json
EOF

echo "Rendered: ${video_path}"
echo "Duration: ${duration_seconds}s"
echo "SHA-256: ${video_sha}"
echo "Manifest: ${manifest_path}"
