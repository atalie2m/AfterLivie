# Native Livestream Archive Replay Composer Execution Plan

This plan translates the proposal and the documentation set in `docs/` into an execution ledger that can be followed until the product reaches the proposal's long-term direction.

The plan is intentionally large. It is meant to be used as a working checklist, release-gate reference, risk register, and handoff document for implementation work.

## Source Documents

- `docs/00_decision_summary.md`
- `docs/01_product_proposal_v2.md`
- `docs/02_scope_partition_release_plan.md`
- `docs/03_architecture_overview.md`
- `docs/04_renderer_adr.md`
- `docs/05_phase0_technical_validation_plan.md`
- `docs/06_implementation_plan.md`
- `docs/07_data_model_storage_schema.md`
- `docs/08_media_pipeline_plan.md`
- `docs/09_testing_validation_matrix.md`
- `docs/10_risks_and_gates.md`
- `docs/11_backlog_v1x_future.md`

## Product Mission

Build a macOS-first native application that transforms a local livestream archive video and prepared timestamped comments into a synchronized replay video.

The product restores livestream context by treating comments, metadata, pinned messages, badges, reactions, and platform labels as timeline events. It is not a general-purpose video editor.

The path is:

1. Prove the architecture with a narrow technical vertical slice.
2. Ship a production-grade single-source v1.0 replay composer.
3. Expand imports, source management, sync review, and layout controls in v1.x.
4. Keep future tracks designed but gated: plugins, automation, CLI, Windows, archive exports, and platform import.

## Operating Rules

- Do not expand the product surface before the render path is proven.
- Do not build broad UI before the Swift/Rust bridge, core model, renderer, media probe, and segment export work end to end.
- Do not let SwiftUI, AppKit, AVFoundation, VideoToolbox, WinUI, DirectWrite, Direct2D, or any platform-native type enter `ReplayCore`.
- Do not let the UI perform authoritative timeline sorting, comment visibility, text measurement, wrapping, truncation, or render-plan logic.
- Do not let preview and export use different semantic overlay behavior.
- Do not mutate imported comment timestamps when applying global or per-source offsets.
- Do not add plugin execution, platform API import, VOD download, multi-source UX, or deep style editing to the v1.0 release blockers.
- Do not promise mathematically lossless output. Promise visible quality controls, diagnostics, reproducibility inputs, and source-preserving layout decisions where feasible.
- Treat Phase 0 as a hard technical gate, not a demo.

## Current Implementation Status

2026-05-04:

- `[~]` Repository bootstrap is in place with a Nix flake dev shell, Rust workspace manifest, `justfile`, CI skeleton, SwiftPM macOS app shell, Codex Run action, fixtures, and v1.0 starter docs.
- `[~]` Phase 0 vertical-slice code exists across Rust crates for core models, canonical JSON import, render-plan generation, CPU overlay PNG/RGBA rendering, ffprobe normalization, project package/SQLite storage, FFmpeg-assisted export, C ABI bridge, and CLI harness.
- `[~]` macOS product shell exists as a SwiftPM `DocumentGroup` app with import, preview, export, diagnostics, settings, and CLI-backed service surfaces.
- `[x]` Nix was found at `/nix/var/nix/profiles/default/bin/nix`; `flake.lock` and `Cargo.lock` were generated.
- `[x]` `nix run .#doctor`, `nix run .#format -- --ci`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo nextest run` pass under Nix. Nextest reports 24 passed tests, with one non-failing leaky-process note from the overlay test binary to investigate during hardening.
- `[x]` The first render-plan fixture, semantic consistency report, and overlay PNG were generated from `fixtures/comments/basic_en.json`.
- `[x]` A synthetic temporary H.264/AAC MP4 was generated, probed as Apple-native-compatible, and exported through the FFmpeg-assisted preview path to `/tmp/afterlivie-preview.mp4` at 1760x720 with AAC audio.
- `[x]` v1.1 import/source expansion is implemented with CSV and JSONL importers, source manifests, import preview, source inspector controls, per-source enable/offset fields, merged timeline diagnostics, fixtures, and passing Rust/Swift gates.
- `[x]` v1.2 multi-source sync and merge UX is implemented with source-priority ordering, sync diagnostics, conservative UTC-metadata alignment suggestions, merged/split multi-platform render layouts, platform/source badges, macOS Sync Review UI, layout selection, source priority controls, offset nudge controls, anchor sync, fixtures, and passing Rust/Swift gates.
- `[!]` The Apple-native AVFoundation/VideoToolbox render path, production typography stack commitment, bundled/license-reviewed media strategy, golden baselines, sample media corpus, and release hardening gates remain open before v1.0 can be considered complete.

## Non-Negotiable Invariants

- Normalized comments are immutable after import.
- Source offsets are non-destructive.
- Every comment has a source ID from day one.
- Render plans are serializable and platform-neutral.
- Preview and export use the same semantic overlay model.
- Renderer owns all authoritative text measurement.
- Renderer owns wrapping, truncation, row measurement, badge placement, emoji handling, CJK line breaking, and RGBA overlay rasterization.
- Core owns source offsets, timeline ordering, comment visibility, region planning, style tokens, schema validation, and render-plan generation.
- Project files cannot store Apple-specific or Windows-specific object encodings.
- Media backend decisions are recorded in diagnostics.
- Font policy, renderer version, template version, media backend, FFmpeg version, source hash, comment hash, importer ID, and importer version are reproducibility inputs.
- Unsupported media, import failures, renderer failures, export failures, and project schema failures produce structured diagnostics with actionable hints.

## Release Map

### Phase 0: Technical Validation

Purpose: prove that the architecture can produce a 10 to 60 second replay video from one local video and one canonical JSON comment source.

Required outcome:

- Swift macOS shell calls Rust.
- Rust imports canonical JSON.
- Rust normalizes comments and emits diagnostics.
- Rust generates a semantic render plan.
- Shared renderer renders comment overlays into RGBA frames.
- Media path probes a video with `ffprobe`.
- Export path composes source video plus overlay into a larger canvas.
- Segment export succeeds.
- Preview/export overlay consistency is checked.
- Renderer ADR is updated with evidence.
- A go, revisit, or stop decision is recorded.

### v1.0: Single-Source Production Replay Composer

Purpose: ship a narrow but credible product.

Required outcome:

- Native macOS document app.
- One local video per project.
- One canonical JSON comment source exposed in UI.
- Project save/load.
- Global sync offset.
- One default sidebar layout.
- Preview segment render.
- Full export render.
- Render progress and cancellation.
- Import, media, renderer, export, and project diagnostics.
- Shared renderer in production export path.
- Preview/export overlay consistency tests.
- FFmpeg probing and compatibility diagnostics.
- Apple-native compatible media path with fallback diagnostics.
- Renderer decision record finalized.

### v1.1: Import and Source Expansion

Purpose: broaden input coverage without changing renderer fundamentals.

Required outcome:

- CSV importer.
- JSONL importer.
- Column mapping UI.
- Import preview table.
- Streaming large-file import.
- Better skipped-row diagnostics.
- Multiple comment source model exposed in inspector.
- Per-source enable/disable.
- Per-source offset fields.
- Single merged timeline view in diagnostics or preview.

### v1.2: Multi-Source Sync and Merge UX

Purpose: make simulcast and multi-platform workflows usable.

Required outcome:

- Multiple sources in UI.
- Manual offset review UI.
- Side-by-side source preview.
- Merged timeline preview.
- Stable source priority and merge ordering.
- Source-level platform labels.
- Merged multi-platform layout.
- Split-platform review layout.
- Merge diagnostics.
- Timestamp-based automatic alignment when reliable metadata exists.

### v1.3: Layout and Styling Expansion

Purpose: improve design flexibility without becoming a general-purpose editor.

Required outcome:

- Additional built-in layout templates.
- Style token controls.
- Metadata region configuration.
- Pinned comment display controls.
- Platform badge styling.
- User template saving.
- Template validation diagnostics.
- Golden tests per template.

### Future Tracks

Purpose: keep the long-term product reachable without blocking the narrow releases.

Tracks:

- Plugin runner.
- CLI and automation.
- Batch rendering.
- Windows feasibility and later productization.
- Platform import format adapters.
- Research and archive exports.
- Interactive web replay only if reprioritized.

## Target Repository Structure

Create or evolve toward this structure:

```text
AfterLivie/
  apps/
    macos/
  crates/
    replay_core/
    replay_importers/
    replay_overlay/
    replay_text/
    replay_media/
    replay_storage/
    replay_cli/
  bindings/
    swift/
  fixtures/
    comments/
    media/
    render_plans/
    golden_images/
    projects/
  docs/
  tools/
    media/
    renderer/
    schema/
    ci/
