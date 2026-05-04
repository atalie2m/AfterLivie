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
