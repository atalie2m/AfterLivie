use replay_core::{
    parse_timestamp_ms, AssetRef, CommentBody, CommentEvent, CommentKind, CommentSource,
    CommentSourceRef, Diagnostic, DiagnosticCategory, DiagnosticSeverity, ImportFingerprint,
    OriginalTimestamp, RawSourceRef, SourceRef, TimestampBasis,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub(crate) const MAX_BODY_TEXT_CHARS: usize = 32_768;
pub(crate) type ImporterResult<T> = Result<T, Box<Diagnostic>>;

#[derive(Debug, Clone)]
pub(crate) struct SourceDefaults<'a> {
    pub source_id: &'a str,
    pub display_name: &'a str,
    pub platform: Option<&'a str>,
    pub enabled: bool,
    pub offset_ms: i64,
    pub source_path: &'a str,
    pub importer_id: &'a str,
    pub fingerprint: Option<String>,
    pub timestamp_basis: TimestampBasis,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub(crate) struct CommentInput {
    pub id: Option<String>,
    pub source_id: String,
    pub platform: Option<String>,
    pub timestamp_value: String,
    pub kind: CommentKind,
    pub author_display_name: Option<String>,
    pub body_text: String,
    pub source_path: String,
    pub row_number: u64,
    pub import_order: u64,
    pub importer_id: &'static str,
    pub importer_version: &'static str,
}

pub(crate) fn make_source(defaults: SourceDefaults<'_>) -> CommentSource {
    CommentSource {
        source_id: defaults.source_id.into(),
        display_name: defaults.display_name.into(),
        platform: defaults.platform.map(str::to_owned),
        importer_id: defaults.importer_id.into(),
        original_file_ref: AssetRef {
            path: defaults.source_path.into(),
            hash: defaults.fingerprint.clone(),
        },
        enabled: defaults.enabled,
        offset_ms: defaults.offset_ms,
        timestamp_basis: defaults.timestamp_basis,
        diagnostics_ref: None,
        visual_style: None,
        metadata: defaults.metadata,
        import_fingerprint: ImportFingerprint {
            algorithm: "sha256".into(),
            value: defaults.fingerprint.unwrap_or_default(),
        },
    }
}

pub(crate) fn normalize_comment(input: CommentInput) -> ImporterResult<CommentEvent> {
    let timestamp_ms = parse_timestamp_ms(&input.timestamp_value).map_err(|error| {
        Box::new(
            Diagnostic::new(
                DiagnosticSeverity::Error,
                DiagnosticCategory::Timestamp,
                "import.invalid_timestamp",
                format!("Timestamp '{}' is invalid: {error}.", input.timestamp_value),
            )
            .with_source_ref(SourceRef::Field {
                path: Some(input.source_path.clone()),
                row: Some(input.row_number),
                field: "timestamp".into(),
            })
            .with_hint("Use integer milliseconds, seconds, MM:SS, or HH:MM:SS."),
        )
    })?;

    let body_text = validate_body_text(input.body_text, &input.source_path, input.row_number)?;
    let author = input
        .author_display_name
        .and_then(non_empty)
        .map(|display_name| replay_core::CommentAuthor {
            id: None,
            display_name,
            color: None,
        });
    let author_name = author
        .as_ref()
        .map(|author| author.display_name.as_str())
        .unwrap_or_default();
    let id = input.id.and_then(non_empty).unwrap_or_else(|| {
        generated_comment_id(
            input.importer_id,
            input.importer_version,
            &input.source_id,
            input.row_number,
            timestamp_ms,
            author_name,
            &body_text,
        )
    });

    Ok(CommentEvent {
        id,
        source_id: input.source_id,
        platform: input.platform,
        timestamp_ms,
        original_timestamp: Some(OriginalTimestamp {
            value: input.timestamp_value,
            basis: TimestampBasis::RelativeToVideoStart,
        }),
        kind: input.kind,
        author,
        body: CommentBody { text: body_text },
        style: None,
        badges: vec![],
        emotes: vec![],
        source: Some(CommentSourceRef {
            path: Some(input.source_path.clone()),
            row: Some(input.row_number),
        }),
        raw_ref: Some(RawSourceRef {
            row: Some(input.row_number),
            pointer: None,
        }),
        import_order: input.import_order,
    })
}

pub(crate) fn parse_body_value(
    value: Option<Value>,
    source_path: &str,
    row_number: u64,
) -> ImporterResult<String> {
    let text = match value {
        Some(Value::String(text)) => Some(text),
        Some(Value::Object(mut object)) => object
            .remove("text")
            .and_then(|value| value.as_str().map(ToOwned::to_owned)),
        _ => None,
    }
    .and_then(non_empty)
    .ok_or_else(|| {
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
    validate_body_text(text, source_path, row_number)
}

pub(crate) fn parse_kind_value(
    value: Option<String>,
    source_path: &str,
    row_number: u64,
) -> ImporterResult<CommentKind> {
    let Some(value) = value.and_then(non_empty) else {
        return Ok(CommentKind::Message);
    };
    let normalized = value.trim().to_ascii_lowercase().replace(['-', ' '], "_");
    serde_json::from_value::<CommentKind>(Value::String(normalized.clone())).map_err(|_| {
        Box::new(
            Diagnostic::new(
                DiagnosticSeverity::Error,
                DiagnosticCategory::Import,
                "import.invalid_kind",
                format!("Comment kind '{value}' is not supported."),
            )
            .with_source_ref(SourceRef::Field {
                path: Some(source_path.into()),
                row: Some(row_number),
                field: "kind".into(),
            })
            .with_hint("Use message, pinned, metadata, reaction, or system."),
        )
    })
}

pub(crate) fn non_empty(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.into())
    }
}

pub(crate) fn duplicate_id_diagnostic(
    source_path: &str,
    source_id: &str,
    comment_id: &str,
    row_number: u64,
) -> Diagnostic {
    Diagnostic::new(
        DiagnosticSeverity::Error,
        DiagnosticCategory::Import,
        "import.duplicate_comment_id",
        format!("Duplicate comment id '{comment_id}' in source '{source_id}'."),
    )
    .with_source_ref(SourceRef::Row {
        path: Some(source_path.into()),
        row: row_number,
    })
    .with_hint("Ensure each comment id is unique within the source.")
}

pub(crate) fn read_failed_diagnostic(
    source_path: &str,
    error: impl std::fmt::Display,
) -> Diagnostic {
    Diagnostic::new(
        DiagnosticSeverity::Fatal,
        DiagnosticCategory::Import,
        "import.read_failed",
        format!("Could not read comment file: {error}"),
    )
    .with_source_ref(SourceRef::File {
        path: source_path.into(),
    })
    .with_hint("Check that the file still exists and is readable.")
}

pub(crate) fn fingerprint_path(path: &Path) -> ImporterResult<String> {
    let mut file = File::open(path)
        .map_err(|error| Box::new(read_failed_diagnostic(&path.to_string_lossy(), error)))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|error| Box::new(read_failed_diagnostic(&path.to_string_lossy(), error)))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(hexish(&hasher.finalize()))
}

