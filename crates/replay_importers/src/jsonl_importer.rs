use crate::common::{
    duplicate_id_diagnostic, fingerprint_path, make_source, normalize_comment, parse_body_value,
    parse_kind_value, read_failed_diagnostic, CommentInput, ImporterResult, SourceDefaults,
};
use crate::source::{ImportFormat, ImportPreview, ImportPreviewRow, ImportSourceSpec};
use replay_core::{
    Badge, CommentAuthor, CommentEvent, CommentStyle, Diagnostic, DiagnosticCategory,
    DiagnosticSeverity, Emote, ImportResult, SourceRef, TimestampBasis,
};
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub const JSONL_IMPORTER_ID: &str = "jsonl";
pub const JSONL_IMPORTER_VERSION: &str = "1.0.0";

pub fn import_jsonl(spec: &ImportSourceSpec) -> ImportResult {
    let path = Path::new(&spec.path);
    let fingerprint = match fingerprint_path(path) {
        Ok(fingerprint) => fingerprint,
        Err(diagnostic) => return fatal_result(spec, *diagnostic),
    };
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) => return fatal_result(spec, read_failed_diagnostic(&spec.path, error)),
    };
    read_jsonl(BufReader::new(file), spec, fingerprint, None)
}

pub fn preview_jsonl(spec: &ImportSourceSpec, max_rows: usize) -> ImportPreview {
    let path = Path::new(&spec.path);
    let fingerprint = match fingerprint_path(path) {
        Ok(fingerprint) => fingerprint,
        Err(diagnostic) => {
            return ImportPreview {
                source_id: spec.source_id.clone(),
                format: ImportFormat::Jsonl,
                importer_id: JSONL_IMPORTER_ID.into(),
                importer_version: JSONL_IMPORTER_VERSION.into(),
                detected_encoding: Some("utf-8".into()),
                headers: vec![],
                csv_mapping: None,
                rows: vec![],
                diagnostics: vec![*diagnostic],
                skipped_count: 0,
            };
        }
    };
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) => {
            return ImportPreview {
                source_id: spec.source_id.clone(),
                format: ImportFormat::Jsonl,
                importer_id: JSONL_IMPORTER_ID.into(),
                importer_version: JSONL_IMPORTER_VERSION.into(),
                detected_encoding: Some("utf-8".into()),
                headers: vec![],
                csv_mapping: None,
                rows: vec![],
                diagnostics: vec![read_failed_diagnostic(&spec.path, error)],
                skipped_count: 0,
            };
        }
    };
    let result = read_jsonl(BufReader::new(file), spec, fingerprint, Some(max_rows));
    ImportPreview {
        source_id: result.source.source_id,
        format: ImportFormat::Jsonl,
        importer_id: JSONL_IMPORTER_ID.into(),
        importer_version: JSONL_IMPORTER_VERSION.into(),
        detected_encoding: result.detected_encoding,
        headers: vec![],
        csv_mapping: None,
        rows: result.comments.iter().map(ImportPreviewRow::from).collect(),
        diagnostics: result.diagnostics,
        skipped_count: result.skipped_count,
    }
}

#[cfg(test)]
pub(crate) fn import_jsonl_str(contents: &str, spec: &ImportSourceSpec) -> ImportResult {
    read_jsonl(
        BufReader::new(contents.as_bytes()),
        spec,
        crate::common::fingerprint_bytes(contents.as_bytes()),
        None,
    )
}