```

## Module Ownership

### ReplayMacApp

Owns:

- native macOS document windows,
- menus,
- file dialogs,
- drag and drop,
- inspectors,
- preview controls,
- timeline scrubber,
- settings,
- progress UI,
- cancellation UI,
- diagnostics presentation,
- project missing-media recovery UI.

Does not own:

- canonical import semantics,
- timeline sorting,
- source merge logic,
- authoritative layout,
- render-plan generation,
- text measurement,
- media compatibility decisions.

### ReplayBridge

Owns:

- Swift/Rust boundary,
- coarse-grained command APIs,
- structured diagnostics across the bridge,
- opaque handles where needed,
- JSON fixture export for debugging,
- memory ownership rules,
- bridge integration tests.

Preferred shape:

- Start with a small C ABI or UniFFI spike in Phase 0.
- Select final strategy before v1.0 UI expansion.
- Keep APIs coarse.
- Avoid high-frequency per-frame Swift-to-Rust calls unless measured safe.

### ReplayCore

Owns:

- project schema,
- normalized comment model,
- comment source model,
- timestamp normalization,
- canonical import orchestration,
- importer registry interfaces,
- diagnostics model,
- timeline sorting,
- source offsets,
- comment visibility windows,
- merge model,
- layout region planning,
- render-plan generation,
- project serialization,
- schema migration,
- reproducibility metadata.

Must not contain:

- SwiftUI,
- AppKit,
- AVFoundation,
- VideoToolbox,
- WinUI,
- DirectWrite,
- Direct2D,
- platform-native font objects,
- platform-native media objects.

### ReplayImporters

Owns:

- canonical JSON importer for v1.0,
- CSV importer for v1.1,
- JSONL importer for v1.1,
- importer trait,
- row-level diagnostics,
- encoding detection policy,
- importer versioning,
- deterministic ID generation,
- import fingerprints,
- large-file streaming behavior.

### ReplayOverlay

Owns:

- font resolution,
- font fallback,
- Unicode segmentation,
- grapheme boundaries,
- CJK line breaking,
- text shaping,
- paragraph layout,
- truncation,
- ellipsis,
- row measurement,
- badge and emote inline placement,
- panel layout,
- pinned and metadata overlay layout,
- RGBA rasterization,
- alpha handling policy,
- golden image output,
- renderer diagnostics,
- cache keys and invalidation.

Candidate stack to validate:

- HarfBuzz or equivalent for shaping.
- ICU or equivalent for segmentation and line breaking.
- Skia, SkParagraph, or equivalent for 2D/paragraph rendering.
- Controlled bundled font stack where legally viable.
- CPU raster path first.
- GPU acceleration only after correctness and profiling.

### ReplayText

Owns text-specific helpers if separated from `ReplayOverlay`:

- grapheme segmentation wrappers,
- line-break classification,
- font policy identifiers,
- fallback resolution,
- text fixture generation,
- reusable shaping test harness.

### ReplayMedia

Owns:

- `ffprobe` integration,
- media metadata normalization,
- compatibility decision tree,
- proxy and intermediate decisions,
- remux/transcode decisions,
- audio policy,
- media diagnostics,
- FFmpeg distribution abstraction.

### ReplayRenderApple

Owns:

- compatible macOS render execution,
- source frame reading,
- source frame placement,
- overlay frame compositing,
- AVFoundation and VideoToolbox writer integration,
- progress,
- cancellation,
- audio passthrough attempt,
- audio re-encode fallback,
- render diagnostics.

### ReplayRenderFFmpeg

Owns:

- assisted workflows,
- proxy generation,
- intermediate generation,
- remuxing,
- audio conversion,
- final mux assistance where required,
- fallback diagnostics.

### ReplayStorage

Owns:

- project package creation,
- atomic project saves,
- project migrations,
- `project.json`,
- `comments.sqlite`,
- source sidecar files,
- assets,
- templates,
- caches,
- diagnostics,
- render history,
- reproducibility manifests.

### ReplayCLI

Owns:

- importer harness,
- render-plan generator,
- overlay frame generator,
- media probe command,
- preview segment command,
- batch render prototype,
- headless regression testing.

## Data Model Plan

### CommentEvent

Required fields:

- `id`
- `source_id`
- `platform`
- `timestamp_ms`
- `original_timestamp`
- `kind`
- `author`
- `body`
- `style`
- `badges`
- `emotes`
- `source`
- `raw_ref`
- `import_order`

Tasks:

- [ ] Define Rust `CommentEvent`.
- [ ] Define `CommentKind`.
- [ ] Define `CommentAuthor`.
- [ ] Define `CommentBody`.
- [ ] Define `CommentStyle`.
- [ ] Define `Badge`.
- [ ] Define `Emote`.
- [ ] Define `OriginalTimestamp`.
- [ ] Define `RawSourceRef`.
- [ ] Add serde serialization.
- [ ] Add schema fixture.
- [ ] Add deterministic equality tests.
- [ ] Add invalid-field diagnostics.

### CommentSource

Required fields:

- `source_id`
- `display_name`
- `platform`
- `importer_id`
- `original_file_ref`
- `enabled`
- `offset_ms`
- `timestamp_basis`
- `diagnostics_ref`
- `visual_style`
- `metadata`
- `import_fingerprint`

Tasks:

- [ ] Define Rust `CommentSource`.
- [ ] Enforce unique source IDs within a project.
- [ ] Store source IDs even for v1.0 single-source projects.
- [ ] Store offset separately from comment timestamps.
- [ ] Add source enable/disable model for future use.
- [ ] Add import fingerprint.
- [ ] Add source diagnostics reference.
- [ ] Add project fixture with one source.
- [ ] Add project fixture with multiple sources for future compatibility.

### TimestampBasis

Supported values:

- `relative_to_video_start`
- `relative_to_stream_start`
- `absolute_utc`
- `local_time_with_timezone`
- `local_time_ambiguous`
- `unknown`

Tasks:

- [ ] Define enum.
- [ ] Require v1.0 canonical JSON to resolve confidently to video-relative time.
- [ ] Emit synchronization-required diagnostics for ambiguous formats.
- [ ] Preserve original timestamp where available.
- [ ] Normalize authoritative timestamp to integer milliseconds.
- [ ] Add tests for milliseconds, seconds, `HH:MM:SS`, `MM:SS`, invalid formats, negative values, and out-of-duration values.

### Stable ID Policy

Rules:

- `source_id` is unique within a project.
- `comment.id` is unique within a source.
- Global comment key is `{source_id}:{comment.id}`.
- Generated IDs are deterministic for the same input file and importer version.
- Duplicate IDs produce diagnostics, not silent overwrites.

Tasks:

- [ ] Define global key helper.
- [ ] Define deterministic generated ID algorithm.
- [ ] Include importer version in generated ID inputs where needed.
- [ ] Detect duplicate IDs.
- [ ] Emit row-level duplicate diagnostics.
- [ ] Add duplicate ID fixture.
- [ ] Add stable ID regression tests.

### ImportResult

Required fields:

- `source`
- `comments`
- `diagnostics`
- `skipped_count`
- `importer_version`
- `detected_format`
- `detected_encoding`

Tasks:

- [ ] Define `ImportResult`.
- [ ] Add importer success fixture.
- [ ] Add importer partial success fixture.
- [ ] Add importer fatal failure fixture.
- [ ] Expose summary counts to Swift.
- [ ] Persist diagnostics to project package.

### Diagnostic

Required fields:

- `id`
- `severity`
- `category`
- `message`
- `source_ref`
- `code`
- `actionable_hint`

Severities:

- `info`
- `warning`
- `error`
- `fatal`

Categories:

- `import`
- `timestamp`
- `media`
- `renderer`
- `export`
- `project`
- `storage`
- `sync`
- `schema`

Tasks:

- [ ] Define `Diagnostic`.
- [ ] Define severity enum.
- [ ] Define category enum.
- [ ] Define source references for files, rows, fields, timestamps, comments, sources, media streams, renderer components, and export stages.
- [ ] Add stable diagnostic codes.
- [ ] Add actionable hints for common failures.
- [ ] Add diagnostic serialization tests.
- [ ] Add UI-ready diagnostic summaries.
- [ ] Add redaction policy for local paths.

## Project Package Plan

Target structure:

```text
ProjectName.replayproj/
  project.json
  comments.sqlite
  sources/
    main.source.json
  assets/
    avatars/
    badges/
    emotes/
    backgrounds/
    fonts/
  templates/
    custom-layouts/
  cache/
    thumbnails/
    waveform/
    preview-renders/
    overlay-cache/
    media-proxies/
    intermediates/
  diagnostics.json
  render-history.json
  reproducibility.json
```

Tasks:

- [ ] Define project bundle extension.
- [ ] Define `project.json` schema version `1.0`.
- [ ] Define project ID generation.
- [ ] Define created/updated timestamps.
- [ ] Define source video asset reference.
- [ ] Define media info reference.
- [ ] Define comment source list.
- [ ] Define layout template selection.
- [ ] Define layout overrides object.
- [ ] Define render settings object.
- [ ] Define overlay renderer object.
- [ ] Define diagnostics sidecar.
- [ ] Define render history sidecar.
- [ ] Define reproducibility sidecar.
- [ ] Define atomic save strategy.
- [ ] Define recovery strategy for interrupted saves.
- [ ] Define missing source media strategy.
- [ ] Define cache cleanup strategy.
- [ ] Add project package validation command.
- [ ] Add project load/save tests.
- [ ] Add migration fixture tests.
- [ ] Add large comments store fixture.

### SQLite Plan

Suggested v1.0 table:

```sql
CREATE TABLE comments (
  global_key TEXT PRIMARY KEY,
  source_id TEXT NOT NULL,
  comment_id TEXT NOT NULL,
  timestamp_ms INTEGER NOT NULL,
  kind TEXT NOT NULL,
  author_json TEXT,
  body_json TEXT NOT NULL,
  style_json TEXT,
  badges_json TEXT,
  emotes_json TEXT,
  raw_ref_json TEXT,
  import_order INTEGER NOT NULL
);

