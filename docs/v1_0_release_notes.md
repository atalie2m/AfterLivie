# v1.0 Release Notes Draft

AfterLivie v1.0 is scoped to a single-source replay composer:

- one local video,
- one canonical JSON comment source,
- one default sidebar layout,
- project package save/load,
- preview segment and full export,
- structured import, media, renderer, export, project, and storage diagnostics.

Current implementation status:

- The Rust model, importer, renderer harness, ffprobe normalization, storage,
  render command, C ABI bridge, CLI, and SwiftPM shell are in place.
- The initial renderer is a deterministic Rust CPU path with system-font
  rasterization and placeholder diagnostics when a glyph/font cannot be used.
- The initial export compositor is FFmpeg-assisted. The Apple-native compatible
  path remains a v1.0 hardening task before public release.

Known limitations:

- No CSV/JSONL importer.
- No multi-source UI.
- No plugin execution.
- No Windows product UI.
- No claim of mathematically lossless output.
