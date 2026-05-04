use anyhow::{anyhow, Context};
use replay_core::{
    AudioCodec, CompatibilityDecision, Diagnostic, DiagnosticCategory, DiagnosticSeverity,
    MediaBackend, MediaCompatibility, ReplayMediaInfo, SourceRef, VideoCodec,
};
use serde_json::Value;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct MediaProbeResult {
    pub info: Option<ReplayMediaInfo>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn ffprobe_path() -> Option<String> {
    std::env::var("AFTERLIVIE_FFPROBE")
        .ok()
        .or_else(|| std::env::var("FFPROBE").ok())
        .or_else(|| which_on_path("ffprobe"))
}

pub fn ffmpeg_path() -> Option<String> {
    std::env::var("AFTERLIVIE_FFMPEG")
        .ok()
        .or_else(|| std::env::var("FFMPEG").ok())
        .or_else(|| which_on_path("ffmpeg"))
}

pub fn ffmpeg_version() -> Option<String> {
    let path = ffmpeg_path()?;
    let output = Command::new(path).arg("-version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .and_then(|text| text.lines().next().map(ToOwned::to_owned))
}

pub fn probe_media(path: impl AsRef<Path>) -> anyhow::Result<MediaProbeResult> {
    let path = path.as_ref();
    let Some(ffprobe) = ffprobe_path() else {
        return Ok(MediaProbeResult {
            info: None,
            diagnostics: vec![Diagnostic::new(
                DiagnosticSeverity::Fatal,
                DiagnosticCategory::Media,
                "media.ffprobe_missing",
                "ffprobe was not found.",
            )
            .with_source_ref(SourceRef::File {
                path: path.to_string_lossy().into_owned(),
            })
            .with_hint("Enter the Nix dev shell or configure AFTERLIVIE_FFPROBE.")],
        });
    };

    let output = Command::new(ffprobe)
        .args([
            "-v",
            "error",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
        ])
        .arg(path)
        .output()
        .with_context(|| format!("failed to run ffprobe for {}", path.display()))?;

    if !output.status.success() {
        return Ok(MediaProbeResult {
            info: None,
            diagnostics: vec![Diagnostic::new(
                DiagnosticSeverity::Fatal,
                DiagnosticCategory::Media,
                "media.ffprobe_failed",
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
            )
            .with_source_ref(SourceRef::File {
                path: path.to_string_lossy().into_owned(),
            })
            .with_hint("Check that the source media exists and ffprobe can read it.")],
        });
    }

    let value: Value = serde_json::from_slice(&output.stdout)
        .context("ffprobe returned JSON that could not be parsed")?;
    Ok(normalize_ffprobe_json(path, &value))
}

pub fn normalize_ffprobe_json(path: &Path, value: &Value) -> MediaProbeResult {
    let streams = value
        .get("streams")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let format = value.get("format").cloned().unwrap_or(Value::Null);
    let video_stream = streams
        .iter()
        .find(|stream| stream.get("codec_type").and_then(Value::as_str) == Some("video"));
    let audio_streams = streams
        .iter()
        .filter(|stream| stream.get("codec_type").and_then(Value::as_str) == Some("audio"))
        .collect::<Vec<_>>();
    let subtitle_count = streams
        .iter()
        .filter(|stream| stream.get("codec_type").and_then(Value::as_str) == Some("subtitle"))
        .count();

    let video_codec = video_stream
        .and_then(|stream| stream.get("codec_name").and_then(Value::as_str))
        .map(map_video_codec);
    let audio_codec = audio_streams
        .first()
        .and_then(|stream| stream.get("codec_name").and_then(Value::as_str))
        .map(map_audio_codec);
    let container = format
        .get("format_name")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);

    let mut diagnostics = Vec::new();
    let compatibility = compatibility_decision(
        container.as_deref(),
        video_codec.as_ref(),
        audio_codec.as_ref(),
        audio_streams.len(),
        subtitle_count,
        &mut diagnostics,
        path,
    );

    let info = ReplayMediaInfo {
        path: path.to_string_lossy().into_owned(),
        duration_ms: format
            .get("duration")
            .and_then(Value::as_str)
            .and_then(parse_seconds_to_ms),
        width: video_stream
            .and_then(|stream| stream.get("width").and_then(Value::as_u64))
            .map(|value| value as u32),
        height: video_stream
            .and_then(|stream| stream.get("height").and_then(Value::as_u64))
            .map(|value| value as u32),
        fps: video_stream
            .and_then(|stream| stream.get("avg_frame_rate").and_then(Value::as_str))
            .and_then(parse_ratio),
        is_vfr: detect_vfr(video_stream),
        video_codec,
        container,
        pixel_format: video_stream
            .and_then(|stream| stream.get("pix_fmt").and_then(Value::as_str))
            .map(ToOwned::to_owned),
        has_audio: !audio_streams.is_empty(),
        audio_codec,
        audio_track_count: audio_streams.len(),
        has_subtitles: subtitle_count > 0,
        color_metadata: video_stream.and_then(color_metadata),
        hdr: video_stream.and_then(detect_hdr),
        bitrate: format
            .get("bit_rate")
            .and_then(Value::as_str)
            .and_then(|value| value.parse::<u64>().ok()),
        timebase: video_stream
            .and_then(|stream| stream.get("time_base").and_then(Value::as_str))
            .map(ToOwned::to_owned),
        start_time_ms: video_stream
            .and_then(|stream| stream.get("start_time").and_then(Value::as_str))
            .and_then(parse_seconds_to_ms),
        rotation_degrees: video_stream.and_then(rotation_degrees),
        sample_aspect_ratio: video_stream
            .and_then(|stream| stream.get("sample_aspect_ratio").and_then(Value::as_str))
            .map(ToOwned::to_owned),
        compatibility,
    };

    MediaProbeResult {
        info: Some(info),
        diagnostics,
    }
}

fn compatibility_decision(
    container: Option<&str>,
    video_codec: Option<&VideoCodec>,
    audio_codec: Option<&AudioCodec>,
    audio_count: usize,
    subtitle_count: usize,
    diagnostics: &mut Vec<Diagnostic>,
    path: &Path,
) -> MediaCompatibility {
    let compatible_container = container
        .map(|name| name.contains("mov") || name.contains("mp4") || name.contains("m4v"))
        .unwrap_or(false);
    let compatible_video = matches!(video_codec, Some(VideoCodec::H264 | VideoCodec::Hevc));
    let compatible_audio = audio_count == 0 || matches!(audio_codec, Some(AudioCodec::Aac));
    let mut reasons = Vec::new();

    if audio_count == 0 {
        diagnostics.push(
            Diagnostic::new(
                DiagnosticSeverity::Warning,
                DiagnosticCategory::Media,
                "media.no_audio",
                "Source media has no audio track.",
            )
            .with_source_ref(SourceRef::File {
                path: path.to_string_lossy().into_owned(),
            })
            .with_hint("The export will be silent unless audio is added in a later version."),
        );
    }
    if audio_count > 1 {
        diagnostics.push(
            Diagnostic::new(
                DiagnosticSeverity::Warning,
                DiagnosticCategory::Media,
                "media.multiple_audio_tracks",
                "Source media has multiple audio tracks; v1.0 uses the first/default track.",
            )
            .with_source_ref(SourceRef::File {
                path: path.to_string_lossy().into_owned(),
            }),
        );
    }
    if subtitle_count > 0 {
        diagnostics.push(
            Diagnostic::new(
                DiagnosticSeverity::Info,
                DiagnosticCategory::Media,
                "media.subtitles_ignored",
                "Subtitle streams are ignored in v1.0 exports.",
            )
            .with_source_ref(SourceRef::File {
                path: path.to_string_lossy().into_owned(),
            }),
        );
    }

    if !compatible_container {
        reasons.push("container is not MP4/MOV compatible".into());
    }
    if !compatible_video {
        reasons.push("video codec is not H.264/HEVC".into());
    }
    if !compatible_audio {
        reasons.push("audio codec is not AAC".into());
    }

    let decision = if compatible_container && compatible_video && compatible_audio {
        CompatibilityDecision::AppleNativeCompatible
    } else if compatible_video && compatible_audio {
        CompatibilityDecision::RemuxCandidate
    } else if compatible_video || compatible_audio {
        CompatibilityDecision::TranscodeRequired
    } else {
        CompatibilityDecision::Unsupported
    };
    let backend = match decision {
        CompatibilityDecision::AppleNativeCompatible => MediaBackend::AppleNative,
        CompatibilityDecision::Unsupported => MediaBackend::Unsupported,
        _ => MediaBackend::FfmpegAssisted,
    };

    if decision == CompatibilityDecision::Unsupported {
        diagnostics.push(
            Diagnostic::new(
                DiagnosticSeverity::Fatal,
                DiagnosticCategory::Media,
                "media.unsupported_source",
                "Source media is not supported by the v1.0 compatibility path.",
            )
            .with_source_ref(SourceRef::File {
                path: path.to_string_lossy().into_owned(),
            })
            .with_hint("Use MP4/MOV H.264 or HEVC video with AAC audio, or transcode externally."),
        );
    }

    MediaCompatibility {
        decision,
        backend,
        reasons,
    }
}

fn map_video_codec(value: &str) -> VideoCodec {
    match value {
        "h264" => VideoCodec::H264,
        "hevc" | "h265" => VideoCodec::Hevc,
        "prores" => VideoCodec::Prores,
        other => VideoCodec::Other(other.into()),
    }
}

fn map_audio_codec(value: &str) -> AudioCodec {
    match value {
        "aac" => AudioCodec::Aac,
        "pcm_s16le" | "pcm_s24le" | "pcm_s32le" => AudioCodec::Pcm,
        "mp3" => AudioCodec::Mp3,
        "opus" => AudioCodec::Opus,
        other => AudioCodec::Other(other.into()),
    }
}

fn parse_seconds_to_ms(value: &str) -> Option<i64> {
    value
        .parse::<f64>()
        .ok()
        .map(|seconds| (seconds * 1000.0).round() as i64)
}

fn parse_ratio(value: &str) -> Option<f64> {
    let (num, den) = value.split_once('/')?;
    let num = num.parse::<f64>().ok()?;
    let den = den.parse::<f64>().ok()?;
    if den.abs() <= f64::EPSILON {
        None
    } else {
        Some(num / den)
    }
}

fn detect_vfr(video_stream: Option<&Value>) -> Option<bool> {
    let stream = video_stream?;
    let avg = stream.get("avg_frame_rate").and_then(Value::as_str)?;
    let real = stream.get("r_frame_rate").and_then(Value::as_str)?;
    Some(avg != real)
}

fn color_metadata(stream: &Value) -> Option<String> {
    let values = [
        stream.get("color_space").and_then(Value::as_str),
        stream.get("color_transfer").and_then(Value::as_str),
        stream.get("color_primaries").and_then(Value::as_str),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    if values.is_empty() {
        None
    } else {
        Some(values.join("/"))
    }
}

fn detect_hdr(stream: &Value) -> Option<bool> {
    let transfer = stream.get("color_transfer").and_then(Value::as_str)?;
    Some(matches!(transfer, "smpte2084" | "arib-std-b67"))
}

fn rotation_degrees(stream: &Value) -> Option<i32> {
    stream
        .pointer("/tags/rotate")
        .and_then(Value::as_str)
        .and_then(|value| value.parse::<i32>().ok())
}

fn which_on_path(binary: &str) -> Option<String> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(binary))
        .find(|candidate| candidate.is_file())
        .map(|candidate| candidate.to_string_lossy().into_owned())
}

