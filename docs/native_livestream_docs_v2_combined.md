# Native Livestream Archive Replay Composer — Documentation Set v2 Combined



---

<!-- Source file: 00_decision_summary.md -->

# 00. Decision Summary

## Final Direction

Proceed with the product, but split the original proposal into release-gated documents.

The revised strategy is:

1. Keep the **shared cross-platform overlay renderer** as a v1.0 architectural requirement.
2. Narrow v1.0 to **single-source replay composition with production-grade rendering consistency**.
3. Keep the data model and render plan ready for multi-source workflows from day one.
4. Move multi-source UX, advanced styling, plugin execution, and Windows productization into v1.x or future tracks.
5. Treat Phase 0 as a hard technical validation gate, not as a demo.

## Accepted Architectural Commitments

### Shared overlay renderer remains core

The shared overlay renderer is not optional polish. It is the component responsible for the product’s defining visual layer: synchronized comment and metadata overlays. It must own text shaping, Unicode segmentation, CJK-aware line breaking, emoji policy, paragraph layout, truncation, row measurement, badge/emote placement, and rasterization.

### Rust core remains the portable semantic layer

The Rust core owns normalized comments, source metadata, project schema, timeline logic, validation, diagnostics, sync offsets, render-plan generation, and storage-facing data structures. It must not contain SwiftUI, AppKit, AVFoundation, VideoToolbox, WinUI, DirectWrite, Direct2D, or platform-native object types.

### macOS remains first product target

The initial product is a native macOS application using Swift, SwiftUI, and AppKit. The UI must feel native, but authoritative project, timeline, import, layout, and render-plan logic must stay out of UI code.

### FFmpeg remains required for media compatibility

FFmpeg or ffprobe is required for robust probing and fallback compatibility. Apple-native media APIs are the preferred fast path when source media is compatible, but the application must not assume all local archives are AVFoundation-friendly.

## Major Scope Correction

The original proposal had a valid but broad public v1. The revised plan separates it into:

### v1.0 Core Quality Gate

A credible, production-grade single-source replay composer:

- one local video,
- one canonical JSON comment source,
- one fixed or lightly configurable sidebar layout,
- Rust core,
- shared overlay renderer,
- project save/load,
- FFmpeg probing,
- Apple-native compatible export path,
- preview segment export,
- final export,
- diagnostics,
- preview/export overlay consistency tests,
- renderer decision record.

### v1.x Product Expansion

Features that are important but should not block v1.0:

- CSV importer,
- JSONL importer,
- multiple comment sources,
- per-source offsets in UI,
- manual sync review UI,
- multi-platform merge UI,
- additional templates,
- richer style controls,
- archive preset refinements.

### Future Tracks

Capabilities that should be designed for but not implemented as v1.0 release blockers:

- external plugin runner,
- advanced automatic sync heuristics,
- batch processing,
- CLI automation,
- Windows product release,
- interactive web replay,
- platform API retrieval,
- automatic VOD download,
- full nonlinear editing.

## Non-Negotiable Invariants

- Normalized comments are immutable after import.
- Source offsets are non-destructive.
- Render plans are serializable and platform-neutral.
- Preview and export use the same semantic overlay model.
- Renderer owns all authoritative text measurement.
- UI never performs authoritative timeline or renderer layout computation.
- Project files cannot store Apple-specific object encodings.
- Media backend decisions are recorded in diagnostics.
- Font policy, renderer version, and template version are reproducibility inputs.

## Practical Definition of Success

The project is succeeding if Phase 0 proves that a macOS prototype can export a 10–60 second replay video where:

- source video is placed at exact integer coordinates,
- comments are rendered by the shared renderer,
- CJK, emoji, and multiline text do not clip,
- preview and export overlay behavior match,
- media path and renderer path are diagnosable,
- performance is acceptable enough to continue,
- packaging and licensing assumptions remain viable.


---

<!-- Source file: 01_product_proposal_v2.md -->

# 01. Product Proposal v2

## 1. Executive Summary

Native Livestream Archive Replay Composer is a macOS-first native application for transforming local livestream archive videos and prepared timestamped comment files into synchronized replay videos.

The product restores the experience of watching a livestream together with its live comments, platform metadata, pinned messages, contextual labels, and replay-oriented overlays. It is not a general-purpose video editor and should not become one.

The first credible release should focus on producing a high-quality single-source replay from:

- one local archived video,
- one prepared timestamped comment file in canonical JSON,
- optional metadata,
- a controlled layout template,
- deterministic render settings.

The application should then synchronize comments with the video timeline, compose the source video into a larger canvas, render a comment panel using a shared overlay renderer, and export a replay video suitable for viewing, sharing, research, or preservation.

## 2. Core Product Thesis

Livestream archives are incomplete when they preserve only video and lose the synchronized social context around that video. Comments, reactions, pinned messages, donation events, platform badges, and audience timing often explain why moments in the video mattered.

A focused replay composer can solve this better than generic video editors, screen recording workflows, or platform-specific tools because it treats comments as timeline events rather than static visual assets.

## 3. Primary Users

### v1.0 Primary Users

v1.0 should target technically capable users who already have a local video and a prepared timestamped comment file.

Typical examples:

- archivists,
- researchers,
- technically capable creators,
- fan archive maintainers,
- event documentation operators.

These users will tolerate a controlled workflow if the output is reliable, deterministic, and high quality.

### v1.x Expansion Users

After v1.0, the product can broaden toward:

- streamers,
- video editors,
- multi-platform stream operators,
- educators,
- less technical creators.

This expansion requires better import ergonomics, more templates, richer styling, and multi-source sync UI.

## 4. Product Goals

### 4.1 v1.0 Goals

1. Create a high-quality replay video from one local video and one comment file.
2. Preserve source video clarity by avoiding unnecessary scaling and awkward canvas decisions.
3. Use a shared overlay renderer for all final comment and metadata overlay rendering.
4. Ensure preview/export overlay consistency.
5. Store projects in deterministic, portable, versioned formats.
6. Use Rust for core project, import, timeline, validation, and render-plan logic.
7. Use Swift/SwiftUI/AppKit for a native macOS document application.
8. Use FFmpeg/ffprobe for media probing and compatibility diagnostics.
9. Provide actionable diagnostics for import, media, renderer, and export failures.
10. Leave the internal model ready for multi-source merge without exposing full multi-source UX in v1.0.

### 4.2 v1.x Goals

1. Add CSV and JSONL importers.
2. Add multiple comment sources per project.
3. Expose per-source sync offsets.
4. Add manual multi-source sync review UI.
5. Add merged multi-platform comment rendering.
6. Add additional templates and style controls.
7. Improve archive and research export artifacts.
8. Add plugin protocol prototypes after importer and diagnostics stability.

## 5. v1.0 Non-Goals

v1.0 should explicitly exclude:

