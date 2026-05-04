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
        #[arg(
            long,
            conflicts_with = "source_manifest",
            required_unless_present = "source_manifest"
        )]
        comments: Option<PathBuf>,
        #[arg(long)]
        source_manifest: Option<PathBuf>,
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
    ImportPreview {
        #[arg(long)]
        source_spec: PathBuf,
        #[arg(long, default_value_t = 100)]
        max_rows: usize,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    MergedTimeline {
        #[arg(
            long,
            conflicts_with = "source_manifest",
            required_unless_present = "source_manifest"
        )]
        comments: Option<PathBuf>,
        #[arg(long)]
        source_manifest: Option<PathBuf>,
        #[arg(long, default_value_t = 0)]
        global_offset_ms: i64,
        #[arg(long, default_value_t = 100)]
        limit: usize,
        #[arg(long)]
        out: Option<PathBuf>,
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
        #[arg(
            long,
            conflicts_with = "source_manifest",
            required_unless_present = "source_manifest"
        )]
        comments: Option<PathBuf>,
        #[arg(long)]
        source_manifest: Option<PathBuf>,
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
        #[arg(
            long,
            conflicts_with = "source_manifest",
            required_unless_present = "source_manifest"
        )]
        comments: Option<PathBuf>,
        #[arg(long)]
        source_manifest: Option<PathBuf>,
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
            source_manifest,
            media_width,
            media_height,
            fps,
            media_duration_ms: _,
            out,
        } => {
            let result = load_import_batch(comments.as_deref(), source_manifest.as_deref())?;
            let plan = default_sidebar_plan(
                media_width,
                media_height,
                fps,
                result.sources.clone(),
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
        Command::ImportPreview {
            source_spec,
            max_rows,
            out,
        } => {
            let spec = read_source_spec(&source_spec)?;
            let preview = replay_importers::preview_source(&spec, max_rows);
            write_or_print_json(out.as_deref(), &preview)
        }
        Command::MergedTimeline {
            comments,
            source_manifest,
            global_offset_ms,
            limit,
            out,
        } => {
            let batch = load_import_batch(comments.as_deref(), source_manifest.as_deref())?;
            let report = replay_importers::merged_timeline_report(&batch, global_offset_ms, limit);
            write_or_print_json(out.as_deref(), &report)
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
            source_manifest,
            out,
            start_ms,
            duration_ms,
        } => {
            let plan = plan_from_probe_and_comments(
                &video,
                comments.as_deref(),
                source_manifest.as_deref(),
            )?;
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
            source_manifest,
            out,
        } => {
            let plan = plan_from_probe_and_comments(
                &video,
                comments.as_deref(),
                source_manifest.as_deref(),
            )?;
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
    comments: Option<&Path>,
    source_manifest: Option<&Path>,
) -> anyhow::Result<replay_core::SemanticRenderPlan> {
    let import_result = load_import_batch(comments, source_manifest)?;
    let probe = replay_media::probe_media(video)?;
    let info = probe
        .info
        .context("media probe did not produce media info; inspect diagnostics")?;
    Ok(default_sidebar_plan(
        info.width.unwrap_or(1280),
        info.height.unwrap_or(720),
        info.fps.unwrap_or(30.0),
        import_result.sources,
        import_result.comments,
    ))
}

fn load_import_batch(
    comments: Option<&Path>,
    source_manifest: Option<&Path>,
) -> anyhow::Result<replay_importers::ImportBatchResult> {
    if let Some(source_manifest) = source_manifest {
        let manifest: replay_importers::SourceManifest =
            serde_json::from_slice(&fs::read(source_manifest)?)?;
        return Ok(replay_importers::import_sources(&manifest.sources));
    }

    let comments = comments.context("provide --comments or --source-manifest")?;
    let result = replay_importers::import_canonical_json(comments);
    Ok(replay_importers::ImportBatchResult {
        sources: vec![result.source.clone()],
        comments: result.comments.clone(),
        diagnostics: result.diagnostics.clone(),
        skipped_count: result.skipped_count,
        source_results: vec![replay_importers::ImportSourceResultSummary {
            source_id: result.source.source_id.clone(),
            display_name: result.source.display_name.clone(),
            importer_id: result.source.importer_id.clone(),
            importer_version: result.importer_version.clone(),
            detected_format: result.detected_format.clone(),
            detected_encoding: result.detected_encoding.clone(),
            comment_count: result.comments.len(),
            skipped_count: result.skipped_count,
            diagnostics: result.diagnostics,
        }],
    })
}

fn read_source_spec(path: &Path) -> anyhow::Result<replay_importers::ImportSourceSpec> {
    serde_json::from_slice(&fs::read(path)?).with_context(|| format!("parse {}", path.display()))
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

fn write_or_print_json(path: Option<&Path>, value: &impl serde::Serialize) -> anyhow::Result<()> {
    if let Some(path) = path {
        write_json(path, value)
    } else {
        print_json(value)
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clap_definition_is_valid() {
        <Cli as clap::CommandFactory>::command().debug_assert();
    }

    #[test]
    fn source_manifest_imports_and_reports_merged_timeline() {
        let dir = tempfile::tempdir().unwrap();
        let csv_path = dir.path().join("comments.csv");
        fs::write(&csv_path, "timestamp,text,author,id\n1s,hello,Livie,c1\n").unwrap();
        let manifest_path = dir.path().join("sources.json");
        fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(&serde_json::json!({
                "sources": [
                    {
                        "path": csv_path.to_string_lossy(),
                        "format": "csv",
                        "sourceId": "csv_main",
                        "displayName": "CSV Main",
                        "platform": "youtube",
                        "enabled": true,
                        "offsetMs": 500
                    }
                ]
            }))
            .unwrap(),
        )
        .unwrap();

        let batch = load_import_batch(None, Some(&manifest_path)).unwrap();
        assert_eq!(batch.sources.len(), 1);
        assert_eq!(batch.comments.len(), 1);
        let report = replay_importers::merged_timeline_report(&batch, 0, 10);
        assert_eq!(report.rows[0].effective_timestamp_ms, 1_500);
        assert_eq!(report.source_summaries[0].comment_count, 1);
    }
}
