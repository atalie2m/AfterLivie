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