fn read_jsonl<R: BufRead>(
    mut reader: R,
    spec: &ImportSourceSpec,
    fingerprint: String,
    max_rows: Option<usize>,
) -> ImportResult {
    let mut comments = Vec::new();
    let mut diagnostics = Vec::new();
    let mut skipped_count = 0;
    let mut seen_ids = BTreeSet::new();
    let mut line = String::new();
    let mut row_number = 0_u64;

    loop {
        if max_rows.is_some_and(|max_rows| comments.len() >= max_rows) {
            break;
        }
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::InvalidData => {
                diagnostics.push(
                    Diagnostic::new(
                        DiagnosticSeverity::Fatal,
                        DiagnosticCategory::Import,
                        "import.invalid_encoding",
                        format!("JSONL file is not valid UTF-8: {error}"),
                    )
                    .with_source_ref(SourceRef::File {
                        path: spec.path.clone(),
                    })
                    .with_hint("Save the JSONL file as UTF-8 and try importing it again."),
                );
                break;
            }
            Err(error) => {
                diagnostics.push(read_failed_diagnostic(&spec.path, error));
                break;
            }
        }
        row_number += 1;
        let trimmed = line
            .trim_end_matches(['\r', '\n'])
            .trim_start_matches('\u{feff}')
            .trim();
        if trimmed.is_empty() {
            skipped_count += 1;
            diagnostics.push(
                Diagnostic::new(
                    DiagnosticSeverity::Error,
                    DiagnosticCategory::Import,
                    "import.empty_jsonl_line",
                    "JSONL line is empty.",
                )
                .with_source_ref(SourceRef::Row {
                    path: Some(spec.path.clone()),
                    row: row_number,
                })
                .with_hint("Remove empty lines from the JSONL file."),
            );
            continue;
        }
        match normalize_line(trimmed, spec, row_number) {
            Ok(comment) => {
                if !seen_ids.insert(comment.id.clone()) {
                    skipped_count += 1;
                    diagnostics.push(duplicate_id_diagnostic(
                        &spec.path,
                        &spec.source_id,
                        &comment.id,
                        row_number,
                    ));
                    continue;
                }
                comments.push(comment);
            }
            Err(diagnostic) => {
                skipped_count += 1;
                diagnostics.push(*diagnostic);
            }
        }
    }

    ImportResult {
        source: make_source(defaults(spec, Some(fingerprint), BTreeMap::new())),
        comments,
        diagnostics,
        skipped_count,
        importer_version: JSONL_IMPORTER_VERSION.into(),
        detected_format: "jsonl_v1".into(),
        detected_encoding: Some("utf-8".into()),
    }
}

fn normalize_line(
    line: &str,
    spec: &ImportSourceSpec,
    row_number: u64,
) -> ImporterResult<CommentEvent> {
    let row = serde_json::from_str::<JsonlComment>(line).map_err(|error| {
        Box::new(
            Diagnostic::new(
                DiagnosticSeverity::Error,
                DiagnosticCategory::Import,
                "import.malformed_jsonl_row",
                format!("JSONL row is not valid JSON: {error}"),
            )
            .with_source_ref(SourceRef::Row {
                path: Some(spec.path.clone()),
                row: row_number,
            })
            .with_hint("Each JSONL line must be one complete JSON comment object."),
        )
    })?;
    let timestamp_value = row
        .timestamp_ms
        .map(|value| value.to_string())
        .or(row.timestamp)
        .ok_or_else(|| {
            Box::new(
                Diagnostic::new(
                    DiagnosticSeverity::Error,
                    DiagnosticCategory::Timestamp,
                    "import.missing_timestamp",
                    "Comment row is missing timestampMs or timestamp.",
                )
                .with_source_ref(SourceRef::Row {
                    path: Some(spec.path.clone()),
                    row: row_number,
                })
                .with_hint("Add timestampMs as integer milliseconds relative to video start."),
            )
        })?;
    let body_text = parse_body_value(row.body, &spec.path, row_number)?;
    let kind = parse_kind_value(row.kind, &spec.path, row_number)?;
    let author_display_name = row.author.map(|author| author.display_name);

    let mut comment = normalize_comment(CommentInput {
        id: row.id,
        source_id: spec.source_id.clone(),
        platform: row.platform.or_else(|| spec.platform.clone()),
        timestamp_value,
        kind,
        author_display_name,
        body_text,
        source_path: spec.path.clone(),
        row_number,
        import_order: row_number - 1,
        importer_id: JSONL_IMPORTER_ID,
        importer_version: JSONL_IMPORTER_VERSION,
    })?;
    comment.style = row.style;
    comment.badges = row.badges.unwrap_or_default();
    comment.emotes = row.emotes.unwrap_or_default();
    Ok(comment)
}

