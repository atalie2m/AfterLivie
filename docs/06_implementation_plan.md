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