pub fn require_ffmpeg() -> anyhow::Result<String> {
    ffmpeg_path().ok_or_else(|| anyhow!("ffmpeg was not found; enter nix develop"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn normalizes_compatible_mp4() {
        let json = serde_json::json!({
          "format": { "duration": "12.5", "format_name": "mov,mp4,m4a,3gp,3g2,mj2", "bit_rate": "1000000" },
          "streams": [
            { "codec_type": "video", "codec_name": "h264", "width": 1280, "height": 720, "avg_frame_rate": "30/1", "r_frame_rate": "30/1", "pix_fmt": "yuv420p" },
            { "codec_type": "audio", "codec_name": "aac" }
          ]
        });
        let result = normalize_ffprobe_json(&PathBuf::from("sample.mp4"), &json);
        let info = result.info.unwrap();
        assert_eq!(info.duration_ms, Some(12_500));
        assert_eq!(
            info.compatibility.decision,
            CompatibilityDecision::AppleNativeCompatible
        );
    }

    #[test]
    fn no_audio_is_diagnostic_but_supported() {
        let json = serde_json::json!({
          "format": { "duration": "1", "format_name": "mov,mp4" },
          "streams": [
            { "codec_type": "video", "codec_name": "hevc", "width": 1280, "height": 720, "avg_frame_rate": "30/1", "r_frame_rate": "30/1" }
          ]
        });
        let result = normalize_ffprobe_json(&PathBuf::from("silent.mp4"), &json);
        assert_eq!(result.diagnostics[0].code, "media.no_audio");
    }
}
