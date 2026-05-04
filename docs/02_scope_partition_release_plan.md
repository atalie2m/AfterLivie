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
