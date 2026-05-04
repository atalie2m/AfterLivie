use crate::{CommentSource, Diagnostic, ReplayMediaInfo};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayProject {
    pub schema_version: String,
    pub project_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub source_video: Option<SourceVideoRef>,
    pub comment_sources: Vec<CommentSource>,
    pub layout_template_id: String,
    pub layout_overrides: serde_json::Value,
    pub render_settings: RenderSettings,
    pub overlay_renderer: OverlayRendererMetadata,
    pub diagnostics: Vec<Diagnostic>,
}

impl ReplayProject {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            schema_version: "1.0".into(),
            project_id: Uuid::new_v4().to_string(),
            created_at: now,
            updated_at: now,
            source_video: None,
            comment_sources: vec![],
            layout_template_id: "classic-sidebar-v1".into(),
            layout_overrides: serde_json::json!({}),
            render_settings: RenderSettings {
                preset: "high_quality_upload".into(),
            },
            overlay_renderer: OverlayRendererMetadata {
                renderer_id: "rust-cpu-overlay".into(),
                renderer_version: env!("CARGO_PKG_VERSION").into(),
                font_policy_id: "system-default-v1".into(),
            },
            diagnostics: vec![],
        }
    }
}

impl Default for ReplayProject {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceVideoRef {
    pub asset_ref: crate::AssetRef,
    pub media_info: Option<ReplayMediaInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderSettings {
    pub preset: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlayRendererMetadata {
    pub renderer_id: String,
    pub renderer_version: String,
    pub font_policy_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderHistoryEntry {
    pub id: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub output_path: String,
    pub preset: String,
    pub diagnostics_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReproducibilityManifest {
    pub app_version: String,
    pub core_version: String,
    pub renderer_version: String,
    pub font_policy_version: String,
    pub template_version: String,
    pub ffmpeg_version: Option<String>,
    pub media_backend: String,
    pub source_video_hash: Option<String>,
    pub comment_file_hash: Option<String>,
    pub importer_id: String,
    pub importer_version: String,
    pub import_options_hash: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_project_has_v1_schema() {
        assert_eq!(ReplayProject::new().schema_version, "1.0");
    }

    #[test]
    fn project_roundtrips() {
        let project = ReplayProject::new();
        let json = serde_json::to_string_pretty(&project).unwrap();
        let decoded: ReplayProject = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.schema_version, "1.0");
        assert_eq!(decoded.project_id, project.project_id);
    }
}
