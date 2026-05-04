use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Error,
    Fatal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticCategory {
    Import,
    Timestamp,
    Media,
    Renderer,
    Export,
    Project,
    Storage,
    Sync,
    Schema,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SourceRef {
    File {
        path: String,
    },
    Row {
        path: Option<String>,
        row: u64,
    },
    Field {
        path: Option<String>,
        row: Option<u64>,
        field: String,
    },
    Timestamp {
        value: String,
    },
    Comment {
        source_id: String,
        comment_id: String,
    },
    Source {
        source_id: String,
    },
    MediaStream {
        path: String,
        stream_index: usize,
    },
    RendererComponent {
        component: String,
    },
    ExportStage {
        stage: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub id: String,
    pub severity: DiagnosticSeverity,
    pub category: DiagnosticCategory,
    pub message: String,
    pub source_ref: Option<SourceRef>,
    pub code: String,
    pub actionable_hint: Option<String>,
}

impl Diagnostic {
    pub fn new(
        severity: DiagnosticSeverity,
        category: DiagnosticCategory,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        let code = code.into();
        let message = message.into();
        Self {
            id: diagnostic_id(&code, &message),
            severity,
            category,
            message,
            source_ref: None,
            code,
            actionable_hint: None,
        }
    }

    pub fn with_source_ref(mut self, source_ref: SourceRef) -> Self {
        self.source_ref = Some(source_ref);
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.actionable_hint = Some(hint.into());
        self
    }
}

pub fn diagnostic_id(code: &str, message: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(code.as_bytes());
    hasher.update(b":");
    hasher.update(message.as_bytes());
    format!("diag_{}", &hexish(&hasher.finalize())[..16])
}

fn hexish(bytes: &[u8]) -> String {
    const TABLE: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(TABLE[(byte >> 4) as usize] as char);
        out.push(TABLE[(byte & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostic_ids_are_stable() {
        assert_eq!(
            diagnostic_id("import.invalid_timestamp", "bad timestamp"),
            diagnostic_id("import.invalid_timestamp", "bad timestamp")
        );
    }

    #[test]
    fn diagnostic_serializes_camel_case_fields() {
        let diagnostic = Diagnostic::new(
            DiagnosticSeverity::Warning,
            DiagnosticCategory::Import,
            "import.skipped",
            "Skipped one row",
        )
        .with_hint("Fix the row and import again.");

        let json = serde_json::to_string(&diagnostic).unwrap();
        assert!(json.contains("actionableHint"));
        assert!(json.contains("sourceRef"));
    }
}
