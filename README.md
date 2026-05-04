# AfterLivie

AfterLivie is a macOS-first native livestream archive replay composer. It turns
one local video plus prepared canonical JSON, CSV, or JSONL comment sources into
a synchronized replay export with a default sidebar comment layout.

## Development

The intended development environment is Nix:

```sh
nix develop
nix run .#doctor
just test
```

This checkout includes:

- a Rust workspace for core/import/render/media/storage/bridge/CLI code,
- a SwiftPM macOS document app under `apps/macos`,
- canonical JSON, CSV, JSONL, and multi-source fixtures under `fixtures/comments`,
- v1.0 quickstart and format docs under `docs/`.

On machines without Nix, Rust, Cargo, ffmpeg, and ffprobe on `PATH`, only the
SwiftPM app shell can be built directly with the system Xcode toolchain.
