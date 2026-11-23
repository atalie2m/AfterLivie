# 08. Media Pipeline Plan

## 1. Media Strategy

Use a two-tier media strategy:

1. **Apple-native compatible path** for common MP4/MOV H.264/HEVC + AAC sources.
2. **FFmpeg-assisted compatibility path** for probing, diagnostics, proxy/intermediate generation, remuxing, audio conversion, and fallback workflows.

The application must not assume all local archive videos are directly usable by AVFoundation.

## 2. Probe First

Every source video should be probed before import completion.

### Probe Fields

- duration,
- resolution,
- frame rate,
- constant/variable frame rate indicator,
- codec,
- container,
- pixel format,
- audio track presence,
- audio codec,
- audio track count,
- subtitle track presence,
- color metadata,
- HDR/SDR indicator,
- estimated bitrate,
- timebase,
- start time,
- rotation metadata,
- sample aspect ratio,
- stream disposition/default track metadata.

## 3. Compatibility Decision Tree

```text
1. Probe source media with ffprobe.
2. Normalize metadata into ReplayMediaInfo.
3. Check Apple-native compatibility.
4. If container and streams are compatible:
   - use Apple-native compatible path.
5. If container is unsupported but streams are compatible:
   - attempt remux into compatible intermediate.
6. If video codec is unsupported:
   - generate editing-friendly intermediate or proxy.
7. If audio codec is unsupported:
   - transcode audio to AAC.
8. If HDR, unusual color, VFR, multiple audio tracks, rotation, or SAR exists:
   - warn, apply deterministic policy, or request user choice where feasible.
9. If fallback conversion fails:
   - mark source unsupported with actionable diagnostics.
```

## 4. Render Pipeline

### Conceptual Flow

```text
Source video frames
  ↓
Decode / read frames
  ↓
Place source into output canvas region
  ↓
Render overlay RGBA frame through shared renderer
  ↓
Alpha composite overlay onto canvas
  ↓
Encode video
  ↓
Handle audio passthrough or re-encode
  ↓
Mux output
```

## 5. Source Video Quality Rules

- Preserve source dimensions where possible.
- Use larger canvas instead of shrinking source video when feasible.
- Avoid fractional scaling.
- Avoid unnecessary color conversion.
- Record color/HDR limitations.
- Do not promise mathematically lossless output.
- Compare cropped output source region against input frames in tests where feasible.

## 6. Audio Policy

### v1.0 Policy

1. If source is MP4/MOV with AAC audio and no audio edits are required, attempt passthrough where technically safe.
2. If passthrough is unsafe or fails, re-encode to AAC.
3. If source audio is not AAC, re-encode to AAC.
4. If source has no audio, emit diagnostic and export silent video.
5. If multiple audio tracks exist, select primary/default track in v1.0 and emit diagnostic.

### Diagnostics

Record:

- selected audio track,
- passthrough attempted,
- passthrough succeeded/failed,
- fallback used,
- codec and bitrate,
- audio/video sync checks.

## 7. Preview vs Final Export

Preview and final export differ by execution profile, not semantic overlay behavior.

### Preview

- short segment,
- lower bitrate,
- faster settings,
- optional lower quality encoder settings,
- same semantic render plan,
- same overlay renderer.

### Final Export

- full or selected duration,
- high-quality upload preset,
- quality-oriented bitrate or CRF equivalent,
- stable diagnostics,
- render history entry.

## 8. v1.0 Media Support Target

| Category | v1.0 Target |
|---|---|
| Primary containers | MP4, MOV |
| Additional containers | diagnostic/fallback best effort |
| Primary video codecs | H.264, HEVC |
| Primary audio codec | AAC |
| Audio passthrough | Attempt for compatible AAC where safe |
| Audio fallback | AAC re-encode |
| Primary source sizes | 720p and 1080p |
| Frame rate | CFR preferred, VFR diagnosed |
| HDR | detect and warn, not full guarantee |
| Multiple audio tracks | select primary/default with diagnostic |
| Subtitles | not v1.0 |
| Alpha video | not v1.0 |

## 9. Render Backend Risks

### AVFoundation-only risk

AVFoundation may not handle all archive files, containers, timebases, or color cases. Mitigation: probe first and keep FFmpeg fallback path.

### Frame-by-frame rendering cost

Full-frame composition can be expensive. Mitigation: start with correctness, then profile caching, GPU composition, or tile-based overlay if required.

### Color metadata risk

Incorrect color/range conversion can visibly alter source. Mitigation: capture metadata and add visual tests.

### Audio muxing risk

Audio passthrough may be unsafe in some composed video paths. Mitigation: fallback to AAC re-encode and diagnose.

## 10. Phase 0 Media Questions

- Which frame composition path is stable enough for v1.0?
- Can overlay RGBA frames be composited without alpha artifacts?
- Can progress and cancellation be implemented cleanly?
- How should VFR be represented in diagnostics?
- Is AAC passthrough practical with the selected video export path?
- Which FFmpeg distribution model is viable?
