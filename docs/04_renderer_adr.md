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
