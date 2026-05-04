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
