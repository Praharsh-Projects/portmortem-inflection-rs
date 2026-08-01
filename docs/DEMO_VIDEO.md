# Demo video rendering

The Devfolio submission requires a judge-accessible demo-video link. The local
renderer produces a narrated, sub-four-minute MP4 from a clean checkout while
capturing the build, example requests, native tests, and untouched upstream
parity run from that exact revision.

## Render

On macOS with Homebrew `ffmpeg`, ImageMagick, and the built-in `say` command:

```sh
./scripts/render_demo_video.sh
```

The ignored `dist/demo/` directory contains:

- `inflection-rs-demo.mp4` — the upload-ready video;
- `manifest.txt` — revision, duration, and SHA-256 bindings;
- `evidence/` — complete console logs and retained evidence used by the video;
- `slides/` — the rendered source frames for visual review.

The script refuses to run from a dirty checkout, fails if a captured command
fails or changes tracked files, and rejects a final duration of 240 seconds or
longer. The generated narration uses the macOS `Daniel` text-to-speech voice by
default; set `DEMO_VOICE` or `DEMO_SPEECH_RATE` to choose another installed
voice or rate. The manifest discloses both the system-generated narration and
the generated branding assets.

## Human review and hosting

Before upload, watch the complete MP4 and compare the displayed revision,
command summaries, metrics, and hashes with `manifest.txt`. The automatic
narration must also be accepted as part of the public presentation.

Devfolio's project editor accepts YouTube, Vimeo, or Loom links. Uploading to a
personal account, choosing visibility, accepting platform or event terms, and
publishing the Devfolio project remain owner-controlled external actions.