- direct YouTube, Twitch, or platform API import,
- automatic VOD download,
- cloud rendering,
- collaboration,
- Windows product release,
- mobile apps,
- full nonlinear video editing,
- arbitrary template scripting,
- third-party in-process binary plugins,
- advanced automatic comment alignment,
- fully platform-faithful chat recreation,
- mathematically lossless output guarantees.

## 6. v1.0 User Journey

1. User creates a new replay project.
2. User selects a local video file.
3. App probes the media file and reports compatibility.
4. User imports a canonical JSON comment file.
5. App normalizes comments and reports import diagnostics.
6. User chooses the default sidebar replay layout.
7. User adjusts a global sync offset if needed.
8. User previews a short segment.
9. User exports a final replay video.
10. App stores render diagnostics and project metadata.

## 7. Output Quality Principles

The product should not claim that output is mathematically lossless. The realistic quality target is:

- avoid unnecessary scaling,
- preserve source region at native size when layout allows,
- avoid fractional scaling,
- avoid avoidable color conversions,
- choose high-quality encoder defaults,
- keep overlay text crisp,
- make media decisions visible in diagnostics,
- make preview decisions trustworthy for export.

## 8. Product Differentiators

The product should differentiate through:

- comment-aware timeline model,
- high-quality shared overlay rendering,
- CJK and emoji robustness,
- deterministic project files,
- preview/export consistency,
- local-first workflows,
- import diagnostics,
- future multi-platform merge readiness,
- future Windows compatibility through shared core and renderer.

## 9. Long-Term End State

The long-term product can become a professional-quality livestream replay composition environment that supports multiple archive formats, multiple comment sources, custom layouts, importer plugins, batch processing, CLI workflows, and native macOS/Windows applications.

The route to that end state should pass through a narrow but strong v1.0 rather than a broad and fragile first release.


---

<!-- Source file: 02_scope_partition_release_plan.md -->

# 02. Scope Partition and Release Plan

## 1. Scope Principle

The project accepts architectural heaviness where it protects core product quality. It does not accept broad feature scope that delays validation of the core architecture.

The correct split is:

- Heavy renderer and media architecture: acceptable in v1.0.
- Broad import, merge, styling, plugin, and Windows scope: separated into v1.x and future tracks.

## 2. v1.0 Release Blockers

v1.0 is blocked only by features required to deliver a credible single-source replay composer.

### Product Surface

- Native macOS document app shell.
- Local video import.
- Canonical JSON comment import.
- Single comment source per project in UI.
- Global comment offset.
- One default sidebar layout.
- Short preview segment render.
- Full export render.
- Project save/load.
- Basic settings.
- Render progress and cancellation.

### Core Architecture

- Rust core crate.
- Swift/Rust bridge.
- Normalized comment model with source ID from day one.
- Project schema.
- Import diagnostics schema.
- Render plan schema.
- SemanticRenderPlan and RenderExecutionProfile separation.
- Project package format.
- SQLite or equivalent comment store prototype if needed for scale.

### Shared Overlay Renderer

- Controlled font policy.
- Text shaping proof carried into production path.
- Unicode segmentation and CJK line breaking.
- Emoji handling policy.
- Paragraph layout.
- Comment row measurement.
- Comment panel layout.
- Transparent RGBA overlay output.
- Golden image tests.
- Preview/export shared rendering path.

### Media and Export

- ffprobe media probing.
- Apple-native fast path for compatible MP4/MOV H.264/HEVC + AAC sources.
- Fallback diagnostic path for unsupported sources.
- Source video placement into larger canvas.
- Overlay compositing.
- AAC passthrough attempt where safe, otherwise AAC re-encode.
- High-quality upload preset.
- Preview preset.
- Export diagnostics.

### Validation

- Phase 0 acceptance tests passed.
- Renderer decision record completed.
- Preview/export overlay consistency test passed.
- Import and render diagnostics usable.
- Long enough sample render proves no immediate memory growth issue.

## 3. v1.0 Explicit Exclusions

These features are designed for, but not delivered as v1.0 blockers:

- CSV importer.
- JSONL importer.
- Multi-source import UI.
- Multi-platform merged comment UI.
- Manual multi-source offset review.
- Multiple built-in layouts beyond the default.
- Deep style editor.
- Plugin runner.
- Windows prototype UI.
- Batch processing.
- Platform API import.

## 4. v1.1 Target: Import and Source Expansion

v1.1 should broaden input coverage without changing renderer fundamentals.

### Deliverables

- CSV importer with column mapping.
- JSONL importer for large files.
- Import preview table.
- Better import diagnostics UI.
- Multiple comment source data model exposed in project inspector.
- Per-source enable/disable.
- Per-source offset fields.
- Single merged timeline view in diagnostics or preview.

### Release Gate

v1.1 ships only if:

- single-source v1.0 remains stable,
- importers do not require renderer changes,
- multiple source timelines are non-destructive,
- diagnostics explain source-specific import issues.

## 5. v1.2 Target: Multi-Source Sync and Merge UX

v1.2 should make simulcast and multi-platform workflows usable.

### Deliverables

- Manual offset review UI.
- Side-by-side source preview.
- Merged timeline preview.
- Source-level platform labels.
- Merged multi-platform layout.
- Split-platform review layout.
- Merge diagnostics.
- Timestamp-based automatic alignment where reliable metadata exists.

### Release Gate

v1.2 ships only if:

- source offset changes are non-destructive,
- merged ordering is stable and documented,
- preview/export consistency holds for merged timelines,
- UI makes offset mistakes visible.

## 6. v1.3 Target: Layout and Styling Expansion

v1.3 should improve design flexibility without becoming a general-purpose editor.

### Deliverables

- Additional layout templates.
- Style token controls.
- Metadata region configuration.
- Pinned comment display controls.
- Platform badge styling.
- User template saving.
- Template validation diagnostics.

### Release Gate

v1.3 ships only if:

- all templates compile to the same render-plan model,
- renderer remains authoritative for text measurement,
- template edits cannot execute arbitrary code,
- preview/export overlay consistency remains enforced.

## 7. v1.x / Future Tracks

### Plugin Track

- External importer plugin protocol.
- JSON-over-stdio runner.
- Plugin discovery from user-approved directories.
- Timeouts and cancellation.
- Example plugin.
- Security review.

### Windows Track

- Rust core builds on Windows CI.
- Shared overlay renderer builds on Windows CI.
- Project files validate cross-platform.
- Windows media backend prototype.
- WinUI shell prototype after core renderer stability.

### Automation Track

- CLI render harness.
- Batch processing.
- Render-plan fixtures.
- Headless validation tests.

## 8. Release Naming

Recommended release sequence:

- **Phase 0:** Technical Validation
- **v0.1:** Internal vertical slice
- **v0.2:** Internal production path hardening
- **v1.0:** Single-source production replay composer
- **v1.1:** Import expansion
- **v1.2:** Multi-source sync and merge
- **v1.3:** Layout and styling expansion
- **v1.x:** Plugins, CLI, Windows feasibility


