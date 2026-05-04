use anyhow::Context;
use clap::{Parser, Subcommand};
use replay_core::{default_sidebar_plan, Diagnostic, DiagnosticCategory, DiagnosticSeverity};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(name = "afterlivie")]
#[command(about = "AfterLivie replay composer CLI harness")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Doctor,
    ImportPlan {
        #[arg(long)]
        comments: PathBuf,
        #[arg(long, default_value_t = 1280)]
        media_width: u32,
        #[arg(long, default_value_t = 720)]
        media_height: u32,
        #[arg(long, default_value_t = 30.0)]
        fps: f64,
        #[arg(long, default_value_t = 0)]
        media_duration_ms: i64,
        #[arg(long)]
        out: PathBuf,
    },
    OverlayFrame {
        #[arg(long)]
        plan: PathBuf,
        #[arg(long)]
        time_ms: i64,
        #[arg(long)]
        out: PathBuf,
    },
    ProbeMedia {
        #[arg(long)]
        video: PathBuf,
    },
    PreviewExport {
        #[arg(long)]
        video: PathBuf,
        #[arg(long)]
        comments: PathBuf,
        #[arg(long)]
        out: PathBuf,
        #[arg(long, default_value_t = 0)]
        start_ms: i64,
        #[arg(long, default_value_t = 30_000)]
        duration_ms: i64,
    },
    FullExport {
        #[arg(long)]
        video: PathBuf,
        #[arg(long)]
        comments: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
    NewProject {
        #[arg(long)]
        out: PathBuf,
    },
    ConsistencyReport {
        #[arg(long)]
        plan: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Doctor => doctor(),
        Command::ImportPlan {
            comments,
            media_width,
            media_height,
            fps,
            media_duration_ms: _,
            out,
        } => {
            let result = replay_importers::import_canonical_json(&comments);
            let plan = default_sidebar_plan(
                media_width,
                media_height,
                fps,
                vec![result.source.clone()],
                result.comments.clone(),
            );
            write_json(
                &out,
                &serde_json::json!({
                    "importResult": result,
                    "renderPlan": plan,
                }),
            )
        }
        Command::OverlayFrame { plan, time_ms, out } => {
            let plan = read_plan(&plan)?;
            let report = replay_overlay::render_overlay_png(&plan, time_ms, &out)?;
            print_json(&report)
        }
        Command::ProbeMedia { video } => {
            let result = replay_media::probe_media(&video)?;
            print_json(&serde_json::json!({
                "mediaInfo": result.info,
                "diagnostics": result.diagnostics,
            }))
        }
        Command::PreviewExport {
            video,
            comments,
            out,
            start_ms,
            duration_ms,
        } => {
            let plan = plan_from_probe_and_comments(&video, &comments)?;
            let request = replay_render::RenderRequest {
                source_video_path: video.to_string_lossy().into_owned(),
                output_path: out.to_string_lossy().into_owned(),
                plan,
                profile: replay_render::preview_profile(start_ms, duration_ms),
            };
            let report = replay_render::render_preview(&request)?;
            print_json(&report)
        }
        Command::FullExport {
            video,
            comments,
            out,
        } => {
            let plan = plan_from_probe_and_comments(&video, &comments)?;
            let request = replay_render::RenderRequest {
                source_video_path: video.to_string_lossy().into_owned(),
                output_path: out.to_string_lossy().into_owned(),
                plan,
                profile: replay_render::final_export_profile(),
            };
            let report = replay_render::render_export(&request)?;
            print_json(&report)
        }
        Command::NewProject { out } => {
            let project = replay_core::ReplayProject::new();
            let package = replay_storage::ProjectPackage::create(&out, &project)?;
            print_json(&serde_json::json!({
                "projectPath": package.root,
                "project": project,
            }))
        }
        Command::ConsistencyReport { plan, out } => {
            let plan = read_plan(&plan)?;
            let report = consistency_report(&plan);
            write_json(&out, &report)
        }
    }
}

