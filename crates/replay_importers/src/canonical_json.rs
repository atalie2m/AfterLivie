use replay_core::{
    parse_timestamp_ms, AssetRef, Badge, CommentAuthor, CommentBody, CommentEvent, CommentKind,
    CommentSource, CommentSourceRef, CommentStyle, Diagnostic, DiagnosticCategory,
    DiagnosticSeverity, Emote, ImportFingerprint, ImportResult, OriginalTimestamp, RawSourceRef,
    SourceRef, TimestampBasis,
};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

pub const CANONICAL_JSON_IMPORTER_ID: &str = "canonical-json";
pub const CANONICAL_JSON_IMPORTER_VERSION: &str = "1.0.0";

pub fn import_canonical_json(path: impl AsRef<Path>) -> ImportResult {
    let path = path.as_ref();
    match fs::read_to_string(path) {
        Ok(contents) => import_canonical_json_str(&contents, path.to_string_lossy().as_ref()),
        Err(error) => fatal_result(
            path.to_string_lossy().as_ref(),
            Diagnostic::new(
                DiagnosticSeverity::Fatal,
                DiagnosticCategory::Import,
                "import.read_failed",
                format!("Could not read comment file: {error}"),
            )
            .with_source_ref(SourceRef::File {
                path: path.to_string_lossy().into_owned(),
            })
            .with_hint("Check that the file still exists and is readable."),
        ),
    }
}

pub fn import_canonical_json_str(contents: &str, source_path: &str) -> ImportResult {
    let fingerprint = fingerprint(contents.as_bytes());
    let parsed = serde_json::from_str::<CanonicalFile>(contents);
    let file = match parsed {
        Ok(file) => file,
        Err(error) => {
            return fatal_result(
                source_path,
                Diagnostic::new(
                    DiagnosticSeverity::Fatal,
                    DiagnosticCategory::Import,
                    "import.malformed_json",
                    format!("Comment file is not valid canonical JSON: {error}"),
                )
                .with_source_ref(SourceRef::File {
                    path: source_path.into(),
                })
                .with_hint("Validate the JSON file and try importing it again."),
            );
        }
    };

    let source_id = file
        .source
        .as_ref()
        .and_then(|source| source.source_id.clone())
        .unwrap_or_else(|| "main".into());
    let source = CommentSource {
        source_id: source_id.clone(),
        display_name: file
            .source
            .as_ref()
            .and_then(|source| source.display_name.clone())
            .unwrap_or_else(|| "Main Chat".into()),
        platform: file
            .source
            .as_ref()
            .and_then(|source| source.platform.clone()),
        importer_id: CANONICAL_JSON_IMPORTER_ID.into(),
        original_file_ref: AssetRef {
            path: source_path.into(),
            hash: Some(fingerprint.clone()),
        },
        enabled: true,
        offset_ms: 0,
        timestamp_basis: TimestampBasis::RelativeToVideoStart,
        diagnostics_ref: None,
        visual_style: None,
        metadata: file
            .source
            .as_ref()
            .and_then(|source| source.metadata.clone())
            .unwrap_or_default(),
        import_fingerprint: ImportFingerprint {
            algorithm: "sha256".into(),
            value: fingerprint,
        },
    };

    let mut diagnostics = Vec::new();
    let mut comments = Vec::new();
    let mut seen_ids = BTreeSet::new();
    let mut skipped_count = 0_u64;

    for (index, row) in file.comments.into_iter().enumerate() {
        let row_number = index as u64 + 1;
        match normalize_row(row, &source_id, source_path, row_number) {
            Ok(comment) => {
                if !seen_ids.insert(comment.id.clone()) {
                    skipped_count += 1;
                    diagnostics.push(
                        Diagnostic::new(
                            DiagnosticSeverity::Error,
                            DiagnosticCategory::Import,
                            "import.duplicate_comment_id",
                            format!(
                                "Duplicate comment id '{}' in source '{}'.",
                                comment.id, source_id
                            ),
                        )
                        .with_source_ref(SourceRef::Row {
                            path: Some(source_path.into()),
                            row: row_number,
                        })
                        .with_hint("Ensure each comment id is unique within the source."),
                    );
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
        source,
        comments,
        diagnostics,
        skipped_count,
        importer_version: CANONICAL_JSON_IMPORTER_VERSION.into(),
        detected_format: "canonical_json_v1".into(),
        detected_encoding: Some("utf-8".into()),
    }
}

fn normalize_row(
    row: CanonicalComment,
    source_id: &str,
    source_path: &str,
    row_number: u64,
) -> Result<CommentEvent, Box<Diagnostic>> {
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
                    path: Some(source_path.into()),
                    row: row_number,
                })
                .with_hint("Add timestampMs as integer milliseconds relative to video start."),
            )
        })?;
    let timestamp_ms = parse_timestamp_ms(&timestamp_value).map_err(|error| {
        Box::new(
            Diagnostic::new(
                DiagnosticSeverity::Error,
                DiagnosticCategory::Timestamp,
                "import.invalid_timestamp",
                format!("Timestamp '{timestamp_value}' is invalid: {error}."),
            )
            .with_source_ref(SourceRef::Timestamp {
                value: timestamp_value.clone(),
            })
            .with_hint("Use integer milliseconds, seconds, MM:SS, or HH:MM:SS."),
        )
    })?;

    let body_text = row.body.and_then(parse_body).ok_or_else(|| {
        Box::new(
            Diagnostic::new(
                DiagnosticSeverity::Error,
                DiagnosticCategory::Import,
                "import.missing_body",
                "Comment row is missing body.text.",
            )
            .with_source_ref(SourceRef::Field {
                path: Some(source_path.into()),
                row: Some(row_number),
                field: "body.text".into(),
            })
            .with_hint("Provide a non-empty comment body."),
        )
    })?;

    let id = row.id.unwrap_or_else(|| {
        generated_comment_id(
            source_id,
            row_number,
            timestamp_ms,
            row.author
                .as_ref()
                .map(|author| author.display_name.as_str())
                .unwrap_or_default(),
            &body_text,
        )
    });

    Ok(CommentEvent {
        id,
        source_id: source_id.into(),
        platform: row.platform,
        timestamp_ms,
        original_timestamp: Some(OriginalTimestamp {
            value: timestamp_value,
            basis: TimestampBasis::RelativeToVideoStart,
        }),
        kind: row.kind.unwrap_or_default(),
        author: row.author,
        body: CommentBody { text: body_text },
        style: row.style,
        badges: row.badges.unwrap_or_default(),
        emotes: row.emotes.unwrap_or_default(),
        source: Some(CommentSourceRef {
            path: Some(source_path.into()),
            row: Some(row_number),
        }),
        raw_ref: Some(RawSourceRef {
            row: Some(row_number),
            pointer: None,
        }),
        import_order: row_number - 1,
    })
}