---

<!-- Source file: 03_architecture_overview.md -->

# 03. Architecture Overview

## 1. System Overview

The product is a macOS-first native application with a portable Rust core, a shared overlay renderer, and platform media backends.

```text
macOS Application
  Swift / SwiftUI / AppKit
        ↓
ReplayBridge
  Swift/Rust interop
        ↓
ReplayCore
  project, import, timeline, validation, render-plan logic
        ↓
ReplayOverlay
  shared text/layout/raster overlay renderer
        ↓
ReplayMedia
  probing, compatibility, media preparation
        ↓
ReplayRenderApple / ReplayRenderFFmpeg
  composition, encoding, muxing
        ↓
Output Video
```

## 2. Module Responsibilities

### ReplayMacApp

Owns native macOS UI behavior:

- document windows,
- menus,
- file dialogs,
- drag-and-drop,
- inspectors,
- preview controls,
- timeline scrubber,
- settings,
- progress and cancellation UI,
- diagnostics presentation.

It does not own authoritative import, merge, layout, render-plan, or text measurement logic.

### ReplayBridge

Provides a narrow Swift/Rust boundary.

Preferred API style:

- coarse-grained commands,
- structured diagnostics,
- opaque handles where needed,
- JSON fixtures for debugging,
- no high-frequency tiny calls during per-frame rendering unless measured safe.

Candidate binding strategy:

- UniFFI for ergonomic Swift bindings, or
- disciplined C ABI for stricter control.

### ReplayCore

Owns platform-neutral semantics:

- project schema,
- normalized comment model,
- comment source model,
- timestamp normalization,
- import registry,
- diagnostics,
- timeline sorting,
- source offsets,
- comment visibility windows,
- merge model,
- layout region planning,
- render-plan generation,
- project serialization,
- schema migration,
- reproducibility metadata.

ReplayCore must not store platform-native types.

### ReplayOverlay

Owns authoritative overlay visual behavior:

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
- badge/emote inline placement,
- panel layout,
- pinned/metadata overlay layout,
- RGBA rasterization,
- golden image output,
- layout diagnostics.

### ReplayMedia

Owns media probing and compatibility decisions:

- ffprobe integration,
- media metadata normalization,
- compatibility decision tree,
- proxy/intermediate decisions,
- remux/transcode decisions,
- audio policy,
- media diagnostics.

### ReplayRenderApple

Owns macOS render execution for compatible paths:

- source frame reading,
- source frame placement,
- overlay frame compositing,
- AVFoundation/VideoToolbox writing,
- progress,
- cancellation,
- audio handling.

### ReplayRenderFFmpeg

Owns fallback or assisted workflows:

- proxy generation,
- intermediate generation,
- remuxing,
- audio conversion,
- final mux assistance where required.

### ReplayStorage

Owns project package and caches:

- `project.json`,
- `comments.sqlite`,
- source metadata,
- assets,
- templates,
- overlay cache,
- media proxies,
- diagnostics,
- render history.

## 3. Critical Boundary: Core vs Renderer

The most important architectural boundary is between ReplayCore and ReplayOverlay.

### ReplayCore owns

- which comments are visible at a given time,
- source offset application,
- merged ordering rules,
- logical canvas regions,
- style tokens,
- render-plan generation.

### ReplayOverlay owns

- measuring text,
- wrapping text,
- calculating row heights,
- resolving fonts,
- placing glyphs,
- handling emoji and CJK breaks,
- drawing final overlay pixels.

Core may define layout regions. Renderer must own text-dependent layout.

## 4. Render Plan Split

Use two related structures.

### SemanticRenderPlan

Contains deterministic rendering semantics:

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

### RenderExecutionProfile

Contains execution-specific choices:

- preview or final export,
- frame range,
- output scale,
- bitrate,
- encoder,
- cache policy,
- temporary file policy,
- progress reporting mode.

Preview and export may differ in execution profile. They must not differ in semantic overlay behavior.

## 5. Project Package

Recommended package structure:

```text
ProjectName.replayproj/
  project.json
  comments.sqlite
  sources/
    source-main.json
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
    preview-renders/
    overlay-cache/
    media-proxies/
    intermediates/
  diagnostics.json
  render-history.json
  reproducibility.json
```

## 6. Architecture Invariants

- No SwiftUI/AppKit type in ReplayCore.
- No AVFoundation/VideoToolbox type in ReplayCore.
- No DirectWrite/Direct2D/WinUI type in ReplayCore.
- No platform-native font object in project files.
- Renderer owns text measurement.
- Project schema is versioned and language-neutral.
- Render plans are serializable.
- Importer outputs normalized comments.
- Source offsets never rewrite imported timestamps.
- Preview and export cannot use different overlay layout engines.
- Media backend decisions are recorded in diagnostics.

## 7. Suggested Repository Structure

```text
replay-composer/
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
  docs/
  tools/
```

## 8. Build and CI Requirements

CI should eventually include:

- Rust unit tests,
- importer tests,
- renderer golden tests,
- schema validation tests,
- Swift bridge tests,
- macOS app build,
- CLI renderer harness,
- Windows build check for Rust core and overlay renderer.


---

<!-- Source file: 04_renderer_adr.md -->

# 04. Architecture Decision Record: Shared Overlay Renderer

## ADR-001: Use a Shared Cross-Platform Overlay Renderer from v1.0

**Status:** Proposed / to be validated in Phase 0  
**Date:** 2026-05-04

## 1. Context

The application’s defining feature is not basic video export. It is the reconstruction of livestream context through synchronized comments, pinned messages, platform labels, badges, emoji, CJK text, and metadata overlays.

If preview uses platform-native text rendering and export uses a different rendering path, users may see wrapping, clipping, baseline, emoji, or truncation differences between preview and final output. If macOS and Windows later use different text stacks, the same project may produce visibly different overlays.

Therefore overlay rendering must be treated as a core product component, not incidental UI drawing.

## 2. Decision

The product will use a shared overlay renderer from v1.0 for all authoritative comment and metadata overlay rendering.

The renderer will own:

- font resolution policy,
- fallback policy,
- Unicode segmentation,
- grapheme boundary handling,
- CJK line breaking,
- text shaping,
- paragraph layout,
- wrapping,
- truncation,
- ellipsis,
- baseline and line height calculation,
- comment row measurement,
- badge and emote placement,
- panel layout,
- metadata overlay layout,
- transparent RGBA rasterization,
- golden-test output.

The media backend will own:

- video decode,
- source video placement,
- alpha compositing,
- audio handling,
- encoding,
- muxing,
- progress and cancellation.

## 3. Alternatives Considered

### Option A: Platform-native text rendering only

Use SwiftUI/AppKit/CoreText/CoreAnimation for macOS preview and export.

#### Pros

