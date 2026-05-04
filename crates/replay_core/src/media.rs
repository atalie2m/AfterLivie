use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayMediaInfo {
    pub path: String,
    pub duration_ms: Option<i64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fps: Option<f64>,
    pub is_vfr: Option<bool>,
    pub video_codec: Option<VideoCodec>,
    pub container: Option<String>,
    pub pixel_format: Option<String>,
    pub has_audio: bool,
    pub audio_codec: Option<AudioCodec>,
    pub audio_track_count: usize,
    pub has_subtitles: bool,
    pub color_metadata: Option<String>,
    pub hdr: Option<bool>,
    pub bitrate: Option<u64>,
    pub timebase: Option<String>,
    pub start_time_ms: Option<i64>,
    pub rotation_degrees: Option<i32>,
    pub sample_aspect_ratio: Option<String>,
    pub compatibility: MediaCompatibility,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VideoCodec {
    H264,
    Hevc,
    Prores,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioCodec {
    Aac,
    Pcm,
    Mp3,
    Opus,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaCompatibility {
    pub decision: CompatibilityDecision,
    pub backend: MediaBackend,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompatibilityDecision {
    AppleNativeCompatible,
    RemuxCandidate,
    TranscodeRequired,
    Unsupported,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaBackend {
    AppleNative,
    FfmpegAssisted,
    Unsupported,
}

impl Default for MediaCompatibility {
    fn default() -> Self {
        Self {
            decision: CompatibilityDecision::Unknown,
            backend: MediaBackend::FfmpegAssisted,
            reasons: vec![],
        }
    }
}