fn defaults(
    spec: &ImportSourceSpec,
    fingerprint: Option<String>,
    mut metadata: BTreeMap<String, String>,
) -> SourceDefaults<'_> {
    metadata.extend(spec.metadata.clone());
    SourceDefaults {
        source_id: &spec.source_id,
        display_name: spec.display_name.as_deref().unwrap_or(&spec.source_id),
        platform: spec.platform.as_deref(),
        enabled: spec.enabled,
        offset_ms: spec.offset_ms,
        source_path: &spec.path,
        importer_id: JSONL_IMPORTER_ID,
        fingerprint,
        timestamp_basis: spec
            .timestamp_basis
            .clone()
            .unwrap_or(TimestampBasis::RelativeToVideoStart),
        metadata,
    }
}

fn fatal_result(spec: &ImportSourceSpec, diagnostic: Diagnostic) -> ImportResult {
    ImportResult {
        source: make_source(defaults(spec, None, BTreeMap::new())),
        comments: vec![],
        diagnostics: vec![diagnostic],
        skipped_count: 0,
        importer_version: JSONL_IMPORTER_VERSION.into(),
        detected_format: "jsonl_v1".into(),
        detected_encoding: Some("utf-8".into()),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsonlComment {
    id: Option<String>,
    timestamp_ms: Option<i64>,
    timestamp: Option<String>,
    platform: Option<String>,
    kind: Option<String>,
    author: Option<CommentAuthor>,
    body: Option<Value>,
    style: Option<CommentStyle>,
    badges: Option<Vec<Badge>>,
    emotes: Option<Vec<Emote>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::ImportFormat;

    fn spec() -> ImportSourceSpec {
        ImportSourceSpec {
            path: "comments.jsonl".into(),
            format: ImportFormat::Jsonl,
            source_id: "jsonl_main".into(),
            display_name: Some("JSONL Main".into()),
            platform: Some("twitch".into()),
            enabled: true,
            offset_ms: 0,
            csv_mapping: None,
            metadata: BTreeMap::new(),
            timestamp_basis: None,
        }
    }

    #[test]
    fn imports_jsonl_happy_path() {
        let result = import_jsonl_str(
            r#"{"id":"a","timestampMs":1000,"author":{"displayName":"Livie"},"body":{"text":"hello"}}"#
                .to_owned()
                .as_str(),
            &spec(),
        );
        assert_eq!(result.comments.len(), 1);
        assert_eq!(result.comments[0].source_id, "jsonl_main");
        assert_eq!(result.comments[0].platform.as_deref(), Some("twitch"));
    }

    #[test]
    fn valid_final_line_without_newline_is_accepted() {
        let result = import_jsonl_str(r#"{"id":"a","timestamp":"1.5","body":"hello"}"#, &spec());
        assert_eq!(result.comments.len(), 1);
        assert_eq!(result.comments[0].timestamp_ms, 1500);
    }

    #[test]
    fn skips_malformed_and_invalid_rows() {
        let result = import_jsonl_str(
            concat!(
                "{\"id\":\"a\",\"timestampMs\":1000,\"body\":\"ok\"}\n",
                "{\"id\":\"b\",\"timestampMs\":\n",
                "{\"id\":\"a\",\"timestampMs\":2000,\"body\":\"dupe\"}\n",
                "{\"id\":\"c\",\"timestamp\":\"nope\",\"body\":\"bad\"}\n"
            ),
            &spec(),
        );
        assert_eq!(result.comments.len(), 1);
        assert_eq!(result.skipped_count, 3);
        assert!(result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "import.malformed_jsonl_row"));
    }

    #[test]
    fn imports_generated_large_jsonl_stream() {
        let mut contents = String::new();
        for index in 0..1_000 {
            contents.push_str(&format!(
                "{{\"id\":\"c{index}\",\"timestampMs\":{index},\"body\":\"row {index}\"}}\n"
            ));
        }
        let result = import_jsonl_str(&contents, &spec());
        assert_eq!(result.comments.len(), 1_000);
        assert_eq!(result.skipped_count, 0);
    }
}