- Fastest initial implementation.
- Native platform typography.
- Less external dependency complexity.

#### Cons

- Preview/export mismatch risk.
- Windows port requires a different text system.
- CJK/emoji/fallback behavior may vary.
- Golden tests are harder to stabilize.
- Renderer behavior becomes tied to macOS frameworks.

### Option B: Core Animation overlay during export

Use `AVVideoCompositionCoreAnimationTool` with CALayers and attributed strings.

#### Pros

- Good fit for some AVFoundation workflows.
- Relatively easy for simple overlays.

#### Cons

- Text layout still platform-dependent.
- Not portable to Windows.
- Complex dense chat and deterministic layout remain risky.
- Renderer quality is not fully controlled.

### Option C: Shared overlay renderer + native media backend

Use a shared renderer for text/layout/raster overlays while native or FFmpeg-assisted media backend handles video/audio.

#### Pros

- Controls core visual feature.
- Better preview/export consistency.
- More portable to Windows.
- Enables golden image testing.
- Keeps media backend replaceable.

#### Cons

- Higher implementation complexity.
- Dependency packaging/licensing risk.
- Performance must be validated.
- Font and emoji policy must be explicit.

### Option D: Full FFmpeg-centered renderer

Use FFmpeg filtergraphs and generated overlay frames for everything.

#### Pros

- Cross-platform media pipeline.
- Broad format support.
- Fewer AVFoundation constraints.

#### Cons

- Native preview integration can be clunkier.
- Text/layout quality still needs a renderer.
- Harder to provide native-feeling interactive preview.
- FFmpeg packaging/licensing complexity becomes central.

## 4. Selected Direction

Select Option C:

> Shared overlay renderer plus native/FFmpeg-assisted media backend.

This gives the product control over its most important visual layer while allowing pragmatic media backend choices.

## 5. Renderer Stack Candidates

The exact stack must be decided in Phase 0.

Candidate components:

- HarfBuzz or equivalent for shaping.
- ICU or equivalent for Unicode segmentation and line breaking.
- Skia / SkParagraph or equivalent for 2D/paragraph rendering.
- Controlled bundled font stack.
- Optional CPU raster path first.
- Optional GPU acceleration after performance validation.

## 6. Phase 0 Evaluation Criteria

A renderer stack remains viable only if it passes:

### Text Quality

- CJK wrapping works.
- Emoji sequences do not split incorrectly.
- Multiline comments do not clip.
- Long author names truncate predictably.
- Badge/emote inline placement is stable.
- Missing glyphs produce diagnostics.

### Consistency

- Preview frame and export frame match within visual tolerance.
- Golden images are stable enough for CI.
- Renderer version and font policy are recorded.

### Performance

- Dense comment segments render within acceptable preview latency.
- Long render simulation does not show unbounded memory growth.
- Caching materially reduces repeated layout cost.

### Packaging and Licensing

- Dependencies can be bundled or required safely.
- Font and emoji policies are legally viable.
- macOS app notarization remains feasible.

## 7. Kill / Revisit Criteria

Revisit this ADR if:

- the selected renderer stack cannot be bundled safely,
- golden tests are unstable beyond practical tolerance,
- CJK/emoji behavior cannot be made reliable,
- per-frame rasterization is too slow without a major GPU project,
- font fallback cannot be diagnosed,
- renderer integration prevents usable preview/export progress reporting.

A revisit does not automatically mean abandoning the shared renderer. It may mean changing renderer stack, output model, cache strategy, or compositor path.

## 8. Renderer API Shape

Recommended coarse API:

```text
load_renderer_environment(font_policy, assets) -> RendererEnvironment
measure_overlay(overlay_plan, time_ms) -> OverlayMeasurement
render_overlay_frame(overlay_plan, time_ms, output_scale) -> RgbaFrame
render_debug_overlay(overlay_plan, time_ms) -> RgbaFrame + LayoutDiagnostics
prewarm_cache(time_range, comment_ids) -> CacheReport
```

Do not expose high-frequency per-glyph or per-row APIs to Swift unless necessary.

## 9. Consequences

### Positive

- Stronger visual control.
- Better portability.
- Better testability.
- Better user trust in preview.
- Clearer separation of UI and render logic.

### Negative

- Higher initial complexity.
- More dependency management.
- More explicit font/emoji work.
- Need for renderer-specific test infrastructure.

## 10. Decision Record Output Required After Phase 0

After Phase 0, update this ADR with:

- evaluated stack,
- benchmark data,
- golden test evidence,
- packaging assessment,
- licensing assessment,
- chosen v1.0 renderer path,
- fallback path,
- known limitations,
- revisit triggers.


---

<!-- Source file: 05_phase0_technical_validation_plan.md -->

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


---

<!-- Source file: 06_implementation_plan.md -->

# 06. Implementation Plan

## 1. Implementation Strategy

Build the product from the renderer/media vertical slice outward. Do not build broad UI or multi-source UX before the end-to-end render path is proven.

The order is:

1. Bridge.
2. Data model.
3. Canonical import.
4. Renderer harness.
5. Media probe.
6. Segment export.
7. Project package.
8. Minimal macOS workflow.
9. Hardening for v1.0.
10. v1.x expansion.

## 2. Phase 0: Technical Validation

### Goal

Prove the architecture.

### Workstreams

#### Bridge

- Create Rust workspace.
- Create Swift macOS prototype.
- Choose UniFFI or C ABI prototype.
- Pass structured diagnostics across boundary.

#### Core

- Define `CommentEvent`.
- Define `CommentSource`.
- Define `ImportResult`.
- Define `Diagnostic`.
- Define `SemanticRenderPlan`.
- Implement canonical JSON importer.

#### Renderer

- Select renderer candidate stack.
- Render comment rows to RGBA.
- Implement basic font policy.
- Add CJK/emoji/multiline cases.
- Output golden PNGs.

#### Media

- Integrate ffprobe.
- Normalize media metadata.
- Compose source region + overlay region.
- Export short segment.
- Record diagnostics.

### Exit

Go only if Phase 0 acceptance criteria pass.

## 3. Phase 1: v1.0 Core Foundation

### Goal

Create stable product data foundations.

### Deliverables

- `replay_core` crate.
- `replay_importers` module.
- Project schema v1.
- Import diagnostics schema.
- Render-plan schema.
- Versioned style token schema.
- Comment source model with future multi-source support.
- Stable ID policy.
- Timestamp parsing and validation.
- Project serialization/deserialization.
- Basic schema migration mechanism.

### Key Decisions

- Store all normalized timestamps as integer milliseconds relative to video start.
- Preserve original timestamp where available.
- Store source ID even for single-source projects.
- Keep offset separate from imported comment timestamp.

## 4. Phase 2: v1.0 Shared Overlay Renderer

### Goal

Move renderer from prototype to production path.

### Deliverables

