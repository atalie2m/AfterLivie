use anyhow::Context;
use chrono::Utc;
use replay_core::{
    Diagnostic, DiagnosticCategory, DiagnosticSeverity, RenderExecutionProfile,
    RenderExecutionProfileKind, RenderHistoryEntry, SemanticRenderPlan, SourceRef,
};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use uuid::Uuid;

static CANCEL_REQUESTED: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderRequest {
    pub source_video_path: String,
    pub output_path: String,
    pub plan: SemanticRenderPlan,
    pub profile: RenderExecutionProfile,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderReport {
    pub output_path: String,
    pub cancelled: bool,
    pub diagnostics: Vec<Diagnostic>,
    pub render_history_entry: RenderHistoryEntry,
}

pub fn cancel_render() {
    CANCEL_REQUESTED.store(true, Ordering::SeqCst);
}

pub fn reset_cancel_flag() {
    CANCEL_REQUESTED.store(false, Ordering::SeqCst);
}

pub fn render_preview(request: &RenderRequest) -> anyhow::Result<RenderReport> {
    ensure_profile(request, RenderExecutionProfileKind::PreviewSegment)?;
    render_with_ffmpeg(request)
}

pub fn render_export(request: &RenderRequest) -> anyhow::Result<RenderReport> {
    ensure_profile(request, RenderExecutionProfileKind::FinalExport)?;
    render_with_ffmpeg(request)
}

fn ensure_profile(
    request: &RenderRequest,
    expected: RenderExecutionProfileKind,
) -> anyhow::Result<()> {
    if request.profile.kind != expected {
        anyhow::bail!("render request has wrong execution profile");
    }
    Ok(())
}

fn render_with_ffmpeg(request: &RenderRequest) -> anyhow::Result<RenderReport> {
    reset_cancel_flag();
    let started_at = Utc::now();
    let tempdir = tempfile::tempdir()?;
    let overlay_path = tempdir.path().join("overlay.png");
    let overlay_report = replay_overlay::render_overlay_png(
        &request.plan,
        request.profile.start_ms.unwrap_or(0),
        &overlay_path,
    )?;

    let mut diagnostics = overlay_report.diagnostics;
    let ffmpeg = match replay_media::ffmpeg_path() {
        Some(path) => path,
        None => {
            diagnostics.push(
                Diagnostic::new(
                    DiagnosticSeverity::Fatal,
                    DiagnosticCategory::Export,
                    "export.ffmpeg_missing",
                    "ffmpeg was not found.",
                )
                .with_hint("Enter the Nix dev shell or configure AFTERLIVIE_FFMPEG."),
            );
            return Ok(report(
                request,
                started_at,
                false,
                diagnostics,
                request.output_path.clone(),
            ));
        }
    };

    let mut command = Command::new(ffmpeg);
    command.arg("-y");
    if let Some(start_ms) = request.profile.start_ms {
        command.args(["-ss", &seconds_arg(start_ms)]);
    }
    command.arg("-i").arg(&request.source_video_path);
    command.args(["-loop", "1"]).arg("-i").arg(&overlay_path);
    if let Some(duration_ms) = request.profile.duration_ms {
        command.args(["-t", &seconds_arg(duration_ms)]);
    }

    let filter = format!(
        "[0:v]pad={}:{}:{}:{}:black[base];[base][1:v]overlay=0:0:format=auto[v]",
        request.plan.canvas.width,
        request.plan.canvas.height,
        request.plan.regions.source_video.x,
        request.plan.regions.source_video.y
    );
    command
        .args(["-filter_complex", &filter])
        .args(["-map", "[v]"])
        .args(["-map", "0:a?"])
        .args(["-c:v", "libx264"])
        .args(["-pix_fmt", "yuv420p"])
        .args(["-c:a", "aac"])
        .args(["-shortest"])
        .arg(&request.output_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command.spawn().context("failed to spawn ffmpeg")?;
    let cancelled = loop {
        if CANCEL_REQUESTED.load(Ordering::SeqCst) {
            let _ = child.kill();
            let _ = child.wait();
            diagnostics.push(
                Diagnostic::new(
                    DiagnosticSeverity::Warning,
                    DiagnosticCategory::Export,
                    "export.cancelled",
                    "Render was cancelled.",
                )
                .with_source_ref(SourceRef::ExportStage {
                    stage: "ffmpeg_composition".into(),
                }),
            );
            break true;
        }
        if let Some(status) = child.try_wait()? {
            let output = child.wait_with_output()?;
            if !status.success() {
                diagnostics.push(
                    Diagnostic::new(
                        DiagnosticSeverity::Fatal,
                        DiagnosticCategory::Export,
                        "export.ffmpeg_failed",
                        String::from_utf8_lossy(&output.stderr).trim().to_string(),
                    )
                    .with_source_ref(SourceRef::ExportStage {
                        stage: "ffmpeg_composition".into(),
                    })
                    .with_hint("Check source compatibility diagnostics and ffmpeg availability."),
                );
            }
            break false;
        }
        std::thread::sleep(Duration::from_millis(100));
    };

    Ok(report(
        request,
        started_at,
        cancelled,
        diagnostics,
        request.output_path.clone(),
    ))
}

fn report(
    request: &RenderRequest,
    started_at: chrono::DateTime<Utc>,
    cancelled: bool,
    diagnostics: Vec<Diagnostic>,
    output_path: String,
) -> RenderReport {
    RenderReport {
        output_path: output_path.clone(),
        cancelled,
        diagnostics,
        render_history_entry: RenderHistoryEntry {
            id: Uuid::new_v4().to_string(),
            started_at,
            completed_at: Some(Utc::now()),
            output_path,
            preset: request.profile.bitrate_policy.clone(),
            diagnostics_ref: None,
        },
    }
}

fn seconds_arg(ms: i64) -> String {
    format!("{:.3}", ms as f64 / 1000.0)
}

pub fn preview_profile(start_ms: i64, duration_ms: i64) -> RenderExecutionProfile {
    RenderExecutionProfile {
        kind: RenderExecutionProfileKind::PreviewSegment,
        start_ms: Some(start_ms),
        duration_ms: Some(duration_ms),
        output_scale: 1.0,
        bitrate_policy: "preview".into(),
        encoder: "libx264".into(),
        cache_policy: "reuse_allowed".into(),
    }
}

pub fn final_export_profile() -> RenderExecutionProfile {
    RenderExecutionProfile {
        kind: RenderExecutionProfileKind::FinalExport,
        start_ms: None,
        duration_ms: None,
        output_scale: 1.0,
        bitrate_policy: "high_quality_upload".into(),
        encoder: "libx264".into(),
        cache_policy: "reuse_allowed".into(),
    }
}

pub fn output_path_exists(path: impl AsRef<Path>) -> bool {
    PathBuf::from(path.as_ref()).exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profiles_keep_semantics_out_of_execution() {
        let preview = preview_profile(1000, 30_000);
        let final_export = final_export_profile();
        assert_eq!(preview.kind, RenderExecutionProfileKind::PreviewSegment);
        assert_eq!(final_export.kind, RenderExecutionProfileKind::FinalExport);
        assert_eq!(preview.output_scale, final_export.output_scale);
    }
}