fn plan_from_probe_and_comments(
    video: &Path,
    comments: &Path,
) -> anyhow::Result<replay_core::SemanticRenderPlan> {
    let import_result = replay_importers::import_canonical_json(comments);
    let probe = replay_media::probe_media(video)?;
    let info = probe
        .info
        .context("media probe did not produce media info; inspect diagnostics")?;
    Ok(default_sidebar_plan(
        info.width.unwrap_or(1280),
        info.height.unwrap_or(720),
        info.fps.unwrap_or(30.0),
        vec![import_result.source],
        import_result.comments,
    ))
}

fn read_plan(path: &Path) -> anyhow::Result<replay_core::SemanticRenderPlan> {
    let value: serde_json::Value = serde_json::from_slice(&fs::read(path)?)?;
    if let Some(plan) = value.get("renderPlan") {
        Ok(serde_json::from_value(plan.clone())?)
    } else {
        Ok(serde_json::from_value(value)?)
    }
}

fn write_json(path: &Path, value: &impl serde::Serialize) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

fn print_json(value: &impl serde::Serialize) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn consistency_report(plan: &replay_core::SemanticRenderPlan) -> serde_json::Value {
    let sample_times = [0, 1_000, 5_000, 10_000, 30_000];
    let rows = sample_times
        .iter()
        .map(|time_ms| {
            let visible = replay_core::visible_comments(
                &plan.comments,
                &plan.timeline.sources,
                replay_core::VisibilityQuery {
                    time_ms: *time_ms,
                    comment_window_ms: plan.timeline.comment_window.duration_ms,
                    max_visible_comments: plan.timeline.max_visible_comments,
                    global_offset_ms: plan.timeline.global_offset_ms,
                },
            );
            serde_json::json!({
                "timeMs": time_ms,
                "visibleCommentIds": visible.iter().map(|comment| &comment.id).collect::<Vec<_>>(),
                "panelBounds": plan.regions.comments,
                "sourceBounds": plan.regions.source_video,
            })
        })
        .collect::<Vec<_>>();
    serde_json::json!({
        "schemaVersion": "1.0",
        "semanticPlanHashInput": plan,
        "sampleComparisons": rows,
        "result": "preview_export_semantics_share_single_plan"
    })
}

fn doctor() -> anyhow::Result<()> {
    let mut diagnostics = Vec::new();
    check_binary("rustc", &mut diagnostics);
    check_binary("cargo", &mut diagnostics);
    check_binary("ffmpeg", &mut diagnostics);
    check_binary("ffprobe", &mut diagnostics);
    check_binary("swift", &mut diagnostics);
    check_binary("xcodebuild", &mut diagnostics);

    let fatal = diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Fatal);
    print_json(&serde_json::json!({
        "ok": !fatal,
        "diagnostics": diagnostics,
        "ffmpegVersion": replay_media::ffmpeg_version(),
    }))?;

    if fatal {
        anyhow::bail!("doctor found missing required tools");
    }
    Ok(())
}

fn check_binary(binary: &str, diagnostics: &mut Vec<Diagnostic>) {
    let found = std::env::var_os("PATH")
        .and_then(|paths| {
            std::env::split_paths(&paths)
                .map(|path| path.join(binary))
                .find(|candidate| candidate.is_file())
        })
        .is_some();
    if found {
        diagnostics.push(Diagnostic::new(
            DiagnosticSeverity::Info,
            DiagnosticCategory::Project,
            format!("doctor.{binary}_found"),
            format!("{binary} found on PATH."),
        ));
    } else {
        diagnostics.push(
            Diagnostic::new(
                DiagnosticSeverity::Fatal,
                DiagnosticCategory::Project,
                format!("doctor.{binary}_missing"),
                format!("{binary} was not found on PATH."),
            )
            .with_hint("Enter nix develop or install the missing tool."),
        );
    }
}