- Renderer environment initialization.
- Font policy v1.
- Paragraph measurement.
- Comment row layout.
- Panel layout.
- RGBA frame output.
- Premultiplied alpha handling policy.
- Renderer diagnostics.
- Golden image tests.
- Cache keys and invalidation.
- CLI renderer harness.

### Minimum Feature Set

- author + text rows,
- multiline comments,
- max lines,
- ellipsis,
- CJK wrapping,
- emoji sequence handling,
- simple badge placeholder,
- pinned message placeholder or reserved model,
- transparent background,
- panel background and separators.

## 5. Phase 3: v1.0 Media and Export Pipeline

### Goal

Build reliable segment and full export.

### Deliverables

- Media probing.
- Compatibility decision tree.
- Apple-native compatible path.
- FFmpeg-assisted preparation path for known unsupported cases.
- Source placement compositor.
- Overlay compositor.
- Preview preset.
- High-quality upload preset.
- AAC passthrough attempt where safe.
- AAC re-encode fallback.
- Progress and cancellation.
- Export diagnostics.

### Key Constraints

- Source placement should avoid fractional scaling.
- Video will usually be re-encoded because canvas composition changes the frame.
- Audio passthrough may be attempted, but must be diagnosed and fallback-safe.

## 6. Phase 4: v1.0 macOS Product Shell

### Goal

Build the minimal native user workflow.

### Deliverables

- Document app.
- New/open/save project.
- Select video file.
- Import canonical JSON.
- Diagnostics panel.
- Basic video preview.
- Global offset control.
- Default layout preview.
- Preview segment render.
- Full export dialog.
- Render progress.
- Cancellation.
- Render history.

### UI Principle

The UI calls core and renderer/backend APIs. It does not compute authoritative layout, ordering, or comment visibility.

## 7. Phase 5: v1.0 Hardening

### Goal

Make the narrow product reliable enough to release.

### Deliverables

- Crash-safe project saves.
- Better error messages.
- Missing media recovery.
- Import validation improvements.
- Golden image stability.
- Render consistency tests.
- Long-sample export tests.
- Memory profiling.
- Basic documentation.
- Sample files.
- Renderer decision record finalized.

## 8. v1.0 Release Criteria

v1.0 can ship when:

- one local video + one canonical JSON source can be imported,
- default layout can be rendered,
- preview and final export share overlay semantics,
- project save/load is stable,
- diagnostics are actionable,
- renderer stack decision is recorded,
- media path limitations are documented,
- no critical data-loss or render-corruption issues remain.

## 9. v1.1 Implementation: Import Expansion

### Deliverables

- CSV importer.
- JSONL importer.
- Column mapping UI.
- Large JSONL streaming import.
- Import diagnostics table.
- Source metadata display.
- Source enable/disable in project model.

## 10. v1.2 Implementation: Multi-Source Sync and Merge

### Deliverables

- Multiple sources in UI.
- Per-source offsets.
- Side-by-side source preview.
- Merged preview.
- Timestamp-based alignment when reliable metadata exists.
- Manual offset nudge controls.
- Merge diagnostics.
- Multi-platform label rendering.

## 11. v1.3 Implementation: Layout and Styling

### Deliverables

- Additional layout templates.
- Style token editor.
- Metadata overlay controls.
- Pinned comment region controls.
- Platform/source color controls.
- Template save/load.
- Template validation.

## 12. Future Implementation Tracks

### Plugin Runner

- Protocol draft.
- External executable runner.
- JSON-over-stdio request/response.
- Timeout and cancellation.
- User-approved plugin directories.
- Security review.

### Windows Feasibility

- Rust CI on Windows.
- Overlay renderer CI on Windows.
- Project fixture validation.
- Windows media backend spike.
- WinUI shell spike.

### CLI and Automation

- CLI importer.
- CLI render-plan generator.
- CLI overlay frame generator.
- Batch render prototype.


---

<!-- Source file: 07_data_model_storage_schema.md -->

# 07. Data Model and Storage Schema

## 1. Data Model Principles

- Import is non-destructive.
- Normalized comments are stored separately from source offsets.
- Multi-source support exists in the model from day one, even if v1.0 UI exposes only one source.
- Project files are versioned and platform-neutral.
- Renderer and media decisions are reproducibility inputs.
- All authoritative timestamps are integer milliseconds relative to video start after normalization.

## 2. Normalized Comment Event

Conceptual Rust model:

```rust
struct CommentEvent {
    id: String,
    source_id: String,
    platform: Option<String>,
    timestamp_ms: i64,
    original_timestamp: Option<OriginalTimestamp>,
    kind: CommentKind,
    author: Option<CommentAuthor>,
    body: CommentBody,
    style: Option<CommentStyle>,
    badges: Vec<Badge>,
    emotes: Vec<Emote>,
    source: Option<CommentSourceRef>,
    raw_ref: Option<RawSourceRef>,
    import_order: u64,
}
```

## 3. Comment Source

```rust
struct CommentSource {
    source_id: String,
    display_name: String,
    platform: Option<String>,
    importer_id: String,
    original_file_ref: AssetRef,
    enabled: bool,
    offset_ms: i64,
    timestamp_basis: TimestampBasis,
    diagnostics_ref: Option<String>,
    visual_style: Option<SourceVisualStyle>,
    metadata: BTreeMap<String, String>,
    import_fingerprint: ImportFingerprint,
}
```

## 4. Timestamp Basis

Supported values:

- `relative_to_video_start`,
- `relative_to_stream_start`,
- `absolute_utc`,
- `local_time_with_timezone`,
- `local_time_ambiguous`,
- `unknown`.

v1.0 should require canonical JSON that maps confidently to video-relative time or emits a clear synchronization-required diagnostic.

## 5. Stable ID Rules

- `source_id` is unique within a project.
- `comment.id` is unique within a source.
- Global comment key is `{source_id}:{comment.id}`.
- Generated IDs must be deterministic for the same input file and importer version.
- Duplicate IDs are diagnostics, not silent overwrites.

## 6. Import Result

```rust
struct ImportResult {
    source: CommentSource,
    comments: Vec<CommentEvent>,
    diagnostics: Vec<Diagnostic>,
    skipped_count: u64,
    importer_version: String,
    detected_format: String,
    detected_encoding: Option<String>,
}
```

## 7. Diagnostic Model

```rust
struct Diagnostic {
    id: String,
    severity: DiagnosticSeverity,
    category: DiagnosticCategory,
    message: String,
    source_ref: Option<SourceRef>,
    code: String,
    actionable_hint: Option<String>,
}
```

Severity:

- info,
- warning,
- error,
- fatal.

Categories:

- import,
- timestamp,
- media,
- renderer,
- export,
- project,
- storage,
- sync,
- schema.

## 8. Project Package

Recommended structure:

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

## 9. `project.json` Contents

