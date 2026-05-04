use crate::{CommentEvent, CommentSource, MediaBackend};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticRenderPlan {
    pub schema_version: String,
    pub canvas: Canvas,
    pub regions: LayoutRegions,
    pub timeline: TimelinePlan,
    pub ordering: OrderingRules,
    pub style: StyleTokens,
    pub overlay_renderer_id: String,
    pub font_policy_id: String,
    pub emoji_policy_id: String,
    pub audio_policy: AudioPolicy,
    pub media_backend: MediaBackend,
    pub comments: Vec<CommentEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pub fps: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutRegions {
    pub source_video: SourceVideoRegion,
    pub comments: CommentPanelRegion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceVideoRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentPanelRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelinePlan {
    pub comment_window: CommentWindow,
    pub global_offset_ms: i64,
    pub sources: Vec<CommentSource>,
    pub max_visible_comments: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentWindow {
    pub duration_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderingRules {
    pub tie_breakers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleTokens {
    pub panel_background: String,
    pub row_separator: String,
    pub text_primary: String,
    pub text_secondary: String,
    pub author_colors: Vec<String>,
    pub font_size: f32,
    pub row_padding: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioPolicy {
    AttemptPassthroughThenAac,
    AacReencode,
    SilentWithDiagnostic,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderExecutionProfile {
    pub kind: RenderExecutionProfileKind,
    pub start_ms: Option<i64>,
    pub duration_ms: Option<i64>,
    pub output_scale: f32,
    pub bitrate_policy: String,
    pub encoder: String,
    pub cache_policy: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderExecutionProfileKind {
    PreviewSegment,
    FinalExport,
}

pub fn default_sidebar_plan(
    source_width: u32,
    source_height: u32,
    fps: f64,
    sources: Vec<CommentSource>,
    comments: Vec<CommentEvent>,
) -> SemanticRenderPlan {
    let panel_width = 480;
    SemanticRenderPlan {
        schema_version: "1.0".into(),
        canvas: Canvas {
            width: source_width + panel_width,
            height: source_height,
            fps,
        },
        regions: LayoutRegions {
            source_video: SourceVideoRegion {
                x: 0,
                y: 0,
                width: source_width,
                height: source_height,
            },
            comments: CommentPanelRegion {
                x: source_width,
                y: 0,
                width: panel_width,
                height: source_height,
            },
        },
        timeline: TimelinePlan {
            comment_window: CommentWindow {
                duration_ms: 30_000,
            },
            global_offset_ms: 0,
            sources,
            max_visible_comments: 18,
        },
        ordering: OrderingRules {
            tie_breakers: vec![
                "effective_timestamp_ms".into(),
                "event_priority".into(),
                "source_priority".into(),
                "import_order".into(),
                "stable_id".into(),
            ],
        },
        style: StyleTokens {
            panel_background: "#151719E6".into(),
            row_separator: "#2D333A99".into(),
            text_primary: "#F5F7FA".into(),
            text_secondary: "#9BA6B2".into(),
            author_colors: vec![
                "#6EE7B7".into(),
                "#93C5FD".into(),
                "#FCA5A5".into(),
                "#FCD34D".into(),
            ],
            font_size: 18.0,
            row_padding: 12,
        },
        overlay_renderer_id: "rust-cpu-overlay".into(),
        font_policy_id: "system-default-v1".into(),
        emoji_policy_id: "placeholder-diagnostic-v1".into(),
        audio_policy: AudioPolicy::AttemptPassthroughThenAac,
        media_backend: MediaBackend::FfmpegAssisted,
        comments,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_sidebar_uses_no_source_scaling() {
        let plan = default_sidebar_plan(1280, 720, 30.0, vec![], vec![]);
        assert_eq!(plan.canvas.width, 1760);
        assert_eq!(plan.canvas.height, 720);
        assert_eq!(plan.regions.source_video.width, 1280);
        assert_eq!(plan.regions.comments.x, 1280);
    }

    #[test]
    fn render_plan_serializes_deterministically() {
        let plan = default_sidebar_plan(1280, 720, 30.0, vec![], vec![]);
        let json = serde_json::to_string_pretty(&plan).unwrap();
        assert!(json.contains("\"schemaVersion\": \"1.0\""));
        assert!(json.contains("\"overlayRendererId\""));
    }
}