CREATE INDEX idx_comments_time ON comments(timestamp_ms);
CREATE INDEX idx_comments_source_time ON comments(source_id, timestamp_ms);
```

Tasks:

- [ ] Decide whether Phase 0 stores comments in memory only.
- [ ] Decide whether v1.0 requires SQLite before alpha.
- [ ] Add SQLite schema if large-file tests require it.
- [ ] Add query for visible comment window.
- [ ] Add query for source-specific visible window.
- [ ] Add insert transaction for import results.
- [ ] Add duplicate global key protection.
- [ ] Add schema version table.
- [ ] Add migration test fixtures.
- [ ] Add comments database integrity check.

## Render Plan Plan

### SemanticRenderPlan

Owns:

- canvas logical dimensions,
- source video region,
- comment panel region,
- metadata regions,
- source offsets,
- comment visibility rules,
- ordering rules,
- style tokens,
- font policy,
- overlay components,
- audio policy,
- media backend decision.

Tasks:

- [ ] Define Rust `SemanticRenderPlan`.
- [ ] Define canvas object.
- [ ] Define region objects.
- [ ] Define source video region.
- [ ] Define comment panel region.
- [ ] Define metadata regions.
- [ ] Define timeline object.
- [ ] Define comment window.
- [ ] Define global offset.
- [ ] Define per-source offset list.
- [ ] Define source enable flags.
- [ ] Define ordering rules.
- [ ] Define style tokens.
- [ ] Define font policy ID.
- [ ] Define emoji policy.
- [ ] Define renderer ID.
- [ ] Define audio policy.
- [ ] Define media backend decision.
- [ ] Add JSON fixture.
- [ ] Add schema validation.
- [ ] Add deterministic generation tests.
- [ ] Add serialization roundtrip tests.

### RenderExecutionProfile

Owns:

- preview or final export mode,
- frame range,
- output scale,
- bitrate,
- encoder,
- cache policy,
- temporary file policy,
- progress reporting mode.

Tasks:

- [ ] Define Rust `RenderExecutionProfile`.
- [ ] Define preview segment profile.
- [ ] Define final export profile.
- [ ] Define output scale constraints.
- [ ] Define bitrate policy enum.
- [ ] Define encoder settings.
- [ ] Define cache policy.
- [ ] Define temp file policy.
- [ ] Define progress reporting mode.
- [ ] Add tests proving preview/final differ only by execution profile where required.

## Phase 0 Detailed Plan

### P0.0 Repository Bootstrap

Goal: create the minimum development skeleton required for vertical-slice work.

Tasks:

- [ ] Confirm target macOS version.
- [ ] Confirm Swift toolchain and Xcode version.
- [ ] Confirm Rust stable version.
- [ ] Create Rust workspace.
- [ ] Create `crates/replay_core`.
- [ ] Create `crates/replay_importers`.
- [ ] Create `crates/replay_overlay`.
- [ ] Create `crates/replay_media`.
- [ ] Create `crates/replay_cli`.
- [ ] Create `bindings/swift`.
- [ ] Create `apps/macos`.
- [ ] Create fixtures directories.
- [ ] Create tools directories.
- [ ] Add formatting configuration.
- [ ] Add linting configuration.
- [ ] Add local development setup docs.
- [ ] Add CI skeleton for Rust tests.
- [ ] Add CI skeleton for macOS app build once app exists.
- [ ] Add license notices directory for third-party dependencies.

Exit:

- Rust workspace builds.
- Empty macOS shell target builds.
- Fixture directories exist.
- Developer setup is documented.

### P0.1 Bridge Smoke Test

Goal: prove Swift can call Rust and receive structured diagnostics.

Tasks:

- [ ] Choose Phase 0 bridge approach: UniFFI or C ABI.
- [ ] Implement Rust `replay_version()`.
- [ ] Implement Rust `replay_diagnostics_smoke()`.
- [ ] Expose functions to Swift.
- [ ] Call Rust from macOS shell.
- [ ] Show returned version in debug UI or log.
- [ ] Show returned diagnostics in debug UI or log.
- [ ] Add bridge ownership rules.
- [ ] Add bridge error propagation.
- [ ] Add bridge build instructions.
- [ ] Add test for large diagnostic payload.
- [ ] Add test for invalid input payload.
- [ ] Document bridge decision tradeoffs.

Acceptance:

- Swift calls Rust successfully.
- Rust returns version and diagnostics.
- Bridge can pass structured JSON or typed data.
- Build process is reproducible.

### P0.2 Canonical Import and Plan

Goal: import canonical JSON, normalize comments, and generate a semantic render plan.

Tasks:

- [ ] Define canonical JSON v1 input schema.
- [ ] Create `basic_en.json` fixture.
- [ ] Create `cjk_mixed.json` fixture.
- [ ] Create `emoji_sequences.json` fixture.
- [ ] Create `multiline.json` fixture.
- [ ] Create `long_author_names.json` fixture.
- [ ] Create `dense_30s.json` fixture.
- [ ] Create `comments_outside_duration.json` fixture.
- [ ] Create `invalid_rows.json` fixture.
- [ ] Implement canonical JSON parser.
- [ ] Implement timestamp parser.
- [ ] Implement timestamp normalization.
- [ ] Implement source creation.
- [ ] Implement deterministic ID generation.
- [ ] Implement duplicate ID diagnostics.
- [ ] Implement skipped row diagnostics.
- [ ] Implement global offset in plan generation.
- [ ] Implement visibility window calculation.
- [ ] Implement default sidebar layout plan.
- [ ] Serialize `SemanticRenderPlan`.
- [ ] Expose import and plan command through bridge.
- [ ] Add unit tests for importer.
- [ ] Add unit tests for plan generation.
- [ ] Add CLI command to import and dump plan.

Acceptance:

- Canonical JSON imports.
- Comments normalize into Rust model.
- Diagnostics are produced for invalid rows.
- Render plan JSON fixture is generated.
- Swift can trigger import and plan generation.

### P0.3 Renderer Harness

Goal: prove a shared renderer can render overlay frames with acceptable CJK, emoji, multiline, and dense comment behavior.

Tasks:

- [ ] Evaluate renderer stack candidates.
- [ ] Decide initial renderer stack for Phase 0.
- [ ] Define font policy prototype.
- [ ] Add bundled or referenced test font policy.
- [ ] Implement renderer environment initialization.
- [ ] Implement overlay plan input loading.
- [ ] Implement visible comment selection handoff.
- [ ] Implement author text layout.
- [ ] Implement body text layout.
- [ ] Implement multiline layout.
- [ ] Implement max line limit.
- [ ] Implement ellipsis behavior.
- [ ] Implement long author truncation.
- [ ] Implement CJK line breaking sample.
- [ ] Implement emoji grapheme sample.
- [ ] Implement missing glyph diagnostic.
- [ ] Implement basic badge placeholder.
- [ ] Implement panel background.
- [ ] Implement row separators.
- [ ] Implement transparent RGBA output.
- [ ] Implement PNG output for harness.
- [ ] Add golden image directory.
- [ ] Generate golden PNGs.
- [ ] Add visual inspection notes.
- [ ] Add basic pixel/golden comparison command.
- [ ] Add cache instrumentation.
- [ ] Add dense 30-second render stress run.
- [ ] Record renderer benchmark data.
- [ ] Record packaging and licensing notes.

Acceptance:

- Renderer outputs RGBA/PNG overlay frames.
- CJK sample wraps without invalid splitting.
- Emoji sample does not split inside grapheme clusters.
- Multiline sample does not clip.
- Long author names truncate deterministically.
- Missing font/glyph cases emit diagnostics.
- Golden output is stable enough for early CI.

### P0.4 Media Probe and Export

Goal: probe source media and export a short composed replay segment.

Tasks:

- [ ] Add `ffprobe` discovery strategy.
- [ ] Add `ReplayMediaInfo` model.
- [ ] Normalize duration.
- [ ] Normalize resolution.
- [ ] Normalize frame rate.
- [ ] Normalize CFR/VFR indicator.
- [ ] Normalize video codec.
- [ ] Normalize container.
- [ ] Normalize pixel format.
- [ ] Normalize audio presence.
- [ ] Normalize audio codec.
- [ ] Normalize audio track count.
- [ ] Normalize subtitle presence.
- [ ] Normalize color metadata.
- [ ] Normalize HDR/SDR indicator.
- [ ] Normalize bitrate.
- [ ] Normalize timebase.
- [ ] Normalize start time.
- [ ] Normalize rotation metadata.
- [ ] Normalize sample aspect ratio.
- [ ] Normalize default stream disposition.
- [ ] Implement compatibility decision tree.
- [ ] Detect Apple-native compatible MP4/MOV H.264/HEVC plus AAC.
- [ ] Emit diagnostics for unsupported containers.
- [ ] Emit diagnostics for unsupported codecs.
- [ ] Emit diagnostics for VFR.
- [ ] Emit diagnostics for rotation.
- [ ] Emit diagnostics for HDR or unusual color metadata.
- [ ] Emit diagnostics for multiple audio tracks.
- [ ] Implement source placement into larger canvas.
- [ ] Ensure integer source coordinates.
- [ ] Ensure no fractional scaling in default layout.
- [ ] Composite overlay RGBA frame.
- [ ] Encode 10 to 60 second segment.
- [ ] Attempt audio passthrough where safe.
- [ ] Re-encode AAC fallback if passthrough is unsafe or fails.
- [ ] Emit no-audio diagnostic when applicable.
- [ ] Add progress reporting.
- [ ] Add primitive cancellation.
- [ ] Persist export diagnostics.
- [ ] Add CLI preview export command.

Acceptance:

- `ffprobe` metadata is normalized.
- Media path decision is recorded.
- A 10 to 60 second replay segment exports.
- Source video is placed at exact integer coordinates.
- Overlay appears in planned comment region.
- Audio is present or explicit no-audio diagnostic is emitted.
- Unsupported media produces actionable diagnostics.

### P0.5 Preview/Export Consistency and Decision

Goal: prove preview and export share semantic overlay behavior and make the architecture decision.

Tasks:

- [ ] Define preview execution profile.
- [ ] Define export execution profile.
- [ ] Feed both from the same semantic render plan.
- [ ] Compare visible comments at selected timestamps.
- [ ] Compare comment ordering.
- [ ] Compare wrapping.
- [ ] Compare truncation.
- [ ] Compare row heights.
- [ ] Compare panel bounds.
- [ ] Confirm no native text fallback in export.
- [ ] Save consistency report.
- [ ] Update renderer ADR with evaluated stack.
- [ ] Add benchmark data to ADR.
- [ ] Add golden test evidence to ADR.
- [ ] Add packaging assessment to ADR.
- [ ] Add licensing assessment to ADR.
- [ ] Add chosen v1.0 renderer path to ADR.
- [ ] Add fallback path to ADR.
- [ ] Add known limitations to ADR.
- [ ] Add revisit triggers to ADR.
- [ ] Write go/revisit/stop decision memo.

Acceptance:

- Preview/export consistency check passes within defined tolerance.
- Renderer ADR is updated.
- Decision memo exists.
- Phase 0 deliverables are collected.

### Gate 0: Architecture Feasibility

Gate requirements:

- [ ] Swift calls Rust.
- [ ] Canonical import works.
- [ ] Renderer produces overlay frame.
- [ ] Media segment export succeeds.
- [ ] Diagnostics exist.
- [ ] Phase 0 decision memo exists.

Decision options:

- Go.
- Revisit renderer stack.
- Revisit media stack.
- Stop.

Do not proceed to broad UI if renderer or media fails this gate.

## Phase 1 Detailed Plan: v1.0 Core Foundation

Goal: create stable product data foundations.

### Core Crate

Tasks:

- [ ] Finalize `replay_core` public model boundaries.
- [ ] Move import-independent models into core.
- [ ] Move importer-specific code into `replay_importers`.
- [ ] Define error model.
- [ ] Define diagnostics model.
- [ ] Define project model.
- [ ] Define render-plan model.
- [ ] Define source model.
- [ ] Define timeline model.
- [ ] Define style token model.
- [ ] Define audio policy model.
- [ ] Define media decision references.
- [ ] Add serde support.
- [ ] Add schema version support.
- [ ] Add roundtrip tests.
- [ ] Add JSON schema generation if useful.

### Canonical JSON Importer

Tasks:

- [ ] Stabilize canonical JSON v1 schema.
- [ ] Add schema documentation.
- [ ] Add happy path fixture.
- [ ] Add invalid field fixture.
- [ ] Add duplicate ID fixture.
- [ ] Add missing timestamp fixture.
- [ ] Add comments outside duration fixture.
- [ ] Add large file fixture.
- [ ] Add importer version constant.
- [ ] Add import fingerprint.
- [ ] Add row-level diagnostics.
- [ ] Add skipped row count.
- [ ] Add fatal malformed JSON behavior.
- [ ] Add encoding detection or explicit UTF-8 requirement.

### Timeline

Tasks:

- [ ] Define stable sort order.
- [ ] Sort by effective timestamp.
- [ ] Break ties by event priority.
- [ ] Break ties by source priority.
- [ ] Break ties by import order.
- [ ] Break ties by stable ID.
- [ ] Define comment visibility window.
- [ ] Define max visible comments.
- [ ] Define disabled source behavior for future support.
- [ ] Add global offset tests.
- [ ] Add negative offset tests.
- [ ] Add out-of-range offset tests.
- [ ] Add source offset tests.
- [ ] Add future multi-source merge tests.

### Project Schema

Tasks:

- [ ] Implement `project.json` v1.
- [ ] Implement project creation.
- [ ] Implement project load.
- [ ] Implement project save.
- [ ] Implement project validation.
- [ ] Implement project migration skeleton.
- [ ] Implement source video asset reference.
- [ ] Implement comment source references.
- [ ] Implement layout template reference.
- [ ] Implement render settings.
- [ ] Implement renderer metadata.
- [ ] Implement reproducibility metadata.
- [ ] Add schema fixtures.
- [ ] Add migration fixtures.
- [ ] Add invalid project diagnostics.

### Core Foundation Exit Criteria

- [ ] `replay_core` crate exists with stable model boundaries.
- [ ] canonical JSON importer is reliable for v1.0 fixtures.
- [ ] render plan generation is deterministic.
- [ ] project schema v1 loads and saves.
- [ ] diagnostics are structured and UI-ready.
- [ ] source IDs and offsets are modeled for future multi-source support.

## Phase 2 Detailed Plan: v1.0 Shared Overlay Renderer

Goal: move renderer from prototype to production path.

### Renderer Environment

Tasks:

- [ ] Finalize v1.0 renderer stack.
- [ ] Finalize renderer dependency licenses.
- [ ] Finalize font policy v1.
- [ ] Decide bundled fonts vs system fonts plus diagnostics.
- [ ] Define emoji policy.
- [ ] Define missing glyph policy.
- [ ] Define renderer environment initialization.
- [ ] Define asset loading.
- [ ] Define renderer version reporting.
- [ ] Define renderer cache root.
- [ ] Define renderer diagnostics format.

### Text Layout

Tasks:

- [ ] Implement grapheme-aware segmentation.
- [ ] Implement CJK-aware line breaking.
- [ ] Implement mixed Latin/CJK wrapping.
- [ ] Implement punctuation behavior.
- [ ] Implement combining mark behavior.
- [ ] Implement ZWJ emoji behavior.
- [ ] Implement variation selector handling.
- [ ] Implement skin tone emoji handling.
- [ ] Implement paragraph measurement.
- [ ] Implement baseline calculation.
- [ ] Implement line height calculation.
- [ ] Implement max width.
- [ ] Implement max lines.
- [ ] Implement ellipsis.
- [ ] Implement deterministic truncation.

### Comment Row Layout

Tasks:

- [ ] Implement author plus body row.
- [ ] Implement long author truncation.
- [ ] Implement author color token.
- [ ] Implement body text token.
- [ ] Implement multiline body.
- [ ] Implement row padding.
- [ ] Implement row spacing.
- [ ] Implement row height measurement.
- [ ] Implement dense panel overflow policy.
- [ ] Implement pinned placeholder or reserved model.
- [ ] Implement badge placeholder.
- [ ] Implement emote placeholder.
- [ ] Emit diagnostics for missing badge/emote assets.

### Panel Layout

Tasks:

- [ ] Implement comment panel bounds.
- [ ] Implement panel background.
- [ ] Implement panel separators.
- [ ] Implement scrolling or stacking behavior.
- [ ] Implement visible comment clipping policy.
- [ ] Implement metadata region placeholders.
- [ ] Implement debug layout overlay.
- [ ] Implement layout diagnostics.

### Raster and Alpha

Tasks:

- [ ] Define RGBA output format.
- [ ] Define premultiplied alpha policy.
- [ ] Validate transparent background.
- [ ] Validate panel backgrounds.
- [ ] Validate alpha compositing with media backend.
- [ ] Add raster tests.
- [ ] Add pixel tests for transparency.

### Renderer Cache

Tasks:

- [ ] Define cache keys.
- [ ] Include renderer version in cache keys.
- [ ] Include font policy version in cache keys.
- [ ] Include template version in cache keys.
- [ ] Include style token hash in cache keys.
- [ ] Cache shaped runs.
- [ ] Cache paragraph measurements.
- [ ] Cache static row fragments.
- [ ] Cache badge/emote raster assets.
- [ ] Add cache invalidation tests.
- [ ] Add cache performance instrumentation.

### Golden Tests

Tasks:

- [ ] Add default sidebar golden.
- [ ] Add basic English golden.
- [ ] Add CJK mixed golden.
- [ ] Add emoji golden.
- [ ] Add multiline golden.
- [ ] Add long author golden.
- [ ] Add dense panel golden.
- [ ] Add missing glyph diagnostic fixture.
- [ ] Add golden update workflow.
- [ ] Add golden comparison tolerance.
- [ ] Add CI integration.

### Renderer API

Recommended coarse API:

```text
load_renderer_environment(font_policy, assets) -> RendererEnvironment
measure_overlay(overlay_plan, time_ms) -> OverlayMeasurement
render_overlay_frame(overlay_plan, time_ms, output_scale) -> RgbaFrame
render_debug_overlay(overlay_plan, time_ms) -> RgbaFrame + LayoutDiagnostics
prewarm_cache(time_range, comment_ids) -> CacheReport
```

Tasks:

- [ ] Implement environment loading API.
- [ ] Implement measurement API.
- [ ] Implement frame rendering API.
- [ ] Implement debug rendering API.
- [ ] Implement cache prewarm API.
- [ ] Expose necessary APIs through bridge or render backend.
- [ ] Avoid per-glyph APIs across Swift boundary.

### Renderer Exit Criteria

- [ ] Renderer stack is committed.
- [ ] CJK fixture passes.
- [ ] Emoji fixture passes.
- [ ] Multiline fixture passes.
- [ ] Golden images are stable enough for CI.
- [ ] Dense sample performance is acceptable enough to continue.
- [ ] Packaging and licensing are plausible.
- [ ] Renderer diagnostics are usable.

## Phase 3 Detailed Plan: v1.0 Media and Export Pipeline

Goal: build reliable segment and full export.

### Media Probe

Tasks:

- [ ] Discover `ffprobe`.
- [ ] Decide bundled vs external FFmpeg strategy.
- [ ] Capture FFmpeg version.
- [ ] Probe duration.
- [ ] Probe resolution.
- [ ] Probe frame rate.
- [ ] Probe CFR/VFR indicator.
- [ ] Probe codec.
- [ ] Probe container.
- [ ] Probe pixel format.
- [ ] Probe audio track presence.
- [ ] Probe audio codec.
- [ ] Probe audio track count.
- [ ] Probe subtitle track presence.
- [ ] Probe color metadata.
- [ ] Probe HDR/SDR indicator.
- [ ] Probe bitrate.
- [ ] Probe timebase.
- [ ] Probe start time.
- [ ] Probe rotation metadata.
- [ ] Probe sample aspect ratio.
- [ ] Probe stream disposition.
- [ ] Normalize all fields into `ReplayMediaInfo`.
- [ ] Add media info fixture.
- [ ] Add media info serialization test.

### Compatibility Decision Tree

Tasks:

- [ ] Define compatible MP4/MOV path.
- [ ] Define compatible H.264 path.
- [ ] Define compatible HEVC path.
- [ ] Define compatible AAC path.
- [ ] Define unsupported container diagnostic.
- [ ] Define remux candidate decision.
- [ ] Define video intermediate candidate decision.
- [ ] Define audio transcode candidate decision.
- [ ] Define HDR warning policy.
- [ ] Define VFR warning policy.
- [ ] Define rotation policy.
- [ ] Define SAR policy.
- [ ] Define multiple audio track policy.
- [ ] Define subtitle ignored diagnostic.
- [ ] Define unsupported fatal diagnostic.
- [ ] Add decision tree tests.

### Source Quality Rules

Tasks:

- [ ] Preserve source dimensions where possible.
- [ ] Use larger canvas instead of shrinking source where feasible.
- [ ] Avoid fractional scaling.
- [ ] Record any scaling.
- [ ] Record color metadata.
- [ ] Record HDR limitations.
- [ ] Record VFR limitations.
- [ ] Add frame placement tests.
- [ ] Add crop comparison tests where feasible.
- [ ] Add visual QA checklist.

### Render Composition

Tasks:

- [ ] Implement output canvas allocation.
- [ ] Implement source video frame decode/read.
- [ ] Implement integer source placement.
- [ ] Request overlay RGBA frame for timestamp.
- [ ] Composite overlay onto canvas.
- [ ] Handle alpha policy correctly.
- [ ] Encode preview segment.
- [ ] Encode final export.
- [ ] Handle render progress.
- [ ] Handle cancellation.
- [ ] Handle temporary files.
- [ ] Clean temporary files on success.
- [ ] Preserve failed diagnostics on failure.
- [ ] Add render history entry.

### Audio

v1.0 policy:

1. If source is MP4/MOV with AAC audio and no audio edits are required, attempt passthrough where technically safe.
2. If passthrough is unsafe or fails, re-encode to AAC.
3. If source audio is not AAC, re-encode to AAC.
4. If source has no audio, emit diagnostic and export silent video.
5. If multiple audio tracks exist, select primary/default track and emit diagnostic.

Tasks:

- [ ] Detect primary/default audio track.
- [ ] Attempt AAC passthrough where safe.
- [ ] Detect passthrough failure.
- [ ] Fallback to AAC re-encode.
- [ ] Export no-audio source with explicit diagnostic.
- [ ] Select primary track for multi-track source.
- [ ] Record selected audio track.
- [ ] Record passthrough attempted.
- [ ] Record passthrough success/failure.
- [ ] Record fallback used.
- [ ] Record codec and bitrate.
- [ ] Add audio/video sync smoke check.

### Presets

Tasks:

- [ ] Define preview preset.
- [ ] Define high-quality upload preset.
- [ ] Define bitrate policy.
- [ ] Define encoder choice.
- [ ] Define output extension.
- [ ] Define metadata writing policy.
- [ ] Add preset serialization.
- [ ] Add preset diagnostics.

### Media Fixtures

Required:

- [ ] MP4 H.264 AAC 720p.
- [ ] MOV H.264 AAC 1080p.
- [ ] MP4 with no audio.
- [ ] Sample with rotation metadata.
- [ ] Sample with VFR indicator if available.
- [ ] Unsupported or intentionally problematic file.
- [ ] Multiple audio track sample if available.
- [ ] HDR or unusual color sample if available.

### Media Exit Criteria

- [ ] Preview segment export works.
- [ ] Full export works on supported fixture.
- [ ] Unsupported source emits actionable diagnostics.
- [ ] Audio policy works for no-audio and AAC sources.
- [ ] Progress works.
- [ ] Cancellation works.
- [ ] Media diagnostics are saved.

## Phase 4 Detailed Plan: v1.0 macOS Product Shell

Goal: build the minimal native workflow.

### Document App

Tasks:

- [ ] Create macOS app target.
- [ ] Define document type.
- [ ] Implement new project.
- [ ] Implement open project.
- [ ] Implement save project.
- [ ] Implement save as.
- [ ] Implement crash-safe save integration.
- [ ] Implement missing media detection.
- [ ] Implement missing media relink UI.
- [ ] Add app menu commands.
- [ ] Add document window layout.
- [ ] Add native error presentation.

### Import Flow

Tasks:

- [ ] Add local video picker.
- [ ] Add drag-and-drop video import.
- [ ] Probe selected video.
- [ ] Show media compatibility summary.
- [ ] Add canonical JSON picker.
- [ ] Add drag-and-drop comment import.
- [ ] Run canonical importer.
- [ ] Show import summary.
- [ ] Show diagnostics panel.
- [ ] Save imported source into project.

### Preview Flow

Tasks:

- [ ] Add basic video preview.
- [ ] Add timeline scrubber.
- [ ] Add current time display.
- [ ] Add global offset control.
- [ ] Add default layout preview.
- [ ] Add preview segment start/duration controls.
- [ ] Trigger preview segment render.
- [ ] Display preview render result.
- [ ] Show preview diagnostics.

### Export Flow

Tasks:

- [ ] Add full export dialog.
- [ ] Show output path selection.
- [ ] Show preset selection.
- [ ] Show compatibility warnings.
- [ ] Start export.
- [ ] Show progress.
- [ ] Support cancellation.
- [ ] Show completion summary.
- [ ] Show export diagnostics.
- [ ] Write render history.

### Diagnostics UI

Tasks:

- [ ] Display diagnostic severity.
- [ ] Display diagnostic category.
- [ ] Display message.
- [ ] Display actionable hint.
- [ ] Display source reference.
- [ ] Filter by severity.
- [ ] Filter by category.
- [ ] Group by import/media/renderer/export/project.
- [ ] Support copying diagnostic summary.
- [ ] Redact local paths where needed.

### Settings

Tasks:

- [ ] Add basic settings window or inspector.
- [ ] Show FFmpeg path/status.
- [ ] Show renderer version.
- [ ] Show font policy.
- [ ] Show cache location.
- [ ] Add cache cleanup action.
- [ ] Add default export preset selection.

### UI Exit Criteria

- [ ] User can create project.
- [ ] User can select video.
- [ ] User can import canonical JSON.
- [ ] User can see diagnostics.
- [ ] User can adjust global offset.
- [ ] User can preview segment.
- [ ] User can export final.
- [ ] User can cancel render.
- [ ] User can save and reopen project.
- [ ] UI does not own authoritative layout, timeline, or text measurement.

## Phase 5 Detailed Plan: v1.0 Hardening

Goal: make the narrow product reliable enough to release.

### Stability

Tasks:

- [ ] Add crash-safe project saves.
- [ ] Add save recovery tests.
- [ ] Add corrupted project diagnostics.
- [ ] Add missing source media recovery.
- [ ] Add missing comment store recovery behavior.
- [ ] Add failed render cleanup.
- [ ] Add interrupted render recovery.
- [ ] Add memory leak checks for renderer.
- [ ] Add memory leak checks for media export.
- [ ] Add long-sample render tests.

### Diagnostics Quality

Tasks:

- [ ] Replace opaque fatal errors for common failures.
- [ ] Add actionable hints for import errors.
- [ ] Add actionable hints for media incompatibility.
- [ ] Add actionable hints for renderer failures.
- [ ] Add actionable hints for export failures.
- [ ] Add actionable hints for project failures.
- [ ] Add diagnostic documentation.
- [ ] Add redacted diagnostic export package.

### Consistency

Tasks:

- [ ] Add preview/export same-visible-comments tests.
- [ ] Add preview/export same-ordering tests.
- [ ] Add preview/export same-wrapping tests.
- [ ] Add preview/export same-truncation tests.
- [ ] Add preview/export same-row-height tests.
- [ ] Add preview/export same-panel-bounds tests.
- [ ] Add test proving export does not use native text fallback.
- [ ] Add tolerance documentation.

### Performance

Provisional targets:

- Import 100,000 comments within an acceptable interactive window.
- Query a 30-second visible comment window within frame-budget-relevant time.
- Render dense 30-second overlay sample without runaway memory growth.
- Avoid unbounded memory growth during simulated 2-hour render.
- Produce preview segment quickly enough for sync review.
- Keep project load/save acceptable with large comment stores.

Tasks:

- [ ] Define exact target numbers after Phase 0.
- [ ] Add importer benchmark.
- [ ] Add visible-window query benchmark.
- [ ] Add renderer dense benchmark.
- [ ] Add long render memory benchmark.
- [ ] Add preview latency benchmark.
- [ ] Add project load benchmark.
- [ ] Add project save benchmark.
- [ ] Add profiling notes.

### Packaging and Licensing

Tasks:

- [ ] Decide FFmpeg bundled vs external strategy.
- [ ] Record FFmpeg version.
- [ ] Review FFmpeg licensing implications.
- [ ] Review renderer dependency licenses.
- [ ] Review font licenses.
- [ ] Review emoji asset policy.
- [ ] Verify macOS app signing path.
- [ ] Verify notarization path.
- [ ] Verify app size implications.
- [ ] Add third-party notices.
- [ ] Add distribution limitations.

### Documentation and Samples

Tasks:

- [ ] Write user quickstart.
- [ ] Write canonical JSON format guide.
- [ ] Write supported media guide.
- [ ] Write diagnostics guide.
- [ ] Write known limitations.
- [ ] Add sample comment files.
- [ ] Add sample project.
- [ ] Add sample exported replay.
- [ ] Add developer architecture overview.
- [ ] Finalize renderer ADR.
- [ ] Add v1.0 release notes.

### v1.0 Release Criteria

v1.0 can ship only when:

- [ ] One local video plus one canonical JSON source can be imported.
- [ ] Default layout can be rendered.
- [ ] Preview and final export share overlay semantics.
- [ ] Project save/load is stable.
- [ ] Diagnostics are actionable.
- [ ] Renderer stack decision is recorded.
- [ ] Media path limitations are documented.
- [ ] No critical data-loss issue remains.
- [ ] No critical render-corruption issue remains.
- [ ] No critical preview/export consistency mismatch remains.
- [ ] Font and asset licensing are acceptable.
- [ ] FFmpeg distribution model is decided.

## v1.1 Detailed Plan: Import and Source Expansion

Goal: broaden input coverage without changing renderer fundamentals.

### CSV Importer

Tasks:

- [x] Define CSV importer ID.
- [x] Define CSV importer version.
- [x] Add column mapping UI.
- [x] Support timestamp column mapping.
- [x] Support author column mapping.
- [x] Support text column mapping.
- [x] Support kind column mapping if present.
- [x] Support platform/source metadata mapping if present.
- [x] Add encoding detection policy.
- [x] Add invalid encoding diagnostics.
- [x] Add skipped row diagnostics.
- [x] Add duplicate ID behavior.
- [x] Add import preview table.
- [x] Add fixture tests.

### JSONL Importer

Tasks:

- [x] Define JSONL importer ID.
- [x] Define JSONL importer version.
- [x] Implement streaming parser.
- [x] Handle partial line diagnostics.
- [x] Handle malformed row diagnostics.
- [x] Handle huge text field diagnostics.
- [x] Add large-file memory test.
- [x] Add row-level skip behavior.
- [x] Add fixture tests.

### Source Inspector

Tasks:

- [x] Expose multiple comment sources in project inspector.
- [x] Show source display name.
- [x] Show platform label.
- [x] Show importer ID.
- [x] Show importer version.
- [x] Show original file reference.
- [x] Show enabled toggle.
- [x] Show offset field.
- [x] Show timestamp basis.
- [x] Show import diagnostics.
- [x] Show source metadata.

### Merged Timeline Diagnostics

Tasks:

- [x] Show single merged timeline diagnostic view.
- [x] Show source-specific event counts.
- [x] Show skipped row counts.
- [x] Show offset-adjusted timestamp preview.
- [x] Show disabled source behavior.
- [x] Validate no renderer changes are required.

### v1.1 Release Gate

Ship only if:

- [x] v1.0 single-source workflow remains stable.
- [x] CSV importer passes fixture suite.
- [x] JSONL importer passes fixture suite.
- [x] Importers do not require renderer changes.
- [x] Multiple source timelines remain non-destructive.
- [x] Diagnostics explain source-specific import issues.

## v1.2 Detailed Plan: Multi-Source Sync and Merge

Goal: make simulcast and multi-platform workflows usable.

### Multi-Source UI

Tasks:

- [x] Allow adding multiple comment sources.
- [x] Allow removing a source without deleting original imported data until confirmed.
- [x] Allow enabling/disabling each source.
- [x] Allow editing display name.
- [x] Allow editing source platform label.
- [x] Allow editing per-source offset.
- [x] Show source diagnostics.
- [x] Show source event counts.

### Offset Review

Tasks:

- [x] Add manual offset review UI.
- [x] Add offset nudge controls.
- [x] Add keyboard shortcuts for nudge controls.
- [x] Add side-by-side source preview.
- [x] Add anchor-based manual sync.
- [x] Show effective timestamp.
- [x] Show original timestamp.
- [x] Show global offset.
- [x] Show per-source offset.
- [x] Make offset mistakes visible.

### Merge Model

Tasks:

- [x] Define stable merge ordering.
- [x] Define source priority policy.
- [x] Define event priority policy.
- [x] Define tie-breaking policy.
- [x] Define disabled source behavior.
- [x] Define platform label rendering model.
- [x] Add merged ordering tests.
- [x] Add non-destructive offset tests.
- [x] Add source priority tests.
- [x] Add disabled source tests.

### Automatic Alignment

Tasks:

- [x] Support timestamp-based alignment when reliable absolute timestamps exist.
- [x] Emit diagnostic when timestamp basis is ambiguous.
- [x] Do not add advanced automatic comment alignment unless reprioritized.
- [x] Show confidence and assumptions.
- [x] Require user review before applying automatic offset.

### Layouts

Tasks:

- [x] Add merged multi-platform layout.
- [x] Add split-platform review layout.
- [x] Add platform badge rendering.
- [x] Add source color tokens.
- [x] Add golden tests for merged layout.
- [x] Add golden tests for split layout.

### v1.2 Release Gate

Ship only if:

- [x] Source offset changes are non-destructive.
- [x] Merged ordering is stable and documented.
- [x] Preview/export consistency holds for merged timelines.
- [x] UI makes offset mistakes visible.
- [x] Merge diagnostics explain source-specific conflicts.

## v1.3 Detailed Plan: Layout and Styling

Goal: improve design flexibility without becoming a general-purpose editor.

### Built-In Templates

Candidate templates:

- Classic Sidebar refinement.
- Large Canvas Archive layout.
- Vertical Social layout.
- Research Transcript layout.
- Merged Multi-Platform Chat layout.
- Split Multi-Platform Review layout.

Tasks:

- [ ] Define template schema.
- [ ] Define template versioning.
- [ ] Define template validation diagnostics.
- [ ] Implement Classic Sidebar refinement.
- [ ] Implement Large Canvas Archive layout.
- [ ] Implement Vertical Social layout.
- [ ] Implement Research Transcript layout.
- [ ] Implement Merged Multi-Platform Chat layout.
- [ ] Implement Split Multi-Platform Review layout.
- [ ] Add golden tests per template.
- [ ] Add preview/export consistency tests per template.

### Style Tokens

Tasks:

- [ ] Define typography tokens.
- [ ] Define color tokens.
- [ ] Define spacing tokens.
- [ ] Define panel tokens.
- [ ] Define metadata tokens.
- [ ] Define badge tokens.
- [ ] Define pinned comment tokens.
- [ ] Define source color tokens.
- [ ] Add style token editor.
- [ ] Add style token preview.
- [ ] Add style token validation.
- [ ] Add style token migration.

### Metadata and Pinned Comments

Tasks:

- [ ] Define metadata region model.
- [ ] Define stream title display.
- [ ] Define streamer/channel display.
- [ ] Define platform label display.
- [ ] Define pinned comment display.
- [ ] Define donation/superchat-like event placeholder where model supports it.
- [ ] Add diagnostics for unsupported metadata.
- [ ] Add golden tests.

### User Templates

Tasks:

- [ ] Allow user template saving.
- [ ] Allow user template loading.
- [ ] Validate user template schema.
- [ ] Prevent arbitrary code execution.
- [ ] Add template migration.
- [ ] Add template diagnostics.
- [ ] Add safe failure behavior.

### v1.3 Release Gate

Ship only if:

- [ ] All templates compile to the same render-plan model.
- [ ] Renderer remains authoritative for text measurement.
- [ ] Template edits cannot execute arbitrary code.
- [ ] Preview/export overlay consistency remains enforced.
- [ ] Golden tests cover every shipped template.

## Future Track: Plugin Runner

Goal: support external import and data extensions without unsafe in-process plugins.

Principles:

- Do not support in-process binary plugins initially.
- Do not allow arbitrary template scripting without a dedicated security model.
- Prefer external executables with JSON-over-stdio.
- Require user-approved plugin directories.

Tasks:

- [ ] Draft plugin protocol.
- [ ] Define versioned request schema.
- [ ] Define versioned response schema.
- [ ] Define plugin manifest.
- [ ] Define importer plugin capability.
- [ ] Define metadata overlay capability.
- [ ] Define timeline event generation capability.
- [ ] Implement external executable runner.
- [ ] Implement JSON-over-stdio transport.
- [ ] Add timeout support.
- [ ] Add cancellation support.
- [ ] Add logging.
- [ ] Add diagnostics.
- [ ] Add example plugin.
- [ ] Add security review.
- [ ] Add plugin docs.

Gate:

- [ ] Importer and diagnostics core are already stable.
- [ ] Security model is reviewed.
- [ ] Plugin failures cannot corrupt project files.

## Future Track: Windows

Goal: keep Windows feasible without blocking macOS v1.0.

Early compatibility tasks:

- [ ] Build Rust core on Windows CI.
- [ ] Build overlay renderer on Windows CI.
- [ ] Validate project fixtures on Windows.
- [ ] Avoid macOS-only assumptions in schemas.
- [ ] Avoid platform-native object encodings.
- [ ] Add path normalization tests.

Later productization tasks:

- [ ] Evaluate WinUI 3 shell.
- [ ] Evaluate Windows media backend.
- [ ] Evaluate Media Foundation.
- [ ] Evaluate Direct3D compositing.
- [ ] Evaluate FFmpeg-centered Windows fallback.
- [ ] Integrate shared renderer.
- [ ] Validate project compatibility.
- [ ] Add Windows packaging plan.

Non-goal:

- Full-pipeline pixel-identical output across macOS and Windows is not required. Target schema compatibility, shared core behavior, and shared overlay behavior.

## Future Track: CLI and Automation

Goal: enable headless workflows, regression tests, and batch processing.

Tasks:

- [ ] Add CLI importer.
- [ ] Add CLI render-plan generator.
- [ ] Add CLI overlay frame generator.
- [ ] Add CLI media probe.
- [ ] Add CLI preview segment render.
- [ ] Add CLI full render.
- [ ] Add batch render command.
- [ ] Add render preset files.
- [ ] Add headless regression tests.
- [ ] Add machine-readable diagnostics output.
- [ ] Add reproducibility report generation.

Gate:

- [ ] v1.0 render path is stable.
- [ ] Project schema is stable.
- [ ] Diagnostics are stable.

## Future Track: Platform Import

Goal: support common archive formats without direct platform API retrieval as an assumed requirement.

Tasks:

- [ ] Research YouTube archive chat import formats.
- [ ] Research Twitch archive chat import formats.
- [ ] Research third-party chat downloader output formats.
- [ ] Research event platform export formats.
- [ ] Add dataset mapping docs.
- [ ] Add fixture corpus.
- [ ] Add importer prototypes.
- [ ] Add diagnostics for missing metadata.

Explicitly deferred:

- Direct YouTube API import.
- Direct Twitch API import.
- Automatic VOD download.

## Future Track: Research and Archive Exports

Goal: improve archival and research usefulness.

Tasks:

- [ ] Export normalized comment timeline.
- [ ] Export source provenance report.
- [ ] Export render reproducibility report.
- [ ] Export chapter file.
- [ ] Export subtitle-like file.
- [ ] Export comment index.
- [ ] Export redacted diagnostics package.
- [ ] Add privacy redaction options.
- [ ] Add documentation for citations and reproducibility.

## Testing Plan

### Rust Core Tests

Required:

- [ ] Timestamp parsing: milliseconds.
- [ ] Timestamp parsing: seconds.
- [ ] Timestamp parsing: `HH:MM:SS`.
- [ ] Timestamp parsing: `MM:SS`.
- [ ] Timestamp parsing: invalid formats.
- [ ] Import normalization: canonical JSON happy path.
- [ ] Import normalization: missing fields.
- [ ] Import normalization: duplicate IDs.
- [ ] Diagnostics: warning severity.
- [ ] Diagnostics: error severity.
- [ ] Diagnostics: fatal severity.
- [ ] Diagnostics: actionable hints.
- [ ] Sorting: timestamp.
- [ ] Sorting: event priority.
- [ ] Sorting: source priority.
- [ ] Sorting: import order.
- [ ] Sorting: stable ID.
- [ ] Offsets: global offset.
- [ ] Offsets: source offset.
- [ ] Offsets: negative results.
- [ ] Offsets: out-of-range results.
- [ ] Visibility: comment window.
- [ ] Visibility: max visible comments.
- [ ] Visibility: disabled source behavior.
- [ ] Serialization: project load.
- [ ] Serialization: project save.
- [ ] Serialization: schema version.
- [ ] Serialization: migration fixtures.
- [ ] Source model: source IDs.
- [ ] Source model: platform labels.
- [ ] Source model: importer metadata.

### Importer Robustness Tests

v1.0 canonical JSON:

- [ ] Valid minimal file.
- [ ] Valid multiline file.
- [ ] Valid CJK/emoji file.
- [ ] Missing timestamp.
- [ ] Invalid timestamp.
- [ ] Duplicate ID.
- [ ] Unknown kind.
- [ ] Empty text.
- [ ] Comments outside video duration.
- [ ] Malformed JSON.
- [ ] Huge text field.

v1.1 CSV/JSONL:

- [ ] Column mapping.
- [ ] Invalid encoding.
- [ ] Partial line.
- [ ] Streaming large file.
- [ ] Row-level skip diagnostics.

### Shared Overlay Renderer Tests

Required:

- [ ] Font resolution: default font.
- [ ] Font resolution: missing font.
- [ ] Font resolution: fallback font.
- [ ] Unicode: grapheme boundaries.
- [ ] Unicode: combining marks.
- [ ] Unicode: ZWJ emoji.
- [ ] CJK: Japanese line break.
- [ ] CJK: mixed Latin/CJK.
- [ ] CJK: punctuation handling.
- [ ] Emoji: color emoji.
- [ ] Emoji: variation selector.
- [ ] Emoji: skin tone.
- [ ] Emoji: unresolved glyph.
- [ ] Paragraph: max width.
- [ ] Paragraph: max lines.
- [ ] Paragraph: line height.
- [ ] Paragraph: baseline.
- [ ] Comment rows: author plus body.
- [ ] Comment rows: long author.
- [ ] Comment rows: multiline body.
- [ ] Badges/emotes: inline placement.
- [ ] Badges/emotes: missing asset placeholder.
- [ ] Overflow: truncation.
- [ ] Overflow: ellipsis.
- [ ] Overflow: dense panel.
- [ ] Raster: transparency.
- [ ] Raster: premultiplied alpha.
- [ ] Raster: panel backgrounds.
- [ ] Cache: cache key invalidation.
- [ ] Cache: renderer version change.
- [ ] Golden: stable image output.

### Media Tests

Required:

- [ ] Probing: MP4.
- [ ] Probing: MOV.
- [ ] Probing: unsupported container.
- [ ] Probing: no audio.
- [ ] Metadata: resolution.
- [ ] Metadata: FPS.
- [ ] Metadata: duration.
- [ ] Metadata: codec.
- [ ] Metadata: timebase.
- [ ] Metadata: rotation.
- [ ] Placement: exact integer source region.
- [ ] Placement: no unintended scaling.
- [ ] Export: preview segment.
- [ ] Export: final export.
- [ ] Export: cancellation.
- [ ] Audio: AAC passthrough attempt.
- [ ] Audio: AAC fallback.
- [ ] Audio: no audio.
- [ ] Audio: multi-track diagnostic.
- [ ] Color: metadata preservation or warning.
- [ ] Color: SDR/HDR diagnostic.
- [ ] VFR: detection and deterministic policy diagnostic.

### Preview/Export Consistency Tests

Required comparisons:

- [ ] Same visible comments at same timestamp.
- [ ] Same comment ordering.
- [ ] Same wrapping.
- [ ] Same truncation.
- [ ] Same row height.
- [ ] Same pinned/metadata placement.
- [ ] Same panel bounds.
- [ ] No native text fallback in export.

Acceptable differences:

- Bitrate artifacts.
- Encoder differences.
- Preview duration.
- Execution speed.
- Non-overlay video compression differences.

### UI Tests

v1.0:

- [ ] Create project.
- [ ] Select video.
- [ ] Import canonical JSON.
- [ ] Show diagnostics.
- [ ] Adjust global offset.
- [ ] Preview segment.
- [ ] Export final.
- [ ] Cancel render.
- [ ] Save project.
- [ ] Open project.
- [ ] Recover missing source media.

v1.x:

- [ ] Multiple source import.
- [ ] Per-source enable/disable.
- [ ] Offset review.
- [ ] Template selection.
- [ ] Style token editing.

### Security and Privacy Tests

Required:

- [ ] Project does not leak full local paths in exported diagnostics by default.
- [ ] Plugin execution is absent in v1.0.
- [ ] Sandbox file access model is documented.
- [ ] External file references recover gracefully.
- [ ] User-selected fonts/assets are referenced safely.
- [ ] Diagnostic export supports redaction.

## CI Plan

Phase 0 CI:

- [ ] Rust format check.
- [ ] Rust unit tests.
- [ ] Importer fixture tests.
- [ ] Render-plan serialization tests.
- [ ] Renderer harness smoke test if dependencies are available.
- [ ] Media probe tests with small fixture if legally storable.

v1.0 alpha CI:

- [ ] Rust unit tests.
- [ ] Importer tests.
- [ ] Renderer golden tests.
- [ ] Schema validation tests.
- [ ] Swift bridge tests.
- [ ] macOS app build.
- [ ] CLI renderer harness.

v1.0 beta CI:

- [ ] Preview/export consistency tests.
- [ ] Media export smoke tests.
- [ ] Project migration tests.
- [ ] Long sample stress job if CI resources allow.
- [ ] Signing/notarization dry run if available.

Future CI:

- [ ] Windows Rust core build.
- [ ] Windows overlay renderer build.
- [ ] Cross-platform project fixture validation.
- [ ] Plugin protocol tests.
- [ ] Batch render regression tests.

## Risk Management Plan

### R1 Shared Renderer Complexity

Mitigation tasks:

- [ ] Validate in Phase 0.
- [ ] Use proven libraries.
- [ ] Scope renderer to overlays.
- [ ] Build CLI harness.
- [ ] Add golden tests early.

Gate:

- [ ] Continue only if CJK, emoji, multiline rendering, and preview/export consistency are viable.

### R2 Renderer Performance

Mitigation tasks:

- [ ] Cache shaped runs.
- [ ] Cache paragraph measurements.
- [ ] Cache static row fragments.
- [ ] Cache badges and emotes.
- [ ] Render only visible comments.
- [ ] Profile dense fixtures early.

Gate:

- [ ] Dense 30-second fixture renders acceptably before v1.0 hardening.

### R3 Font and Emoji Fidelity

Mitigation tasks:

- [ ] Define default font policy.
- [ ] Record font versions.
- [ ] Diagnose missing glyphs.
- [ ] Use bundled open-source fonts where feasible.
- [ ] Review licenses.

Gate:

- [ ] Font and asset licensing review before public release.

### R4 Media Backend Feasibility

Mitigation tasks:

- [ ] Build full segment export in Phase 0.
- [ ] Keep media backend replaceable.
- [ ] Separate overlay rendering from media execution.

Gate:

- [ ] 10 to 60 second export succeeds with diagnostics before broader app work.

### R5 Source Video Quality Degradation

Mitigation tasks:

- [ ] Preserve native source region where possible.
- [ ] Avoid fractional scaling.
- [ ] Add frame comparison tests.
- [ ] Diagnose color/HDR/VFR.

Gate:

- [ ] Default layout passes visual and automated placement checks.

### R6 FFmpeg Packaging and Licensing

Mitigation tasks:

- [ ] Decide bundled vs external early.
- [ ] Isolate behind `ReplayMedia`.
- [ ] Document version.
- [ ] Review licenses.

Gate:

- [ ] Distribution model decision before v1.0 beta.

### R7 Swift/Rust Interop Complexity

Mitigation tasks:

- [ ] Keep API coarse.
- [ ] Use generated bindings or disciplined C ABI.
- [ ] Test ownership.
- [ ] Test errors.
- [ ] Test large payloads.

Gate:

- [ ] Bridge smoke and large payload tests pass before UI expansion.

### R8 Scope Creep

Mitigation tasks:

- [ ] Enforce release partition.
- [ ] Keep multi-source in model, not v1.0 UI.
- [ ] Move broad features to v1.x.
- [ ] Reject plugin runner and full multi-source UX from v1.0.

Gate:

- [ ] v1.0 backlog contains no plugin runner or full multi-source UX.

### R9 Project File Instability

Mitigation tasks:

- [ ] Version schemas from the start.
- [ ] Use fixtures.
- [ ] Add migration tests.
- [ ] Add validation command.

Gate:

- [ ] Project load/save fixtures pass before v1.0 alpha.

### R10 Diagnostics Weakness

Mitigation tasks:

- [ ] Use structured diagnostics.
- [ ] Include severity, category, source reference, code, and actionable hint.
- [ ] Avoid opaque fatal errors.

Gate:

- [ ] v1.0 cannot ship with opaque fatal errors for common failure classes.

## Decision Log Requirements

Create or update decision records for:

- [ ] Bridge strategy.
- [ ] Renderer stack.
- [ ] Font policy.
- [ ] Emoji policy.
- [ ] FFmpeg distribution model.
- [ ] Apple-native render path.
- [ ] Audio passthrough policy.
- [ ] Project package format.
- [ ] SQLite requirement.
- [ ] Template schema.
- [ ] Plugin protocol before implementation.

Each decision record must include:

- Status.
- Date.
- Context.
- Decision.
- Alternatives considered.
- Consequences.
- Evidence.
- Revisit triggers.

## Documentation Plan

Developer docs:

- [ ] Architecture overview.
- [ ] Module ownership.
- [ ] Build setup.
- [ ] Bridge setup.
- [ ] Fixture generation.
- [ ] Renderer golden workflow.
- [ ] Media fixture policy.
- [ ] Diagnostics code catalog.
- [ ] Schema reference.

User docs:

- [ ] Quickstart.
- [ ] Canonical JSON guide.
- [ ] Supported media guide.
- [ ] Diagnostics guide.
- [ ] Preview/export consistency explanation.
- [ ] Known limitations.
- [ ] Troubleshooting guide.

Release docs:

- [ ] v1.0 release notes.
- [ ] v1.0 limitations.
- [ ] License notices.
- [ ] FFmpeg notice.
- [ ] Renderer/font notices.
- [ ] Sample files.

## v1.0 Final Launch Checklist

Product:

- [ ] New project works.
- [ ] Open project works.
- [ ] Save project works.
- [ ] Select video works.
- [ ] Probe media works.
- [ ] Import canonical JSON works.
- [ ] Diagnostics panel works.
- [ ] Global offset works.
- [ ] Default sidebar layout works.
- [ ] Preview segment render works.
- [ ] Full export works.
- [ ] Progress works.
- [ ] Cancellation works.
- [ ] Render history works.
- [ ] Reproducibility manifest is written.

Quality:

- [ ] No critical data loss bugs.
- [ ] No critical render corruption bugs.
- [ ] No critical preview/export consistency bugs.
- [ ] No known unbounded memory growth in supported workflows.
- [ ] Dense 30-second fixture passes.
- [ ] Long sample render passes.
- [ ] Golden image tests pass.
- [ ] Media placement tests pass.
- [ ] Importer fixture tests pass.
- [ ] Project migration tests pass.

Distribution:

- [ ] FFmpeg strategy decided.
- [ ] License notices complete.
- [ ] App signing path verified.
- [ ] Notarization path verified.
- [ ] Sample project included or linked.
- [ ] Known limitations documented.

## Explicit Non-Goals Until Reprioritized

- Direct YouTube, Twitch, or platform API import in v1.0.
- Automatic VOD download in v1.0.
- Cloud rendering.
- Collaboration.
- Windows product release in v1.0.
- Mobile apps.
- Full nonlinear video editing.
- Arbitrary template scripting.
- Third-party in-process binary plugins.
- Advanced automatic comment alignment.
- Fully platform-faithful chat recreation.
- Mathematically lossless output guarantees.
- Real-time livestream capture.
- Arbitrary motion graphics scripting.

## Working Status Legend

Use these markers when updating this plan:

- `[ ]` Not started.
- `[~]` In progress.
- `[x]` Complete.
- `[!]` Blocked.
- `[?]` Needs decision.

When changing a status, add a short dated note near the relevant section if the decision or result affects architecture, release scope, risk, or future work.
