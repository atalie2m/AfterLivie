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