```json
{
  "schemaVersion": "1.0",
  "projectId": "...",
  "createdAt": "...",
  "updatedAt": "...",
  "sourceVideo": {
    "assetRef": "...",
    "mediaInfoRef": "..."
  },
  "commentSources": [
    {
      "sourceId": "main",
      "displayName": "Main Chat",
      "platform": null,
      "enabled": true,
      "offsetMs": 0,
      "timestampBasis": "relative_to_video_start"
    }
  ],
  "layout": {
    "templateId": "classic-sidebar-v1",
    "overrides": {}
  },
  "renderSettings": {
    "preset": "high_quality_upload"
  },
  "overlayRenderer": {
    "rendererId": "shared",
    "rendererVersion": "...",
    "fontPolicyId": "default-v1"
  }
}
```

## 10. SQLite Tables

Suggested v1.0 tables:

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

Suggested future tables:

```sql
CREATE TABLE sources (...);
CREATE TABLE diagnostics (...);
CREATE TABLE raw_payload_refs (...);
CREATE TABLE renderer_cache_index (...);
```

## 11. Render Plan Model

Separate semantic plan and execution profile.

### SemanticRenderPlan

```json
{
  "schemaVersion": "1.0",
  "canvas": {
    "width": 1920,
    "height": 1080,
    "fps": 30
  },
  "regions": {
    "sourceVideo": {
      "x": 80,
      "y": 120,
      "width": 1280,
      "height": 720
    },
    "comments": {
      "x": 1400,
      "y": 120,
      "width": 440,
      "height": 720
    }
  },
  "timeline": {
    "commentWindowMs": 30000,
    "globalOffsetMs": 0,
    "sources": [
      {
        "sourceId": "main",
        "offsetMs": 0,
        "enabled": true
      }
    ]
  },
  "overlay": {
    "renderer": "shared",
    "fontPolicy": "project-default",
    "emojiPolicy": "shared-or-diagnosed-system",
    "textLayoutMode": "deterministic"
  }
}
```

### RenderExecutionProfile

```json
{
  "kind": "preview_segment",
  "startMs": 60000,
  "durationMs": 30000,
  "outputScale": 1.0,
  "bitratePolicy": "preview",
  "cachePolicy": "reuse_allowed"
}
```

## 12. Reproducibility Manifest

Add `reproducibility.json`:

```json
{
  "appVersion": "...",
  "coreVersion": "...",
  "rendererVersion": "...",
  "fontPolicyVersion": "...",
  "templateVersion": "...",
  "ffmpegVersion": "...",
  "mediaBackend": "...",
  "sourceVideoHash": "...",
  "commentFileHash": "...",
  "importerId": "canonical-json",
  "importerVersion": "...",
  "importOptionsHash": "..."
}
```

This is especially important for archival and research use.


---

<!-- Source file: 08_media_pipeline_plan.md -->

# 08. Media Pipeline Plan

## 1. Media Strategy

Use a two-tier media strategy:

1. **Apple-native compatible path** for common MP4/MOV H.264/HEVC + AAC sources.
2. **FFmpeg-assisted compatibility path** for probing, diagnostics, proxy/intermediate generation, remuxing, audio conversion, and fallback workflows.

The application must not assume all local archive videos are directly usable by AVFoundation.

## 2. Probe First

Every source video should be probed before import completion.

### Probe Fields

- duration,
- resolution,
- frame rate,
- constant/variable frame rate indicator,
- codec,
- container,
- pixel format,
- audio track presence,
- audio codec,
- audio track count,
- subtitle track presence,
- color metadata,
- HDR/SDR indicator,
- estimated bitrate,
- timebase,
- start time,
- rotation metadata,
- sample aspect ratio,
- stream disposition/default track metadata.

## 3. Compatibility Decision Tree

```text
1. Probe source media with ffprobe.
2. Normalize metadata into ReplayMediaInfo.
3. Check Apple-native compatibility.
4. If container and streams are compatible:
   - use Apple-native compatible path.
5. If container is unsupported but streams are compatible:
   - attempt remux into compatible intermediate.
6. If video codec is unsupported:
   - generate editing-friendly intermediate or proxy.
7. If audio codec is unsupported:
   - transcode audio to AAC.
8. If HDR, unusual color, VFR, multiple audio tracks, rotation, or SAR exists:
   - warn, apply deterministic policy, or request user choice where feasible.
9. If fallback conversion fails:
   - mark source unsupported with actionable diagnostics.
```

## 4. Render Pipeline

### Conceptual Flow

```text
Source video frames
  ↓
Decode / read frames
  ↓
Place source into output canvas region
  ↓
Render overlay RGBA frame through shared renderer
  ↓
Alpha composite overlay onto canvas
  ↓
Encode video
  ↓
Handle audio passthrough or re-encode
  ↓
Mux output
```

## 5. Source Video Quality Rules

- Preserve source dimensions where possible.
- Use larger canvas instead of shrinking source video when feasible.
- Avoid fractional scaling.
- Avoid unnecessary color conversion.
- Record color/HDR limitations.
- Do not promise mathematically lossless output.
- Compare cropped output source region against input frames in tests where feasible.

## 6. Audio Policy

### v1.0 Policy

1. If source is MP4/MOV with AAC audio and no audio edits are required, attempt passthrough where technically safe.
2. If passthrough is unsafe or fails, re-encode to AAC.
3. If source audio is not AAC, re-encode to AAC.
4. If source has no audio, emit diagnostic and export silent video.
5. If multiple audio tracks exist, select primary/default track in v1.0 and emit diagnostic.

### Diagnostics

Record:

- selected audio track,
- passthrough attempted,
- passthrough succeeded/failed,
- fallback used,
- codec and bitrate,
- audio/video sync checks.

## 7. Preview vs Final Export

Preview and final export differ by execution profile, not semantic overlay behavior.

### Preview

- short segment,
- lower bitrate,
- faster settings,
- optional lower quality encoder settings,
- same semantic render plan,
- same overlay renderer.

### Final Export

- full or selected duration,
- high-quality upload preset,
- quality-oriented bitrate or CRF equivalent,
- stable diagnostics,
- render history entry.

## 8. v1.0 Media Support Target

| Category | v1.0 Target |
|---|---|
| Primary containers | MP4, MOV |
| Additional containers | diagnostic/fallback best effort |
| Primary video codecs | H.264, HEVC |
| Primary audio codec | AAC |
| Audio passthrough | Attempt for compatible AAC where safe |
| Audio fallback | AAC re-encode |
| Primary source sizes | 720p and 1080p |
| Frame rate | CFR preferred, VFR diagnosed |
| HDR | detect and warn, not full guarantee |
| Multiple audio tracks | select primary/default with diagnostic |
| Subtitles | not v1.0 |
| Alpha video | not v1.0 |

## 9. Render Backend Risks

### AVFoundation-only risk

AVFoundation may not handle all archive files, containers, timebases, or color cases. Mitigation: probe first and keep FFmpeg fallback path.

### Frame-by-frame rendering cost

