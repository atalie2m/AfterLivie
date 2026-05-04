# 05. Phase 0 Technical Validation Plan

## 1. Purpose

Phase 0 is a hard technical validation gate. It is not a product demo and not a reduced public release.

The goal is to answer:

> Can the proposed architecture reliably compose a local video and timestamped comments into a high-quality replay export using Swift macOS UI, Rust core, shared overlay renderer, and macOS/FFmpeg media pipeline assumptions?

## 2. Required Vertical Slice

Phase 0 must implement the smallest end-to-end flow:

1. Create a macOS prototype shell.
2. Call Rust from Swift.
3. Import one canonical JSON comment file.
4. Normalize comments into Rust data model.
5. Probe one local video with ffprobe.
6. Generate a semantic render plan.
7. Render comments using the shared overlay renderer.
8. Compose source video and overlay into a larger canvas.
9. Export a 10–60 second preview segment.
10. Compare preview/export overlay behavior.
11. Record diagnostics.

## 3. Included Scope

### Application

- Minimal macOS shell.
- File selectors for video and comment JSON.
- Export button.
- Basic progress display.
- Basic error display.

### Rust Core

- Comment model.
- Source model with source ID.
- Canonical JSON importer.
- Timestamp normalization.
- Global offset.
- Visibility window.
- Render-plan generation.
- Diagnostics payload.

### Overlay Renderer

- Font loading policy prototype.
- Text shaping prototype.
- CJK wrapping sample.
- Emoji sequence sample.
- Multiline sample.
- Comment row layout.
- Sidebar panel layout.
- Transparent RGBA overlay frame output.
- Golden image fixture output.

### Media

- ffprobe metadata extraction.
- MP4/MOV H.264 or HEVC + AAC fast path sample.
- Source video placement into a larger canvas.
- Overlay alpha compositing.
- Audio handling smoke test.
- Export using one selected render path.

## 4. Excluded Scope

Do not implement during Phase 0:

- CSV importer,
- JSONL importer,
- multi-source UI,
- manual sync review UI,
- plugin runner,
- multiple templates,
- style editor,
- archive preset,
- Windows UI,
- platform API retrieval,
- batch processing.

## 5. Acceptance Criteria

Phase 0 is complete only if all of the following are true.

### End-to-End Export

- A 10–60 second replay export succeeds.
- Output canvas is larger than or equal to the source region plan.
- Source video is placed at exact integer coordinates.
- Overlay appears in the planned comment region.
- Audio is present or an explicit no-audio diagnostic is emitted.

### Shared Renderer

- Overlay text is rendered by the shared renderer, not native UI text.
- CJK sample text wraps without invalid splitting.
- Emoji sample does not break inside grapheme clusters.
- Multiline comments do not clip.
- Long author names have deterministic truncation.
- Missing font/glyph cases emit diagnostics.

### Preview/Export Consistency

- The same semantic render plan feeds preview and export.
- Comment ordering matches between preview and export.
- Wrapping and truncation match between preview and export.
- Comment visibility timing matches between preview and export.

### Media

- ffprobe metadata is normalized into `ReplayMediaInfo`.
- Media path decision is recorded.
- Unsupported media produces actionable diagnostics.
- Video placement does not introduce fractional scaling.

### Performance

- Dense sample segment is renderable without obvious UI lockup.
- Renderer cache is observable.
- No unbounded memory growth appears in short stress runs.
- Cancellation mechanism exists, even if primitive.

### Diagnostics

- Import diagnostics exist.
- Renderer diagnostics exist.
- Media diagnostics exist.
- Export diagnostics exist.
- Diagnostics can be saved with the project or sidecar output.

## 6. Required Test Fixtures

### Comment Fixtures

- `basic_en.json`
- `cjk_mixed.json`
- `emoji_sequences.json`
- `multiline.json`
- `long_author_names.json`
- `dense_30s.json`
- `comments_outside_duration.json`
- `invalid_rows.json`

### Media Fixtures

- MP4 H.264 AAC 720p.
- MOV H.264 AAC 1080p.
- MP4 with no audio.
- Sample with rotation metadata.
- Sample with VFR indicator if available.
- Unsupported or intentionally problematic file for diagnostics.

### Render Fixtures

- Sidebar 1920×1080 plan.
- Source region at 1280×720.
- Fixed comment panel width.
- Fixed font policy.
- Fixed comment window.

## 7. Phase 0 Milestones

### P0.1 Bridge Smoke Test

- Swift app calls Rust.
- Rust returns version and diagnostics.
- Build process is documented.

### P0.2 Import and Plan

- Canonical JSON imported.
- Comments normalized.
- Render plan generated.

### P0.3 Renderer Harness

- CLI or test harness renders overlay PNGs.
- Golden fixtures generated.
- CJK/emoji/multiline cases pass basic visual inspection.

### P0.4 Media Probe and Export

- ffprobe metadata read.
- 10–60 second export produced.
- Overlay composited over or beside source video.

### P0.5 Consistency and Decision

- Preview/export consistency checked.
- Renderer ADR updated.
- Continue/revisit decision made.

## 8. Kill / Revisit Gates

### Continue if

- shared renderer output is visually acceptable,
- renderer can be integrated into export path,
- media path can produce a valid segment,
- packaging/licensing looks manageable,
- diagnostics can explain failures,
- performance is not obviously fatal.

### Revisit if

- renderer stack cannot handle CJK/emoji reliably,
- overlay rendering is too slow for dense sample segments,
- preview/export mismatch cannot be controlled,
- media composition path causes unacceptable source degradation,
- FFmpeg integration creates unresolved distribution blockers,
- Swift/Rust bridge proves too brittle.

## 9. Deliverables

- Phase 0 prototype source.
- Sample exported replay video.
- Render-plan fixture.
- Overlay golden images.
- Media diagnostics sample.
- Import diagnostics sample.
- Renderer ADR update.
- Go / Revisit / Stop decision memo.
