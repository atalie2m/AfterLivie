# 01. Product Proposal v2

## 1. Executive Summary

Native Livestream Archive Replay Composer is a macOS-first native application for transforming local livestream archive videos and prepared timestamped comment files into synchronized replay videos.

The product restores the experience of watching a livestream together with its live comments, platform metadata, pinned messages, contextual labels, and replay-oriented overlays. It is not a general-purpose video editor and should not become one.

The first credible release should focus on producing a high-quality single-source replay from:

- one local archived video,
- one prepared timestamped comment file in canonical JSON,
- optional metadata,
- a controlled layout template,
- deterministic render settings.

The application should then synchronize comments with the video timeline, compose the source video into a larger canvas, render a comment panel using a shared overlay renderer, and export a replay video suitable for viewing, sharing, research, or preservation.

## 2. Core Product Thesis

Livestream archives are incomplete when they preserve only video and lose the synchronized social context around that video. Comments, reactions, pinned messages, donation events, platform badges, and audience timing often explain why moments in the video mattered.

A focused replay composer can solve this better than generic video editors, screen recording workflows, or platform-specific tools because it treats comments as timeline events rather than static visual assets.

## 3. Primary Users

### v1.0 Primary Users

v1.0 should target technically capable users who already have a local video and a prepared timestamped comment file.

Typical examples:

- archivists,
- researchers,
- technically capable creators,
- fan archive maintainers,
- event documentation operators.

These users will tolerate a controlled workflow if the output is reliable, deterministic, and high quality.

### v1.x Expansion Users

After v1.0, the product can broaden toward:

- streamers,
- video editors,
- multi-platform stream operators,
- educators,
- less technical creators.

This expansion requires better import ergonomics, more templates, richer styling, and multi-source sync UI.

## 4. Product Goals

### 4.1 v1.0 Goals

1. Create a high-quality replay video from one local video and one comment file.
2. Preserve source video clarity by avoiding unnecessary scaling and awkward canvas decisions.
3. Use a shared overlay renderer for all final comment and metadata overlay rendering.
4. Ensure preview/export overlay consistency.
5. Store projects in deterministic, portable, versioned formats.
6. Use Rust for core project, import, timeline, validation, and render-plan logic.
7. Use Swift/SwiftUI/AppKit for a native macOS document application.
8. Use FFmpeg/ffprobe for media probing and compatibility diagnostics.
9. Provide actionable diagnostics for import, media, renderer, and export failures.
10. Leave the internal model ready for multi-source merge without exposing full multi-source UX in v1.0.

### 4.2 v1.x Goals

1. Add CSV and JSONL importers.
2. Add multiple comment sources per project.
3. Expose per-source sync offsets.
4. Add manual multi-source sync review UI.
5. Add merged multi-platform comment rendering.
6. Add additional templates and style controls.
7. Improve archive and research export artifacts.
8. Add plugin protocol prototypes after importer and diagnostics stability.

## 5. v1.0 Non-Goals

v1.0 should explicitly exclude:

- direct YouTube, Twitch, or platform API import,
- automatic VOD download,
- cloud rendering,
- collaboration,
- Windows product release,
- mobile apps,
- full nonlinear video editing,
- arbitrary template scripting,
- third-party in-process binary plugins,
- advanced automatic comment alignment,
- fully platform-faithful chat recreation,
- mathematically lossless output guarantees.

## 6. v1.0 User Journey

1. User creates a new replay project.
2. User selects a local video file.
3. App probes the media file and reports compatibility.
4. User imports a canonical JSON comment file.
5. App normalizes comments and reports import diagnostics.
6. User chooses the default sidebar replay layout.
7. User adjusts a global sync offset if needed.
8. User previews a short segment.
9. User exports a final replay video.
10. App stores render diagnostics and project metadata.

## 7. Output Quality Principles

The product should not claim that output is mathematically lossless. The realistic quality target is:

- avoid unnecessary scaling,
- preserve source region at native size when layout allows,
- avoid fractional scaling,
- avoid avoidable color conversions,
- choose high-quality encoder defaults,
- keep overlay text crisp,
- make media decisions visible in diagnostics,
- make preview decisions trustworthy for export.

## 8. Product Differentiators

The product should differentiate through:

- comment-aware timeline model,
- high-quality shared overlay rendering,
- CJK and emoji robustness,
- deterministic project files,
- preview/export consistency,
- local-first workflows,
- import diagnostics,
- future multi-platform merge readiness,
- future Windows compatibility through shared core and renderer.

## 9. Long-Term End State

The long-term product can become a professional-quality livestream replay composition environment that supports multiple archive formats, multiple comment sources, custom layouts, importer plugins, batch processing, CLI workflows, and native macOS/Windows applications.

The route to that end state should pass through a narrow but strong v1.0 rather than a broad and fragile first release.