fn parse_body(value: Value) -> Option<String> {
    match value {
        Value::String(text) => non_empty(text),
        Value::Object(mut object) => object
            .remove("text")
            .and_then(|value| value.as_str().map(ToOwned::to_owned))
            .and_then(non_empty),
        _ => None,
    }
}

fn non_empty(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.into())
    }
}

fn generated_comment_id(
    source_id: &str,
    row_number: u64,
    timestamp_ms: i64,
    author: &str,
    body: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(CANONICAL_JSON_IMPORTER_ID.as_bytes());
    hasher.update(CANONICAL_JSON_IMPORTER_VERSION.as_bytes());
    hasher.update(source_id.as_bytes());
    hasher.update(row_number.to_le_bytes());
    hasher.update(timestamp_ms.to_le_bytes());
    hasher.update(author.as_bytes());
    hasher.update(body.as_bytes());
    let hash = hasher.finalize();
    let suffix = hash[..8]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("generated_{suffix}")
}

fn fingerprint(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let hash = hasher.finalize();
    hash.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn fatal_result(source_path: &str, diagnostic: Diagnostic) -> ImportResult {
    CommentSource {
        source_id: "main".into(),
        display_name: "Main Chat".into(),
        platform: None,
        importer_id: CANONICAL_JSON_IMPORTER_ID.into(),
        original_file_ref: AssetRef {
            path: source_path.into(),
            hash: None,
        },
        enabled: true,
        offset_ms: 0,
        timestamp_basis: TimestampBasis::Unknown,
        diagnostics_ref: None,
        visual_style: None,
        metadata: Default::default(),
        import_fingerprint: ImportFingerprint {
            algorithm: "sha256".into(),
            value: String::new(),
        },
    }
    .into_import_result(vec![diagnostic])
}

trait IntoFatalImportResult {
    fn into_import_result(self, diagnostics: Vec<Diagnostic>) -> ImportResult;
}

impl IntoFatalImportResult for CommentSource {
    fn into_import_result(self, diagnostics: Vec<Diagnostic>) -> ImportResult {
        ImportResult {
            source: self,
            comments: vec![],
            skipped_count: 0,
            importer_version: CANONICAL_JSON_IMPORTER_VERSION.into(),
            detected_format: "canonical_json_v1".into(),
            detected_encoding: Some("utf-8".into()),
            diagnostics,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CanonicalFile {
    source: Option<CanonicalSource>,
    comments: Vec<CanonicalComment>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CanonicalSource {
    source_id: Option<String>,
    display_name: Option<String>,
    platform: Option<String>,
    metadata: Option<std::collections::BTreeMap<String, String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CanonicalComment {
    id: Option<String>,
    platform: Option<String>,
    timestamp_ms: Option<i64>,
    timestamp: Option<String>,
    kind: Option<CommentKind>,
    author: Option<CommentAuthor>,
    body: Option<Value>,
    style: Option<CommentStyle>,
    badges: Option<Vec<Badge>>,
    emotes: Option<Vec<Emote>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imports_happy_path() {
        let json = r##"{
          "source": { "sourceId": "main", "displayName": "Main Chat", "platform": "youtube" },
          "comments": [
            { "id": "c1", "timestampMs": 1000, "author": { "displayName": "Livie" }, "body": { "text": "hello" } },
            { "timestamp": "00:02", "body": "second" }
          ]
        }"##;
        let result = import_canonical_json_str(json, "fixture.json");
        assert_eq!(result.comments.len(), 2);
        assert_eq!(result.skipped_count, 0);
        assert_eq!(result.comments[1].timestamp_ms, 2000);
        assert!(result.comments[1].id.starts_with("generated_"));
    }

    #[test]
    fn reports_invalid_rows_and_duplicates() {
        let json = r##"{
          "comments": [
            { "id": "dupe", "timestampMs": 1000, "body": "hello" },
            { "id": "dupe", "timestampMs": 1200, "body": "again" },
            { "timestamp": "nope", "body": "bad" },
            { "timestampMs": 1500, "body": "" }
          ]
        }"##;
        let result = import_canonical_json_str(json, "fixture.json");
        assert_eq!(result.comments.len(), 1);
        assert_eq!(result.skipped_count, 3);
        assert_eq!(result.diagnostics.len(), 3);
    }

    #[test]
    fn malformed_json_is_fatal() {
        let result = import_canonical_json_str("{", "bad.json");
        assert!(result.comments.is_empty());
        assert_eq!(result.diagnostics[0].severity, DiagnosticSeverity::Fatal);
    }
}
