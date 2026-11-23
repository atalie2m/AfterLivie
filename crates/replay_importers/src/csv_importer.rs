use crate::common::{
    duplicate_id_diagnostic, fingerprint_path, make_source, normalize_comment, parse_kind_value,
    read_failed_diagnostic, CommentInput, ImporterResult, SourceDefaults,
};
use crate::source::{
    CsvColumnMapping, ImportFormat, ImportPreview, ImportPreviewRow, ImportSourceSpec,
};
use replay_core::{
    CommentEvent, Diagnostic, DiagnosticCategory, DiagnosticSeverity, ImportResult, SourceRef,
    TimestampBasis,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub const CSV_IMPORTER_ID: &str = "csv";
pub const CSV_IMPORTER_VERSION: &str = "1.0.0";

pub fn import_csv(spec: &ImportSourceSpec) -> ImportResult {
    let path = Path::new(&spec.path);
    let fingerprint = match fingerprint_path(path) {
        Ok(fingerprint) => fingerprint,
        Err(diagnostic) => return fatal_result(spec, *diagnostic),
    };
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) => return fatal_result(spec, read_failed_diagnostic(&spec.path, error)),
    };
    read_csv(file, spec, fingerprint, None).result
}

pub fn preview_csv(spec: &ImportSourceSpec, max_rows: usize) -> ImportPreview {
    let path = Path::new(&spec.path);
    let fingerprint = match fingerprint_path(path) {
        Ok(fingerprint) => fingerprint,
        Err(diagnostic) => {
            return ImportPreview {
                source_id: spec.source_id.clone(),
                format: ImportFormat::Csv,
                importer_id: CSV_IMPORTER_ID.into(),
                importer_version: CSV_IMPORTER_VERSION.into(),
                detected_encoding: Some("utf-8".into()),
                headers: vec![],
                csv_mapping: spec.csv_mapping.clone(),
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
                format: ImportFormat::Csv,
                importer_id: CSV_IMPORTER_ID.into(),
                importer_version: CSV_IMPORTER_VERSION.into(),
                detected_encoding: Some("utf-8".into()),
                headers: vec![],
                csv_mapping: spec.csv_mapping.clone(),
                rows: vec![],
                diagnostics: vec![read_failed_diagnostic(&spec.path, error)],
                skipped_count: 0,
            };
        }
    };
    let read = read_csv(file, spec, fingerprint, Some(max_rows));
    ImportPreview {
        source_id: read.result.source.source_id,
        format: ImportFormat::Csv,
        importer_id: CSV_IMPORTER_ID.into(),
        importer_version: CSV_IMPORTER_VERSION.into(),
        detected_encoding: read.result.detected_encoding,
        headers: read.headers,
        csv_mapping: read.mapping,
        rows: read
            .result
            .comments
            .iter()
            .map(ImportPreviewRow::from)
            .collect(),
        diagnostics: read.result.diagnostics,
        skipped_count: read.result.skipped_count,
    }
}

pub fn auto_csv_mapping(headers: &[String]) -> CsvColumnMapping {
    CsvColumnMapping {
        timestamp: find_known_header(
            headers,
            &["timestampMs", "timestamp_ms", "timestamp", "time"],
        )
        .unwrap_or_default(),
        text: find_known_header(headers, &["text", "body", "message", "comment"])
            .unwrap_or_default(),
        author: find_known_header(headers, &["author", "authorName", "displayName"]),
        id: find_known_header(headers, &["id", "commentId"]),
        kind: find_known_header(headers, &["kind", "type"]),
        platform: find_known_header(headers, &["platform"]),
        metadata: BTreeMap::new(),
    }
}

#[cfg(test)]
pub(crate) fn import_csv_str(contents: &str, spec: &ImportSourceSpec) -> ImportResult {
    read_csv(
        contents.as_bytes(),
        spec,
        crate::common::fingerprint_bytes(contents.as_bytes()),
        None,
    )
    .result
}

struct CsvReadResult {
    result: ImportResult,
    headers: Vec<String>,
    mapping: Option<CsvColumnMapping>,
}