#[cfg(test)]
pub(crate) fn fingerprint_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hexish(&hasher.finalize())
}

fn validate_body_text(text: String, source_path: &str, row_number: u64) -> ImporterResult<String> {
    let text = non_empty(text).ok_or_else(|| {
        Box::new(
            Diagnostic::new(
                DiagnosticSeverity::Error,
                DiagnosticCategory::Import,
                "import.missing_body",
                "Comment row has an empty body.",
            )
            .with_source_ref(SourceRef::Field {
                path: Some(source_path.into()),
                row: Some(row_number),
                field: "body.text".into(),
            })
            .with_hint("Provide a non-empty comment body."),
        )
    })?;
    if text.chars().count() > MAX_BODY_TEXT_CHARS {
        return Err(Box::new(
            Diagnostic::new(
                DiagnosticSeverity::Error,
                DiagnosticCategory::Import,
                "import.body_too_large",
                format!("Comment body exceeds {MAX_BODY_TEXT_CHARS} characters."),
            )
            .with_source_ref(SourceRef::Field {
                path: Some(source_path.into()),
                row: Some(row_number),
                field: "body.text".into(),
            })
            .with_hint("Shorten the row or split it into separate comments."),
        ));
    }
    Ok(text)
}

fn generated_comment_id(
    importer_id: &str,
    importer_version: &str,
    source_id: &str,
    row_number: u64,
    timestamp_ms: i64,
    author: &str,
    body: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(importer_id.as_bytes());
    hasher.update(importer_version.as_bytes());
    hasher.update(source_id.as_bytes());
    hasher.update(row_number.to_le_bytes());
    hasher.update(timestamp_ms.to_le_bytes());
    hasher.update(author.as_bytes());
    hasher.update(body.as_bytes());
    let hash = hasher.finalize();
    format!("generated_{}", &hexish(&hash)[..16])
}

fn hexish(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
