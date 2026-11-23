use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentEvent {
    pub id: String,
    pub source_id: String,
    pub platform: Option<String>,
    pub timestamp_ms: i64,
    pub original_timestamp: Option<OriginalTimestamp>,
    pub kind: CommentKind,
    pub author: Option<CommentAuthor>,
    pub body: CommentBody,
    pub style: Option<CommentStyle>,
    #[serde(default)]
    pub badges: Vec<Badge>,
    #[serde(default)]
    pub emotes: Vec<Emote>,
    pub source: Option<CommentSourceRef>,
    pub raw_ref: Option<RawSourceRef>,
    pub import_order: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommentKind {
    #[default]
    Message,
    Pinned,
    Metadata,
    Reaction,
    System,
}

impl CommentKind {
    pub fn priority(self) -> u8 {
        match self {
            CommentKind::Pinned => 0,
            CommentKind::Metadata => 1,
            CommentKind::Message => 2,
            CommentKind::Reaction => 3,
            CommentKind::System => 4,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentAuthor {
    pub id: Option<String>,
    pub display_name: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentBody {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentStyle {
    pub color: Option<String>,
    pub weight: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Badge {
    pub id: String,
    pub label: Option<String>,
    pub asset_ref: Option<AssetRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Emote {
    pub id: String,
    pub text: String,
    pub asset_ref: Option<AssetRef>,
    pub start: Option<usize>,
    pub end: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OriginalTimestamp {
    pub value: String,
    pub basis: TimestampBasis,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawSourceRef {
    pub row: Option<u64>,
    pub pointer: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentSourceRef {
    pub path: Option<String>,
    pub row: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentSource {
    pub source_id: String,
    pub display_name: String,
    pub platform: Option<String>,
    pub importer_id: String,
    pub original_file_ref: AssetRef,
    pub enabled: bool,
    pub offset_ms: i64,
    pub timestamp_basis: TimestampBasis,
    pub diagnostics_ref: Option<String>,
    pub visual_style: Option<SourceVisualStyle>,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
    pub import_fingerprint: ImportFingerprint,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimestampBasis {
    RelativeToVideoStart,
    RelativeToStreamStart,
    AbsoluteUtc,
    LocalTimeWithTimezone,
    LocalTimeAmbiguous,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceVisualStyle {
    pub color: Option<String>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetRef {
    pub path: String,
    pub hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportFingerprint {
    pub algorithm: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub source: CommentSource,
    pub comments: Vec<CommentEvent>,
    pub diagnostics: Vec<crate::Diagnostic>,
    pub skipped_count: u64,
    pub importer_version: String,
    pub detected_format: String,
    pub detected_encoding: Option<String>,
}

pub fn global_comment_key(source_id: &str, comment_id: &str) -> String {
    format!("{source_id}:{comment_id}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_key_is_source_scoped() {
        assert_eq!(global_comment_key("main", "c1"), "main:c1");
    }

    #[test]
    fn comment_event_roundtrips() {
        let event = CommentEvent {
            id: "c1".into(),
            source_id: "main".into(),
            platform: Some("youtube".into()),
            timestamp_ms: 1200,
            original_timestamp: None,
            kind: CommentKind::Message,
            author: Some(CommentAuthor {
                id: None,
                display_name: "Livie".into(),
                color: None,
            }),
            body: CommentBody {
                text: "hello".into(),
            },
            style: None,
            badges: vec![],
            emotes: vec![],
            source: None,
            raw_ref: None,
            import_order: 0,
        };

        let json = serde_json::to_string(&event).unwrap();
        let decoded: CommentEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, event);
    }
}