fn read_csv<R: Read>(
    reader: R,
    spec: &ImportSourceSpec,
    fingerprint: String,
    max_rows: Option<usize>,
) -> CsvReadResult {
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(reader);
    let headers = match reader.headers() {
        Ok(headers) => headers.iter().map(strip_bom).collect::<Vec<_>>(),
        Err(error) => {
            let diagnostic = csv_error_diagnostic(&error, &spec.path, None);
            return CsvReadResult {
                result: fatal_result(spec, diagnostic),
                headers: vec![],
                mapping: spec.csv_mapping.clone(),
            };
        }
    };
    let mapping = resolved_mapping(spec.csv_mapping.clone(), &headers);
    let indices = match ColumnIndices::resolve(&headers, &mapping, &spec.path) {
        Ok(indices) => indices,
        Err(diagnostics) => {
            return CsvReadResult {
                result: ImportResult {
                    source: make_source(defaults(spec, Some(fingerprint), BTreeMap::new())),
                    comments: vec![],
                    diagnostics,
                    skipped_count: 0,
                    importer_version: CSV_IMPORTER_VERSION.into(),
                    detected_format: "csv_v1".into(),
                    detected_encoding: Some("utf-8".into()),
                },
                headers,
                mapping: Some(mapping),
            };
        }
    };

    let mut comments = Vec::new();
    let mut diagnostics = Vec::new();
    let mut skipped_count = 0;
    let mut seen_ids = BTreeSet::new();
    let mut source_metadata = BTreeMap::new();

    for (index, record) in reader.records().enumerate() {
        if max_rows.is_some_and(|max_rows| comments.len() >= max_rows) {
            break;
        }
        let fallback_row = index as u64 + 2;
        let record = match record {
            Ok(record) => record,
            Err(error) => {
                skipped_count += 1;
                diagnostics.push(csv_error_diagnostic(&error, &spec.path, Some(fallback_row)));
                continue;
            }
        };
        let row_number = fallback_row;
        for (metadata_key, index) in &indices.metadata {
            if !source_metadata.contains_key(metadata_key) {
                if let Some(value) = value_at(&record, *index) {
                    source_metadata.insert(metadata_key.clone(), value);
                }
            }
        }
        match normalize_record(&record, &indices, spec, row_number) {
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

    CsvReadResult {
        result: ImportResult {
            source: make_source(defaults(spec, Some(fingerprint), source_metadata)),
            comments,
            diagnostics,
            skipped_count,
            importer_version: CSV_IMPORTER_VERSION.into(),
            detected_format: "csv_v1".into(),
            detected_encoding: Some("utf-8".into()),
        },
        headers,
        mapping: Some(mapping),
    }
}

fn normalize_record(
    record: &csv::StringRecord,
    indices: &ColumnIndices,
    spec: &ImportSourceSpec,
    row_number: u64,
) -> ImporterResult<CommentEvent> {
    let timestamp_value = required_value(
        record,
        indices.timestamp,
        "timestamp",
        &spec.path,
        row_number,
    )?;
    let body_text = required_value(record, indices.text, "text", &spec.path, row_number)?;
    let kind = parse_kind_value(
        indices.kind.and_then(|index| value_at(record, index)),
        &spec.path,
        row_number,
    )?;
    normalize_comment(CommentInput {
        id: indices.id.and_then(|index| value_at(record, index)),
        source_id: spec.source_id.clone(),
        platform: indices
            .platform
            .and_then(|index| value_at(record, index))
            .or_else(|| spec.platform.clone()),
        timestamp_value,
        kind,
        author_display_name: indices.author.and_then(|index| value_at(record, index)),
        body_text,
        source_path: spec.path.clone(),
        row_number,
        import_order: row_number.saturating_sub(2),
        importer_id: CSV_IMPORTER_ID,
        importer_version: CSV_IMPORTER_VERSION,
    })
}

#[derive(Debug, Clone)]
struct ColumnIndices {
    timestamp: usize,
    text: usize,
    id: Option<usize>,
    author: Option<usize>,
    kind: Option<usize>,
    platform: Option<usize>,
    metadata: Vec<(String, usize)>,
}

impl ColumnIndices {
    fn resolve(
        headers: &[String],
        mapping: &CsvColumnMapping,
        source_path: &str,
    ) -> Result<Self, Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        let timestamp =
            match resolve_required(headers, &mapping.timestamp, "timestamp", source_path) {
                Ok(index) => Some(index),
                Err(diagnostic) => {
                    diagnostics.push(*diagnostic);
                    None
                }
            };
        let text = match resolve_required(headers, &mapping.text, "text", source_path) {
            Ok(index) => Some(index),
            Err(diagnostic) => {
                diagnostics.push(*diagnostic);
                None
            }
        };
        let metadata = mapping
            .metadata
            .iter()
            .filter_map(|(key, column)| {
                resolve_optional(headers, column)
                    .map(|index| (key.clone(), index))
                    .or_else(|| {
                        diagnostics.push(missing_mapping_diagnostic(
                            source_path,
                            &format!("csvMapping.metadata.{key}"),
                            column,
                        ));
                        None
                    })
            })
            .collect();

        if diagnostics.is_empty() {
            Ok(Self {
                timestamp: timestamp.expect("checked above"),
                text: text.expect("checked above"),
                id: mapping
                    .id
                    .as_deref()
                    .and_then(|column| resolve_optional(headers, column)),
                author: mapping
                    .author
                    .as_deref()
                    .and_then(|column| resolve_optional(headers, column)),
                kind: mapping
                    .kind
                    .as_deref()
                    .and_then(|column| resolve_optional(headers, column)),
                platform: mapping
                    .platform
                    .as_deref()
                    .and_then(|column| resolve_optional(headers, column)),
                metadata,
            })
        } else {
            Err(diagnostics)
        }
    }
}

fn resolve_required(
    headers: &[String],
    column: &str,
    field: &str,
    source_path: &str,
) -> ImporterResult<usize> {
    if column.trim().is_empty() {
        return Err(Box::new(
            Diagnostic::new(
                DiagnosticSeverity::Fatal,
                DiagnosticCategory::Import,
                "import.missing_column_mapping",
                format!("CSV import is missing a {field} column mapping."),
            )
            .with_source_ref(SourceRef::Field {
                path: Some(source_path.into()),
                row: None,
                field: format!("csvMapping.{field}"),
            })
            .with_hint("Map the required CSV column before importing."),
        ));
    }
    resolve_optional(headers, column).ok_or_else(|| {
        Box::new(missing_mapping_diagnostic(
            source_path,
            &format!("csvMapping.{field}"),
            column,
        ))
    })
}

fn resolve_optional(headers: &[String], column: &str) -> Option<usize> {
    if column.trim().is_empty() {
        return None;
    }
    let normalized = normalize_header(column);
    headers
        .iter()
        .position(|header| header == column || normalize_header(header) == normalized)
}

fn resolved_mapping(mapping: Option<CsvColumnMapping>, headers: &[String]) -> CsvColumnMapping {
    let auto = auto_csv_mapping(headers);
    let Some(mut mapping) = mapping else {
        return auto;
    };
    if mapping.timestamp.trim().is_empty() {
        mapping.timestamp = auto.timestamp;
    }
    if mapping.text.trim().is_empty() {
        mapping.text = auto.text;
    }
    if mapping.id.is_none() {
        mapping.id = auto.id;
    }
    if mapping.author.is_none() {
        mapping.author = auto.author;
    }
    if mapping.kind.is_none() {
        mapping.kind = auto.kind;
    }
    if mapping.platform.is_none() {
        mapping.platform = auto.platform;
    }
    mapping
}

fn required_value(
    record: &csv::StringRecord,
    index: usize,
    field: &str,
    source_path: &str,
    row_number: u64,
) -> ImporterResult<String> {
    value_at(record, index).ok_or_else(|| {
        Box::new(
            Diagnostic::new(
                DiagnosticSeverity::Error,
                DiagnosticCategory::Import,
                "import.missing_field",
                format!("CSV row is missing {field}."),
            )
            .with_source_ref(SourceRef::Field {
                path: Some(source_path.into()),
                row: Some(row_number),
                field: field.into(),
            })
            .with_hint("Fill the required field or remove the row."),
        )
    })
}

fn value_at(record: &csv::StringRecord, index: usize) -> Option<String> {
    record
        .get(index)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn csv_error_diagnostic(
    error: &csv::Error,
    source_path: &str,
    fallback_row: Option<u64>,
) -> Diagnostic {
    let row = error
        .position()
        .map(|position| position.line())
        .or(fallback_row);
    let (severity, code, hint) = if matches!(error.kind(), csv::ErrorKind::Utf8 { .. }) {
        (
            DiagnosticSeverity::Fatal,
            "import.invalid_encoding",
            "Save the CSV as UTF-8 and try importing it again.",
        )
    } else {
        (
            DiagnosticSeverity::Error,
            "import.malformed_csv_row",
            "Fix or remove the malformed CSV row and import again.",
        )
    };
    Diagnostic::new(
        severity,
        DiagnosticCategory::Import,
        code,
        format!("Could not parse CSV row: {error}"),
    )
    .with_source_ref(SourceRef::Row {
        path: Some(source_path.into()),
        row: row.unwrap_or(0),
    })
    .with_hint(hint)
}

fn missing_mapping_diagnostic(source_path: &str, field: &str, column: &str) -> Diagnostic {
    Diagnostic::new(
        DiagnosticSeverity::Fatal,
        DiagnosticCategory::Import,
        "import.mapped_column_missing",
        format!("Mapped CSV column '{column}' does not exist."),
    )
    .with_source_ref(SourceRef::Field {
        path: Some(source_path.into()),
        row: None,
        field: field.into(),
    })
    .with_hint("Choose one of the CSV headers reported by import preview.")
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
        importer_id: CSV_IMPORTER_ID,
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
        importer_version: CSV_IMPORTER_VERSION.into(),
        detected_format: "csv_v1".into(),
        detected_encoding: Some("utf-8".into()),
    }
}

fn find_known_header(headers: &[String], candidates: &[&str]) -> Option<String> {
    candidates.iter().find_map(|candidate| {
        headers
            .iter()
            .find(|header| normalize_header(header) == normalize_header(candidate))
            .cloned()
    })
}

fn normalize_header(value: &str) -> String {
    strip_bom(value)
        .chars()
        .filter(|character| *character != '_' && *character != '-' && !character.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase()
}

fn strip_bom(value: &str) -> String {
    value.trim_start_matches('\u{feff}').to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::ImportFormat;

    fn spec(mapping: Option<CsvColumnMapping>) -> ImportSourceSpec {
        ImportSourceSpec {
            path: "comments.csv".into(),
            format: ImportFormat::Csv,
            source_id: "csv_main".into(),
            display_name: Some("CSV Main".into()),
            platform: Some("youtube".into()),
            enabled: true,
            offset_ms: 0,
            csv_mapping: mapping,
            metadata: BTreeMap::new(),
            timestamp_basis: None,
        }
    }

    #[test]
    fn imports_csv_happy_path_with_auto_mapping() {
        let result = import_csv_str(
            "timestamp,text,author,id\n1.5,hello,Livie,c1\n00:03,world,Ada,c2\n",
            &spec(None),
        );
        assert_eq!(result.comments.len(), 2);
        assert_eq!(result.comments[0].timestamp_ms, 1500);
        assert_eq!(
            result.comments[0].author.as_ref().unwrap().display_name,
            "Livie"
        );
        assert_eq!(result.skipped_count, 0);
    }

    #[test]
    fn empty_mapping_fields_fall_back_to_auto_mapping() {
        let result = import_csv_str(
            "timestamp,text,author,id\n1,hello,Livie,c1\n",
            &spec(Some(CsvColumnMapping::default())),
        );
        assert_eq!(result.comments.len(), 1);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn mapping_failures_are_fatal() {
        let result = import_csv_str(
            "time,message\n1,hello\n",
            &spec(Some(CsvColumnMapping {
                timestamp: "missing".into(),
                text: "message".into(),
                ..Default::default()
            })),
        );
        assert!(result.comments.is_empty());
        assert_eq!(result.diagnostics[0].severity, DiagnosticSeverity::Fatal);
    }

    #[test]
    fn skips_invalid_rows_and_duplicates() {
        let result = import_csv_str(
            "timestamp,text,id\n1,ok,c1\nnope,bad,c2\n2,again,c1\n3,,c3\n",
            &spec(None),
        );
        assert_eq!(result.comments.len(), 1);
        assert_eq!(result.skipped_count, 3);
        assert!(result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "import.duplicate_comment_id"));
    }

    #[test]
    fn invalid_encoding_is_diagnostic() {
        let result = read_csv(
            &b"timestamp,text\n1,\xff\n"[..],
            &spec(None),
            "hash".into(),
            None,
        )
        .result;
        assert!(result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "import.invalid_encoding"));
    }

    #[test]
    fn huge_text_field_is_skipped() {
        let body = "x".repeat(crate::common::MAX_BODY_TEXT_CHARS + 1);
        let result = import_csv_str(&format!("timestamp,text\n1,{body}\n"), &spec(None));
        assert!(result.comments.is_empty());
        assert_eq!(result.skipped_count, 1);
        assert_eq!(result.diagnostics[0].code, "import.body_too_large");
    }
}