Full-frame composition can be expensive. Mitigation: start with correctness, then profile caching, GPU composition, or tile-based overlay if required.

### Color metadata risk

Incorrect color/range conversion can visibly alter source. Mitigation: capture metadata and add visual tests.

### Audio muxing risk

Audio passthrough may be unsafe in some composed video paths. Mitigation: fallback to AAC re-encode and diagnose.

## 10. Phase 0 Media Questions

- Which frame composition path is stable enough for v1.0?
- Can overlay RGBA frames be composited without alpha artifacts?
- Can progress and cancellation be implemented cleanly?
- How should VFR be represented in diagnostics?
- Is AAC passthrough practical with the selected video export path?
- Which FFmpeg distribution model is viable?


---

<!-- Source file: 09_testing_validation_matrix.md -->

# 09. Testing and Validation Matrix

## 1. Testing Strategy

Testing must validate product invariants, not only individual functions. The most important invariants are:

- imports normalize comments deterministically,
- renderer owns text measurement,
- preview and export share overlay semantics,
- media decisions are diagnosable,
- project files are portable and versioned,
- source video quality is not unnecessarily degraded.

## 2. Rust Core Tests

| Area | Required Tests |
|---|---|
| Timestamp parsing | ms, seconds, HH:MM:SS, MM:SS, invalid formats |
| Import normalization | canonical JSON happy path, missing fields, duplicate IDs |
| Diagnostics | warnings/errors/fatal categories and actionable hints |
| Sorting | timestamp, event priority, source priority, import order, stable ID |
| Offsets | global offset, source offset, negative/out-of-range results |
| Visibility | comment window, max visible comments, disabled source behavior |
| Serialization | project load/save, schema version, migration fixtures |
| Source model | source IDs, platform labels, importer metadata |

## 3. Importer Robustness Tests

v1.0 canonical JSON:

- valid minimal file,
- valid multiline file,
- valid CJK/emoji file,
- missing timestamp,
- invalid timestamp,
- duplicate ID,
- unknown kind,
- empty text,
- comments outside video duration,
- malformed JSON,
- huge text field.

v1.1 CSV/JSONL:

- column mapping,
- invalid encoding,
- partial line,
- streaming large file,
- row-level skip diagnostics.

## 4. Shared Overlay Renderer Tests

| Area | Required Tests |
|---|---|
| Font resolution | default font, missing font, fallback font |
| Unicode | grapheme boundaries, combining marks, ZWJ emoji |
| CJK | Japanese line break, mixed Latin/CJK, punctuation handling |
| Emoji | color emoji, variation selector, skin tone, unresolved glyph |
| Paragraph | max width, max lines, line height, baseline |
| Comment rows | author + body, long author, multiline body |
| Badges/emotes | inline placement, missing asset placeholder |
| Overflow | truncation, ellipsis, dense panel overflow |
| Raster | transparency, premultiplied alpha, panel backgrounds |
| Cache | cache key invalidation, renderer version change |
| Golden | stable image output for fixtures |

## 5. Media Tests

| Area | Required Tests |
|---|---|
| Probing | MP4, MOV, unsupported container, no audio |
| Metadata | resolution, fps, duration, codec, timebase, rotation |
| Placement | exact integer source region, no unintended scaling |
| Export | preview segment, final export, cancellation |
| Audio | AAC passthrough attempt, AAC fallback, no audio, multi-track diagnostic |
| Color | metadata preservation or warning, SDR/HDR diagnostic |
| VFR | detection and deterministic policy diagnostic |

## 6. Preview/Export Consistency Tests

Required comparisons:

- same visible comments at same timestamp,
- same comment ordering,
- same wrapping,
- same truncation,
- same row height,
- same pinned/metadata placement,
- same panel bounds,
- no native text fallback in export.

Acceptable differences:

- bitrate artifacts,
- encoder differences,
- preview duration,
- execution speed,
- non-overlay video compression differences.

## 7. UI Tests

v1.0 macOS UI:

- create project,
- select video,
- import canonical JSON,
- show diagnostics,
- adjust global offset,
- preview segment,
- export final,
- cancel render,
- save/open project,
- missing source media recovery.

v1.x UI:

- multiple source import,
- per-source enable/disable,
- offset review,
- template selection,
- style token editing.

## 8. Performance Tests

Suggested provisional targets, to be finalized after Phase 0:

- Import 100,000 comments within an acceptable interactive window.
- Query a 30-second visible comment window within frame-budget-relevant time.
- Render dense 30-second overlay sample without runaway memory growth.
- Avoid unbounded memory growth during simulated 2-hour render.
- Produce preview segment quickly enough for sync review.
- Keep project load/save acceptable with large comment stores.

## 9. Security and Privacy Tests

- Project does not leak full local paths in exported diagnostics by default.
- Plugin execution absent in v1.0.
- Sandbox file access model is documented.
- External file references recover gracefully.
- User-selected font/assets are referenced safely.

## 10. Release Gate Matrix

| Gate | Must Pass |
|---|---|
| Phase 0 | vertical slice, renderer proof, media segment export |
| v1.0 alpha | project package, default layout, preview/final export |
| v1.0 beta | diagnostics, cancellation, consistency, sample docs |
| v1.0 release | no critical data loss, no critical render corruption, documented limitations |
| v1.1 | CSV/JSONL tests and importer diagnostics |
| v1.2 | multi-source sync and merge consistency |
| v1.3 | template validation and renderer consistency across templates |


---

<!-- Source file: 10_risks_and_gates.md -->

# 10. Risks and Gates

## 1. Risk Register

### R1. Shared Renderer Complexity

**Risk:** Building a shared renderer in v1.0 is technically heavy.

**Mitigation:** Validate in Phase 0, use proven libraries, scope renderer to overlays, build CLI harness, add golden tests early.

**Gate:** Continue only if CJK/emoji/multiline rendering and preview/export consistency are viable.

### R2. Renderer Performance

**Risk:** Dense comments or long videos make overlay rendering too slow.

**Mitigation:** Cache shaped runs, paragraph measurements, static row fragments, badges, and emotes. Render only visible comments. Profile dense fixtures early.

**Gate:** Dense 30-second fixture must render acceptably before v1.0 hardening.

### R3. Font and Emoji Fidelity

**Risk:** Font fallback, emoji assets, and missing glyphs vary by platform or license constraints.

**Mitigation:** Define default font policy, record font versions, diagnose missing glyphs, use bundled open-source fonts where feasible.

**Gate:** Font and asset licensing review before public release.

### R4. Media Backend Feasibility

**Risk:** Combining video decode, overlay composition, audio, export, progress, and cancellation becomes unstable.

**Mitigation:** Build full segment export in Phase 0. Keep media backend replaceable. Separate overlay rendering from media execution.

**Gate:** 10–60 second export must succeed with diagnostics before broader app work.

### R5. Source Video Quality Degradation

