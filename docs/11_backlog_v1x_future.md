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
