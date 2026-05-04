use crate::{canonical_json, csv_importer, jsonl_importer};
use chrono::{DateTime, Utc};
use replay_core::{
    effective_timestamp_ms, sort_comments, CommentEvent, CommentSource, Diagnostic,
    DiagnosticCategory, DiagnosticSeverity, ImportResult, SourceRef, TimestampBasis,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportFormat {
    CanonicalJson,
    Csv,
    Jsonl,
}

impl ImportFormat {
    pub fn from_path(path: impl AsRef<Path>) -> Self {
        match path
            .as_ref()
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.to_ascii_lowercase())
            .as_deref()
        {
            Some("csv") => Self::Csv,
            Some("jsonl") | Some("ndjson") => Self::Jsonl,
            _ => Self::CanonicalJson,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportSourceSpec {
    pub path: String,
    pub format: ImportFormat,
    pub source_id: String,
    pub display_name: Option<String>,
    pub platform: Option<String>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub offset_ms: i64,
    #[serde(default)]
    pub csv_mapping: Option<CsvColumnMapping>,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
    #[serde(default)]
    pub timestamp_basis: Option<TimestampBasis>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceManifest {
    pub sources: Vec<ImportSourceSpec>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CsvColumnMapping {
    pub timestamp: String,
    pub text: String,
    pub id: Option<String>,
    pub author: Option<String>,
    pub kind: Option<String>,
    pub platform: Option<String>,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreview {
    pub source_id: String,
    pub format: ImportFormat,
    pub importer_id: String,
    pub importer_version: String,
    pub detected_encoding: Option<String>,
    pub headers: Vec<String>,
    pub csv_mapping: Option<CsvColumnMapping>,
    pub rows: Vec<ImportPreviewRow>,
    pub diagnostics: Vec<Diagnostic>,
    pub skipped_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreviewRow {
    pub row_number: u64,
    pub source_id: String,
    pub comment_id: String,
    pub timestamp_ms: i64,
    pub author_display_name: Option<String>,
    pub text: String,
    pub kind: String,
    pub platform: Option<String>,
}

impl From<&CommentEvent> for ImportPreviewRow {
    fn from(comment: &CommentEvent) -> Self {
        Self {
            row_number: comment
                .raw_ref
                .as_ref()
                .and_then(|raw| raw.row)
                .unwrap_or(0),
            source_id: comment.source_id.clone(),
            comment_id: comment.id.clone(),
            timestamp_ms: comment.timestamp_ms,
            author_display_name: comment
                .author
                .as_ref()
                .map(|author| author.display_name.clone()),
            text: comment.body.text.clone(),
            kind: format!("{:?}", comment.kind).to_ascii_lowercase(),
            platform: comment.platform.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportBatchResult {
    pub sources: Vec<CommentSource>,
    pub comments: Vec<CommentEvent>,
    pub diagnostics: Vec<Diagnostic>,
    pub skipped_count: u64,
    pub source_results: Vec<ImportSourceResultSummary>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportSourceResultSummary {
    pub source_id: String,
    pub display_name: String,
    pub importer_id: String,
    pub importer_version: String,
    pub detected_format: String,
    pub detected_encoding: Option<String>,
    pub comment_count: usize,
    pub skipped_count: u64,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergedTimelineReport {
    pub source_summaries: Vec<MergedTimelineSourceSummary>,
    pub rows: Vec<MergedTimelineRow>,
    pub diagnostics: Vec<Diagnostic>,
    #[serde(default)]
    pub alignment_suggestions: Vec<AlignmentSuggestion>,
    #[serde(default)]
    pub ordering_rules: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergedTimelineSourceSummary {
    pub source_id: String,
    pub display_name: String,
    pub platform: Option<String>,
    pub enabled: bool,
    pub offset_ms: i64,
    pub priority: usize,
    pub comment_count: usize,
    pub skipped_count: u64,
    pub first_effective_timestamp_ms: Option<i64>,
    pub last_effective_timestamp_ms: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergedTimelineRow {
    pub source_id: String,
    pub source_display_name: String,
    pub comment_id: String,
    pub original_timestamp_ms: i64,
    pub effective_timestamp_ms: i64,
    pub author_display_name: Option<String>,
    pub text: String,
    pub kind: String,
    pub platform: Option<String>,
    pub platform_label: Option<String>,
    pub source_priority: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlignmentSuggestion {
    pub source_id: String,
    pub suggested_offset_ms: i64,
    pub confidence: f32,
    pub basis: String,
    pub reason: String,
    pub requires_review: bool,
}

pub fn import_source(spec: &ImportSourceSpec) -> ImportResult {
    match spec.format {
        ImportFormat::CanonicalJson => {
            let mut result = canonical_json::import_canonical_json(&spec.path);
            apply_spec_to_result(&mut result, spec);
            result
        }
        ImportFormat::Csv => csv_importer::import_csv(spec),
        ImportFormat::Jsonl => jsonl_importer::import_jsonl(spec),
    }
}

pub fn preview_source(spec: &ImportSourceSpec, max_rows: usize) -> ImportPreview {
    match spec.format {
        ImportFormat::CanonicalJson => {
            let result = import_source(spec);
            ImportPreview {
                source_id: result.source.source_id,
                format: spec.format,
                importer_id: result.source.importer_id,
                importer_version: result.importer_version,
                detected_encoding: result.detected_encoding,
                headers: vec![],
                csv_mapping: None,
                rows: result
                    .comments
                    .iter()
                    .take(max_rows)
                    .map(ImportPreviewRow::from)
                    .collect(),
                diagnostics: result.diagnostics,
                skipped_count: result.skipped_count,
            }
        }
        ImportFormat::Csv => csv_importer::preview_csv(spec, max_rows),
        ImportFormat::Jsonl => jsonl_importer::preview_jsonl(spec, max_rows),
    }
}

pub fn import_sources(specs: &[ImportSourceSpec]) -> ImportBatchResult {
    let duplicate_diagnostics = duplicate_source_diagnostics(specs);
    if !duplicate_diagnostics.is_empty() {
        return ImportBatchResult {
            sources: vec![],
            comments: vec![],
            skipped_count: 0,
            source_results: vec![],
            diagnostics: duplicate_diagnostics,
        };
    }

    let mut sources = Vec::new();
    let mut comments = Vec::new();
    let mut diagnostics = Vec::new();
    let mut skipped_count = 0;
    let mut source_results = Vec::new();

    for spec in specs {
        let result = import_source(spec);
        skipped_count += result.skipped_count;
        diagnostics.extend(result.diagnostics.clone());
        source_results.push(ImportSourceResultSummary {
            source_id: result.source.source_id.clone(),
            display_name: result.source.display_name.clone(),
            importer_id: result.source.importer_id.clone(),
            importer_version: result.importer_version.clone(),
            detected_format: result.detected_format.clone(),
            detected_encoding: result.detected_encoding.clone(),
            comment_count: result.comments.len(),
            skipped_count: result.skipped_count,
            diagnostics: result.diagnostics.clone(),
        });
        sources.push(result.source);
        comments.extend(result.comments);
    }

    ImportBatchResult {
        sources,
        comments,
        diagnostics,
        skipped_count,
        source_results,
    }
}

pub fn merged_timeline_report(
    batch: &ImportBatchResult,
    global_offset_ms: i64,
    limit: usize,
) -> MergedTimelineReport {
    let mut diagnostics = batch.diagnostics.clone();
    diagnostics.extend(merge_diagnostics(batch, global_offset_ms));
    let alignment_suggestions = alignment_suggestions(batch, &mut diagnostics);
    let skipped_by_source = batch
        .source_results
        .iter()
        .map(|summary| (summary.source_id.clone(), summary.skipped_count))
        .collect::<BTreeMap<_, _>>();
    let source_display_names = batch
        .sources
        .iter()
        .map(|source| (source.source_id.clone(), source.display_name.clone()))
        .collect::<BTreeMap<_, _>>();
    let source_platforms = batch
        .sources
        .iter()
        .map(|source| (source.source_id.clone(), source.platform.clone()))
        .collect::<BTreeMap<_, _>>();
    let source_priorities = source_priorities(&batch.sources);
    let mut enabled_comments = batch
        .comments
        .iter()
        .filter(|comment| {
            batch
                .sources
                .iter()
                .find(|source| source.source_id == comment.source_id)
                .map(|source| source.enabled)
                .unwrap_or(false)
        })
        .cloned()
        .collect::<Vec<_>>();
    sort_comments(&mut enabled_comments, &batch.sources, global_offset_ms);
    let source_offsets = batch
        .sources
        .iter()
        .map(|source| (source.source_id.clone(), source.offset_ms))
        .collect::<BTreeMap<_, _>>();
    let rows = enabled_comments
        .iter()
        .take(limit)
        .map(|comment| MergedTimelineRow {
            source_id: comment.source_id.clone(),
            source_display_name: source_display_names
                .get(&comment.source_id)
                .cloned()
                .unwrap_or_else(|| comment.source_id.clone()),
            comment_id: comment.id.clone(),
            original_timestamp_ms: comment.timestamp_ms,
            effective_timestamp_ms: effective_timestamp_ms(
                comment,
                &source_offsets,
                global_offset_ms,
            ),
            author_display_name: comment
                .author
                .as_ref()
                .map(|author| author.display_name.clone()),
            text: comment.body.text.clone(),
            kind: format!("{:?}", comment.kind).to_ascii_lowercase(),
            platform: comment.platform.clone(),
            platform_label: platform_label(comment, &source_platforms),
            source_priority: source_priorities
                .get(&comment.source_id)
                .copied()
                .unwrap_or(usize::MAX),
        })
        .collect();

    MergedTimelineReport {
        source_summaries: batch
            .sources
            .iter()
            .map(|source| MergedTimelineSourceSummary {
                source_id: source.source_id.clone(),
                display_name: source.display_name.clone(),
                platform: source.platform.clone(),
                enabled: source.enabled,
                offset_ms: source.offset_ms,
                priority: source_priorities
                    .get(&source.source_id)
                    .copied()
                    .unwrap_or(usize::MAX),
                comment_count: batch
                    .comments
                    .iter()
                    .filter(|comment| comment.source_id == source.source_id)
                    .count(),
                skipped_count: skipped_by_source
                    .get(&source.source_id)
                    .copied()
                    .unwrap_or_default(),
                first_effective_timestamp_ms: source_effective_range(
                    batch,
                    source,
                    &source_offsets,
                    global_offset_ms,
                )
                .map(|range| range.0),
                last_effective_timestamp_ms: source_effective_range(
                    batch,
                    source,
                    &source_offsets,
                    global_offset_ms,
                )
                .map(|range| range.1),
            })
            .collect(),
        rows,
        diagnostics,
        alignment_suggestions,
        ordering_rules: ordering_rules(),
    }
}

fn apply_spec_to_result(result: &mut ImportResult, spec: &ImportSourceSpec) {
    let source_id = spec.source_id.trim();
    if !source_id.is_empty() {
        result.source.source_id = source_id.into();
    }
    if let Some(display_name) = spec
        .display_name
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        result.source.display_name = display_name.trim().into();
    }
    if spec.platform.is_some() {
        result.source.platform = spec.platform.clone();
    }
    result.source.enabled = spec.enabled;
    result.source.offset_ms = spec.offset_ms;
    result.source.timestamp_basis = spec
        .timestamp_basis
        .clone()
        .unwrap_or_else(|| result.source.timestamp_basis.clone());
    result.source.metadata.extend(spec.metadata.clone());
    result.source.original_file_ref.path = spec.path.clone();
    for comment in &mut result.comments {
        comment.source_id = result.source.source_id.clone();
        if comment.platform.is_none() {
            comment.platform = result.source.platform.clone();
        }
        if let Some(source) = &mut comment.source {
            source.path = Some(spec.path.clone());
        }
    }
}

pub fn ordering_rules() -> Vec<String> {
    vec![
        "effective_timestamp_ms".into(),
        "event_priority".into(),
        "source_priority".into(),
        "import_order".into(),
        "stable_id".into(),
    ]
}

fn source_priorities(sources: &[CommentSource]) -> BTreeMap<String, usize> {
    sources
        .iter()
        .enumerate()
        .map(|(priority, source)| (source.source_id.clone(), priority))
        .collect()
}

fn platform_label(
    comment: &CommentEvent,
    source_platforms: &BTreeMap<String, Option<String>>,
) -> Option<String> {
    comment
        .platform
        .clone()
        .or_else(|| source_platforms.get(&comment.source_id).cloned().flatten())
}

fn source_effective_range(
    batch: &ImportBatchResult,
    source: &CommentSource,
    source_offsets: &BTreeMap<String, i64>,
    global_offset_ms: i64,
) -> Option<(i64, i64)> {
    batch
        .comments
        .iter()
        .filter(|comment| comment.source_id == source.source_id)
        .map(|comment| effective_timestamp_ms(comment, source_offsets, global_offset_ms))
        .fold(None, |range, value| match range {
            Some((first, last)) => Some((first.min(value), last.max(value))),
            None => Some((value, value)),
        })
}

fn merge_diagnostics(batch: &ImportBatchResult, global_offset_ms: i64) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let source_offsets = batch
        .sources
        .iter()
        .map(|source| (source.source_id.clone(), source.offset_ms))
        .collect::<BTreeMap<_, _>>();

    for source in &batch.sources {
        if !source.enabled {
            diagnostics.push(
                Diagnostic::new(
                    DiagnosticSeverity::Info,
                    DiagnosticCategory::Sync,
                    "sync.source_disabled",
                    format!(
                        "Source '{}' is disabled and omitted from merged rows.",
                        source.display_name
                    ),
                )
                .with_source_ref(SourceRef::Source {
                    source_id: source.source_id.clone(),
                })
                .with_hint("Enable the source before reviewing or rendering its comments."),
            );
        }

        if let Some((first, _)) =
            source_effective_range(batch, source, &source_offsets, global_offset_ms)
        {
            if first < 0 {
                diagnostics.push(
                    Diagnostic::new(
                        DiagnosticSeverity::Warning,
                        DiagnosticCategory::Sync,
                        "sync.negative_effective_timestamp",
                        format!(
                            "Source '{}' has comments before time zero after offsets.",
                            source.display_name
                        ),
                    )
                    .with_source_ref(SourceRef::Source {
                        source_id: source.source_id.clone(),
                    })
                    .with_hint("Review the global and per-source offsets for this source."),
                );
            }
        }
    }

    let mut enabled_comments = batch
        .comments
        .iter()
        .filter(|comment| {
            batch
                .sources
                .iter()
                .find(|source| source.source_id == comment.source_id)
                .map(|source| source.enabled)
                .unwrap_or(false)
        })
        .cloned()
        .collect::<Vec<_>>();
    sort_comments(&mut enabled_comments, &batch.sources, global_offset_ms);

    let mut timestamp_counts = BTreeMap::<i64, usize>::new();
    for comment in &enabled_comments {
        *timestamp_counts
            .entry(effective_timestamp_ms(
                comment,
                &source_offsets,
                global_offset_ms,
            ))
            .or_default() += 1;
    }
    for (timestamp_ms, count) in timestamp_counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .take(8)
    {
        diagnostics.push(
            Diagnostic::new(
                DiagnosticSeverity::Info,
                DiagnosticCategory::Sync,
                "sync.exact_timestamp_tie",
                format!(
                    "{count} comments share effective timestamp {timestamp_ms} ms; stable tie-breakers were applied."
                ),
            )
            .with_hint("Source priority controls ordering after event priority for exact timestamp ties."),
        );
    }

    diagnostics
}

fn alignment_suggestions(
    batch: &ImportBatchResult,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<AlignmentSuggestion> {
    if batch.sources.iter().filter(|source| source.enabled).count() < 2 {
        return vec![];
    }

    let mut reliable = Vec::new();
    for source in batch.sources.iter().filter(|source| source.enabled) {
        match source_sync_start(source) {
            Ok(Some(start)) => reliable.push((source, start)),
            Ok(None) => diagnostics.push(
                Diagnostic::new(
                    DiagnosticSeverity::Info,
                    DiagnosticCategory::Sync,
                    "sync.alignment_metadata_missing",
                    format!(
                        "Source '{}' has no reliable UTC sync metadata.",
                        source.display_name
                    ),
                )
                .with_source_ref(SourceRef::Source {
                    source_id: source.source_id.clone(),
                })
                .with_hint("Add syncStartUtc, streamStartUtc, or archiveStartUtc to source metadata to enable automatic offset suggestions."),
            ),
            Err(value) => diagnostics.push(
                Diagnostic::new(
                    DiagnosticSeverity::Warning,
                    DiagnosticCategory::Sync,
                    "sync.alignment_metadata_ambiguous",
                    format!(
                        "Source '{}' has UTC sync metadata that could not be parsed: {}.",
                        source.display_name, value
                    ),
                )
                .with_source_ref(SourceRef::Source {
                    source_id: source.source_id.clone(),
                })
                .with_hint("Use an RFC 3339 UTC timestamp such as 2026-05-04T12:00:00Z."),
            ),
        }
    }

    let Some((reference, reference_start)) = reliable.first() else {
        return vec![];
    };
    let reference_display_name = reference.display_name.clone();
    let reference_start = *reference_start;

    reliable
        .into_iter()
        .skip(1)
        .map(|(source, source_start)| AlignmentSuggestion {
            source_id: source.source_id.clone(),
            suggested_offset_ms: source_start
                .signed_duration_since(reference_start)
                .num_milliseconds(),
            confidence: 0.95,
            basis: "source_utc_start_metadata".into(),
            reason: format!(
                "Align '{}' to reference source '{}' using reliable UTC start metadata.",
                source.display_name, reference_display_name
            ),
            requires_review: true,
        })
        .collect()
}

fn source_sync_start(source: &CommentSource) -> Result<Option<DateTime<Utc>>, String> {
    for key in ["syncStartUtc", "streamStartUtc", "archiveStartUtc"] {
        let Some(value) = source
            .metadata
            .get(key)
            .filter(|value| !value.trim().is_empty())
        else {
            continue;
        };
        return DateTime::parse_from_rfc3339(value.trim())
            .map(|value| Some(value.with_timezone(&Utc)))
            .map_err(|_| value.clone());
    }
    Ok(None)
}

fn duplicate_source_diagnostics(specs: &[ImportSourceSpec]) -> Vec<Diagnostic> {
    let mut seen = BTreeSet::new();
    let mut diagnostics = Vec::new();
    for spec in specs {
        if spec.source_id.trim().is_empty() {
            diagnostics.push(
                Diagnostic::new(
                    DiagnosticSeverity::Fatal,
                    DiagnosticCategory::Import,
                    "import.missing_source_id",
                    "Source manifest entry is missing sourceId.",
                )
                .with_source_ref(SourceRef::File {
                    path: spec.path.clone(),
                })
                .with_hint("Assign a unique sourceId to every source in the manifest."),
            );
        } else if !seen.insert(spec.source_id.clone()) {
            diagnostics.push(
                Diagnostic::new(
                    DiagnosticSeverity::Fatal,
                    DiagnosticCategory::Import,
                    "import.duplicate_source_id",
                    format!("Source id '{}' appears more than once.", spec.source_id),
                )
                .with_source_ref(SourceRef::Source {
                    source_id: spec.source_id.clone(),
                })
                .with_hint("Use a unique sourceId for each imported source."),
            );
        }
    }
    diagnostics
}

fn default_enabled() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use replay_core::{AssetRef, CommentBody, CommentKind, ImportFingerprint, TimestampBasis};

    fn source(source_id: &str, path: &str, format: ImportFormat) -> ImportSourceSpec {
        ImportSourceSpec {
            path: path.into(),
            format,
            source_id: source_id.into(),
            display_name: Some(source_id.into()),
            platform: None,
            enabled: true,
            offset_ms: 0,
            csv_mapping: None,
            metadata: BTreeMap::new(),
            timestamp_basis: None,
        }
    }

    fn comment_source(source_id: &str, enabled: bool, offset_ms: i64) -> CommentSource {
        CommentSource {
            source_id: source_id.into(),
            display_name: source_id.into(),
            platform: None,
            importer_id: "test".into(),
            original_file_ref: AssetRef {
                path: format!("{source_id}.json"),
                hash: None,
            },
            enabled,
            offset_ms,
            timestamp_basis: TimestampBasis::RelativeToVideoStart,
            diagnostics_ref: None,
            visual_style: None,
            metadata: BTreeMap::new(),
            import_fingerprint: ImportFingerprint {
                algorithm: "sha256".into(),
                value: source_id.into(),
            },
        }
    }

    fn comment(source_id: &str, id: &str, timestamp_ms: i64) -> CommentEvent {
        comment_with_kind_order(source_id, id, timestamp_ms, CommentKind::Message, 0)
    }

    fn comment_with_kind_order(
        source_id: &str,
        id: &str,
        timestamp_ms: i64,
        kind: CommentKind,
        import_order: u64,
    ) -> CommentEvent {
        CommentEvent {
            id: id.into(),
            source_id: source_id.into(),
            platform: None,
            timestamp_ms,
            original_timestamp: None,
            kind,
            author: None,
            body: CommentBody { text: id.into() },
            style: None,
            badges: vec![],
            emotes: vec![],
            source: None,
            raw_ref: None,
            import_order,
        }
    }

    #[test]
    fn duplicate_source_ids_are_fatal() {
        let result = import_sources(&[
            source("main", "one.json", ImportFormat::CanonicalJson),
            source("main", "two.json", ImportFormat::CanonicalJson),
        ]);
        assert!(result.comments.is_empty());
        assert_eq!(result.diagnostics[0].severity, DiagnosticSeverity::Fatal);
        assert_eq!(result.diagnostics[0].code, "import.duplicate_source_id");
    }

    #[test]
    fn merged_timeline_rows_apply_offsets_and_omit_disabled_sources() {
        let batch = ImportBatchResult {
            sources: vec![
                comment_source("enabled", true, 500),
                comment_source("disabled", false, 0),
            ],
            comments: vec![
                comment("enabled", "a", 1_000),
                comment("disabled", "b", 1_000),
            ],
            diagnostics: vec![],
            skipped_count: 0,
            source_results: vec![],
        };
        let report = merged_timeline_report(&batch, 100, 10);
        assert_eq!(report.rows.len(), 1);
        assert_eq!(report.rows[0].comment_id, "a");
        assert_eq!(report.rows[0].effective_timestamp_ms, 1_600);
        assert_eq!(report.source_summaries.len(), 2);
    }

    #[test]
    fn merged_ordering_uses_event_priority_source_priority_import_order_and_id() {
        let batch = ImportBatchResult {
            sources: vec![
                comment_source("primary", true, 0),
                comment_source("secondary", true, 0),
            ],
            comments: vec![
                comment_with_kind_order("secondary", "z", 1_000, CommentKind::Message, 1),
                comment_with_kind_order("primary", "m", 1_000, CommentKind::Message, 2),
                comment_with_kind_order("secondary", "p", 1_000, CommentKind::Pinned, 3),
                comment_with_kind_order("primary", "a", 1_000, CommentKind::Message, 0),
            ],
            diagnostics: vec![],
            skipped_count: 0,
            source_results: vec![],
        };
        let report = merged_timeline_report(&batch, 0, 10);
        assert_eq!(
            report
                .rows
                .iter()
                .map(|row| row.comment_id.as_str())
                .collect::<Vec<_>>(),
            ["p", "a", "m", "z"]
        );
        assert_eq!(report.ordering_rules, ordering_rules());
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "sync.exact_timestamp_tie"));
    }

    #[test]
    fn source_priority_changes_tie_order() {
        let comments = vec![
            comment("primary", "a", 1_000),
            comment("secondary", "b", 1_000),
        ];
        let first = merged_timeline_report(
            &ImportBatchResult {
                sources: vec![
                    comment_source("primary", true, 0),
                    comment_source("secondary", true, 0),
                ],
                comments: comments.clone(),
                diagnostics: vec![],
                skipped_count: 0,
                source_results: vec![],
            },
            0,
            10,
        );
        let second = merged_timeline_report(
            &ImportBatchResult {
                sources: vec![
                    comment_source("secondary", true, 0),
                    comment_source("primary", true, 0),
                ],
                comments,
                diagnostics: vec![],
                skipped_count: 0,
                source_results: vec![],
            },
            0,
            10,
        );
        assert_eq!(first.rows[0].source_id, "primary");
        assert_eq!(second.rows[0].source_id, "secondary");
    }

    #[test]
    fn reliable_utc_metadata_produces_review_required_alignment_suggestion() {
        let mut reference = comment_source("reference", true, 0);
        reference
            .metadata
            .insert("syncStartUtc".into(), "2026-05-04T12:00:00Z".into());
        let mut secondary = comment_source("secondary", true, 0);
        secondary
            .metadata
            .insert("streamStartUtc".into(), "2026-05-04T12:00:05Z".into());
        let batch = ImportBatchResult {
            sources: vec![reference, secondary],
            comments: vec![comment("reference", "a", 0), comment("secondary", "b", 0)],
            diagnostics: vec![],
            skipped_count: 0,
            source_results: vec![],
        };
        let report = merged_timeline_report(&batch, 0, 10);
        assert_eq!(report.alignment_suggestions.len(), 1);
        assert_eq!(report.alignment_suggestions[0].source_id, "secondary");
        assert_eq!(report.alignment_suggestions[0].suggested_offset_ms, 5_000);
        assert!(report.alignment_suggestions[0].requires_review);
    }

    #[test]
    fn ambiguous_alignment_metadata_is_diagnostic_without_suggestion() {
        let mut reference = comment_source("reference", true, 0);
        reference
            .metadata
            .insert("syncStartUtc".into(), "2026-05-04T12:00:00Z".into());
        let mut secondary = comment_source("secondary", true, 0);
        secondary
            .metadata
            .insert("syncStartUtc".into(), "not a date".into());
        let batch = ImportBatchResult {
            sources: vec![reference, secondary],
            comments: vec![comment("reference", "a", 0), comment("secondary", "b", 0)],
            diagnostics: vec![],
            skipped_count: 0,
            source_results: vec![],
        };
        let report = merged_timeline_report(&batch, 0, 10);
        assert!(report.alignment_suggestions.is_empty());
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "sync.alignment_metadata_ambiguous"));
    }

    #[test]
    fn merge_report_keeps_existing_json_fields() {
        let batch = ImportBatchResult {
            sources: vec![comment_source("main", true, 0)],
            comments: vec![comment("main", "a", 1_000)],
            diagnostics: vec![],
            skipped_count: 0,
            source_results: vec![],
        };
        let report = merged_timeline_report(&batch, 0, 10);
        let value = serde_json::to_value(report).unwrap();
        assert!(value.get("sourceSummaries").is_some());
        assert!(value.get("rows").is_some());
        assert!(value.get("diagnostics").is_some());
        assert!(value.get("alignmentSuggestions").is_some());
        assert!(value["rows"][0].get("effectiveTimestampMs").is_some());
    }
}