**Risk:** Scaling, color conversion, or encoding decisions visibly degrade source video.

**Mitigation:** Preserve native source region where possible, avoid fractional scaling, add frame comparison tests, diagnose color/HDR/VFR.

**Gate:** Default layout must pass visual and automated placement checks.

### R6. FFmpeg Packaging and Licensing

**Risk:** FFmpeg distribution affects licensing, notarization, app size, and updates.

**Mitigation:** Decide bundled vs external early. Isolate behind `ReplayMedia`. Document version. Review licenses.

**Gate:** Distribution model decision before v1.0 beta.

### R7. Swift/Rust Interop Complexity

**Risk:** Bridge introduces memory, build, or debugging problems.

**Mitigation:** Keep API coarse. Use generated bindings or disciplined C ABI. Test ownership and errors.

**Gate:** Bridge smoke and large payload tests must pass before UI expansion.

### R8. Scope Creep

**Risk:** Multi-source, styling, plugins, and Windows work delay v1.0.

**Mitigation:** Enforce scope partition. Keep multi-source in model, not UI. Move broad features to v1.x.

**Gate:** v1.0 backlog cannot include plugin runner or full multi-source UX.

### R9. Project File Instability

**Risk:** Early schema changes break projects.

**Mitigation:** Version schemas from the start, use fixtures, add migration tests.

**Gate:** Project load/save fixtures before v1.0 alpha.

### R10. Diagnostics Weakness

**Risk:** Users cannot understand failed imports, unsupported media, renderer issues, or export failures.

**Mitigation:** Structured diagnostics with severity, category, source reference, and actionable hint.

**Gate:** v1.0 cannot ship with opaque fatal errors for common failure classes.

## 2. Phase Gates

### Gate 0: Architecture Feasibility

Required:

- Swift calls Rust.
- Canonical import works.
- Renderer produces overlay frame.
- Media segment export succeeds.
- Diagnostics exist.

Decision:

- Go,
- Revisit renderer/media stack,
- Stop.

### Gate 1: Renderer Commitment

Required:

- CJK fixture passes.
- Emoji fixture passes.
- Golden images stable enough.
- Dense sample performance acceptable.
- Packaging/licensing plausible.

Decision:

- Adopt selected renderer stack,
- change stack,
- reduce renderer feature set,
- revisit architecture.

### Gate 2: v1.0 Alpha

Required:

- project package,
- default layout,
- preview export,
- final export,
- import/media/render diagnostics,
- save/open.

### Gate 3: v1.0 Beta

Required:

- cancellation,
- crash-safe saves,
- long sample testing,
- audio policy stable,
- docs and samples,
- no critical consistency mismatch.

### Gate 4: v1.0 Release

Required:

- release notes document limitations,
- renderer ADR finalized,
- no critical data loss,
- no critical export corruption,
- diagnostics acceptable,
- source quality strategy validated.

## 3. Decision Rules

- If renderer fails but media works, do not proceed to broad UI; revisit renderer stack.
- If media fails but renderer works, do not proceed to broad UI; revisit compositor/export path.
- If both work but performance is poor, profile before expanding scope.
- If project schema is unstable, do not add multi-source UI.
- If diagnostics are weak, do not add more import formats.


---

<!-- Source file: 11_backlog_v1x_future.md -->

# 11. v1.x and Future Backlog

## 1. Backlog Principle

Future features should be designed for through schemas and boundaries, but not implemented before v1.0 unless they directly reduce architectural risk.

## 2. v1.1 Backlog: Import Expansion

### Features

- CSV importer.
- JSONL importer.
- Column mapping UI.
- Import preview table.
- Streaming import for large files.
- Better skipped-row diagnostics.
- Source metadata inspector.

### Technical Tasks

- Importer trait stabilization.
- Row-level diagnostics.
- Large file memory tests.
- Encoding detection policy.
- Import fixture suite.

## 3. v1.2 Backlog: Multi-Source Merge and Sync

### Features

- Multiple sources in UI.
- Per-source offsets.
- Source enable/disable.
- Merged timeline preview.
- Side-by-side source preview.
- Manual sync nudging.
- Anchor-based manual sync.
- Platform labels in overlay.
- Merge diagnostics.

### Technical Tasks

- Stable merge ordering.
- Source priority policy.
- Offset non-destructive update tests.
- Automatic alignment from reliable absolute timestamps.
- Split-platform review layout.

## 4. v1.3 Backlog: Layout and Styling

### Features

- Classic Sidebar refinement.
- Large Canvas Archive layout.
- Vertical Social layout.
- Research Transcript layout.
- Merged Multi-Platform Chat layout.
- Split Multi-Platform Review layout.
- Typography controls.
- Color and spacing tokens.
- Metadata region controls.
- Pinned comment presentation.
- User template saving.

### Technical Tasks

- Template schema validation.
- Template migration.
- Style token preview.
- Golden tests per template.
- Template diagnostics.

## 5. Plugin Backlog

### Importer Plugins

- JSON-over-stdio protocol.
- Versioned request/response schema.
- External executable runner.
- Timeouts.
- Cancellation.
- Logging.
- Example plugin.
- Security review.

### Overlay/Data Plugins

- Generate additional timeline events.
- Provide metadata overlays.
- Export subtitles or comment indexes.

### Exclusions

Do not support in-process binary plugins initially.
Do not allow arbitrary template scripting in v1.x without a dedicated security model.

## 6. Windows Backlog

### Early Compatibility

- Build Rust core on Windows CI.
- Build overlay renderer on Windows CI.
- Validate project fixtures on Windows.
- Avoid macOS-only assumptions in schemas.

### Productization Later

- WinUI 3 shell.
- Windows media backend.
- Media Foundation / Direct3D / FFmpeg evaluation.
- Shared renderer integration.
- Cross-platform project compatibility tests.

### Windows Non-Goal

Do not target full-pipeline pixel-identical output. Target schema compatibility, shared core behavior, and shared overlay behavior.

## 7. Automation Backlog

- CLI importer.
- CLI render-plan generator.
- CLI overlay renderer.
- Batch render command.
- Render preset files.
- Headless regression testing.

## 8. Platform Import Backlog

- YouTube archive chat import format.
- Twitch archive chat import format.
- Third-party chat downloader output import.
- Event platform exports.
- Research dataset mappings.

Direct API retrieval and VOD download remain future product decisions, not assumed requirements.

## 9. Research/Archive Backlog

- Normalized comment timeline export.
- Source provenance report.
- Render reproducibility report.
- Chapter file export.
- Subtitle-like export.
- Comment index export.
- Redacted diagnostics package.

## 10. Explicitly Deferred Indefinitely Unless Reprioritized

- Full nonlinear editing.
- Cloud rendering.
- Real-time livestream capture.
- Mobile apps.
- Collaborative editing.
- Fully platform-faithful chat UI recreation.
- Arbitrary motion graphics scripting.
