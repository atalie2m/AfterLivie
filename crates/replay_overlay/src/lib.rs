use replay_core::{
    visible_comments, Diagnostic, DiagnosticCategory, DiagnosticSeverity, SemanticRenderPlan,
    SourceRef, VisibilityQuery,
};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use tiny_skia::{Paint, Pixmap, Rect, Transform};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlayRenderReport {
    pub width: u32,
    pub height: u32,
    pub time_ms: i64,
    pub visible_comment_count: usize,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug)]
struct RendererFont {
    font: Option<fontdue::Font>,
    diagnostics: Vec<Diagnostic>,
}

pub fn render_overlay_png(
    plan: &SemanticRenderPlan,
    time_ms: i64,
    output_path: impl AsRef<Path>,
) -> anyhow::Result<OverlayRenderReport> {
    let (rgba, report) = render_overlay_rgba(plan, time_ms);
    write_png(output_path, plan.canvas.width, plan.canvas.height, &rgba)?;
    Ok(report)
}

pub fn render_overlay_rgba(
    plan: &SemanticRenderPlan,
    time_ms: i64,
) -> (Vec<u8>, OverlayRenderReport) {
    let mut diagnostics = Vec::new();
    let mut pixmap = Pixmap::new(plan.canvas.width, plan.canvas.height)
        .expect("render plan canvas dimensions must be non-zero");

    let panel = plan.regions.comments;
    fill_rect(
        &mut pixmap,
        panel.x,
        panel.y,
        panel.width,
        panel.height,
        parse_color(&plan.style.panel_background).unwrap_or([21, 23, 25, 230]),
    );

    let font = load_font();
    diagnostics.extend(font.diagnostics.clone());

    let visible = visible_comments(
        &plan.comments,
        &plan.timeline.sources,
        VisibilityQuery {
            time_ms,
            comment_window_ms: plan.timeline.comment_window.duration_ms,
            max_visible_comments: plan.timeline.max_visible_comments,
            global_offset_ms: plan.timeline.global_offset_ms,
        },
    );

    let mut y = panel.y + plan.style.row_padding;
    let max_text_width = panel.width.saturating_sub(plan.style.row_padding * 2);
    for (index, comment) in visible.iter().enumerate() {
        let author = comment
            .author
            .as_ref()
            .map(|author| author.display_name.as_str())
            .unwrap_or("Unknown");
        let author_color = plan
            .style
            .author_colors
            .get(index % plan.style.author_colors.len().max(1))
            .and_then(|value| parse_color(value))
            .unwrap_or([110, 231, 183, 255]);

        let author_text = truncate_graphemes(author, 22);
        let text = format!("{author_text}: {}", comment.body.text);
        let wrapped = wrap_text(&text, max_text_width, plan.style.font_size);
        let line_height = (plan.style.font_size * 1.35).ceil() as u32;
        let row_height = (wrapped.len() as u32 * line_height) + plan.style.row_padding;
        if y + row_height > panel.y + panel.height {
            break;
        }

        fill_rect(
            &mut pixmap,
            panel.x,
            y + row_height.saturating_sub(1),
            panel.width,
            1,
            parse_color(&plan.style.row_separator).unwrap_or([45, 51, 58, 153]),
        );

        let mut line_y = y + (plan.style.row_padding / 2) + line_height - 4;
        for (line_index, line) in wrapped.iter().enumerate() {
            let color = if line_index == 0 {
                author_color
            } else {
                parse_color(&plan.style.text_primary).unwrap_or([245, 247, 250, 255])
            };
            draw_text(
                &mut pixmap,
                &font,
                line,
                panel.x + plan.style.row_padding,
                line_y,
                plan.style.font_size,
                color,
            );
            line_y += line_height;
        }
        y += row_height;
    }

    if font.font.is_none() {
        diagnostics.push(
            Diagnostic::new(
                DiagnosticSeverity::Warning,
                DiagnosticCategory::Renderer,
                "renderer.placeholder_text",
                "Overlay frame was rendered with deterministic placeholder glyphs because no usable system font was found.",
            )
            .with_source_ref(SourceRef::RendererComponent {
                component: "text_rasterizer".into(),
            })
            .with_hint("Install a standard TrueType/OpenType sans-serif font or configure the font policy."),
        );
    }

    let data = pixmap.data().to_vec();
    (
        data,
        OverlayRenderReport {
            width: plan.canvas.width,
            height: plan.canvas.height,
            time_ms,
            visible_comment_count: visible.len(),
            diagnostics,
        },
    )
}

fn load_font() -> RendererFont {
    let candidates = [
        "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        "/System/Library/Fonts/Supplemental/Arial.ttf",
        "/System/Library/Fonts/Supplemental/Helvetica.ttf",
        "/Library/Fonts/Arial.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/dejavu/DejaVuSans.ttf",
    ];

    for candidate in candidates {
        if let Ok(bytes) = std::fs::read(candidate) {
            if let Ok(font) = fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default()) {
                return RendererFont {
                    font: Some(font),
                    diagnostics: vec![],
                };
            }
        }
    }

    RendererFont {
        font: None,
        diagnostics: vec![Diagnostic::new(
            DiagnosticSeverity::Warning,
            DiagnosticCategory::Renderer,
            "renderer.font_not_found",
            "No usable system font was found for CPU overlay rendering.",
        )
        .with_hint("Configure a font policy or install a standard sans-serif font.")],
    }
}

fn wrap_text(text: &str, max_width: u32, font_size: f32) -> Vec<String> {
    let max_units = (max_width as f32 / (font_size * 0.56)).max(4.0) as usize;
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut units = 0_usize;

    for grapheme in UnicodeSegmentation::graphemes(text, true) {
        let width = if grapheme.is_ascii() { 1 } else { 2 };
        if units + width > max_units && !current.is_empty() {
            lines.push(current.trim_end().to_string());
            current.clear();
            units = 0;
        }
        current.push_str(grapheme);
        units += width;
    }
    if !current.is_empty() {
        lines.push(current.trim_end().to_string());
    }
    lines.truncate(3);
    let original_count = UnicodeSegmentation::graphemes(text, true).count();
    let rendered_count = lines
        .iter()
        .map(|line| UnicodeSegmentation::graphemes(line.as_str(), true).count())
        .sum::<usize>();
    if rendered_count < original_count {
        if let Some(last) = lines.last_mut() {
            *last = format!(
                "{}...",
                truncate_graphemes(last, max_units.saturating_sub(3))
            );
        }
    }
    lines
}

fn truncate_graphemes(text: &str, max: usize) -> String {
    let graphemes = UnicodeSegmentation::graphemes(text, true).collect::<Vec<_>>();
    if graphemes.len() <= max {
        text.to_string()
    } else {
        format!("{}...", graphemes[..max.saturating_sub(3)].concat())
    }
}

fn fill_rect(pixmap: &mut Pixmap, x: u32, y: u32, width: u32, height: u32, color: [u8; 4]) {
    if width == 0 || height == 0 {
        return;
    }
    let Some(rect) = Rect::from_xywh(x as f32, y as f32, width as f32, height as f32) else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_color_rgba8(color[0], color[1], color[2], color[3]);
    pixmap.fill_rect(rect, &paint, Transform::identity(), None);
}

fn draw_text(
    pixmap: &mut Pixmap,
    renderer_font: &RendererFont,
    text: &str,
    x: u32,
    baseline_y: u32,
    font_size: f32,
    color: [u8; 4],
) {
    let Some(font) = renderer_font.font.as_ref() else {
        draw_placeholder_text(pixmap, text, x, baseline_y, font_size, color);
        return;
    };

    let mut cursor_x = x as i32;
    for grapheme in UnicodeSegmentation::graphemes(text, true) {
        let mut chars = grapheme.chars();
        let Some(ch) = chars.next() else {
            continue;
        };
        if chars.next().is_some() || ch.is_control() {
            draw_placeholder_glyph(pixmap, cursor_x, baseline_y as i32, font_size, color);
            cursor_x += (font_size * 0.9) as i32;
            continue;
        }
        let (metrics, bitmap) = font.rasterize(ch, font_size);
        if metrics.width == 0 || metrics.height == 0 {
            cursor_x += metrics.advance_width.ceil() as i32;
            continue;
        }

        let origin_x = cursor_x + metrics.xmin;
        let origin_y = baseline_y as i32 - metrics.height as i32 - metrics.ymin;
        for glyph_y in 0..metrics.height {
            for glyph_x in 0..metrics.width {
                let alpha = bitmap[glyph_y * metrics.width + glyph_x];
                if alpha == 0 {
                    continue;
                }
                blend_pixel(
                    pixmap,
                    origin_x + glyph_x as i32,
                    origin_y + glyph_y as i32,
                    color,
                    alpha,
                );
            }
        }
        cursor_x += metrics.advance_width.ceil() as i32;
    }
}

fn draw_placeholder_text(
    pixmap: &mut Pixmap,
    text: &str,
    x: u32,
    baseline_y: u32,
    font_size: f32,
    color: [u8; 4],
) {
    let mut cursor_x = x;
    let height = (font_size * 0.7) as u32;
    for grapheme in UnicodeSegmentation::graphemes(text, true) {
        let width = if grapheme.is_ascii() {
            (font_size * 0.35) as u32
        } else {
            (font_size * 0.75) as u32
        }
        .max(4);
        fill_rect(
            pixmap,
            cursor_x,
            baseline_y.saturating_sub(height),
            width,
            height,
            [color[0], color[1], color[2], color[3].min(210)],
        );
        cursor_x += width + 2;
    }
}

fn draw_placeholder_glyph(
    pixmap: &mut Pixmap,
    x: i32,
    baseline_y: i32,
    font_size: f32,
    color: [u8; 4],
) {
    let size = (font_size * 0.75) as u32;
    if x < 0 || baseline_y < 0 {
        return;
    }
    fill_rect(
        pixmap,
        x as u32,
        (baseline_y as u32).saturating_sub(size),
        size,
        size,
        [color[0], color[1], color[2], color[3].min(180)],
    );
}

fn blend_pixel(pixmap: &mut Pixmap, x: i32, y: i32, color: [u8; 4], glyph_alpha: u8) {
    if x < 0 || y < 0 || x as u32 >= pixmap.width() || y as u32 >= pixmap.height() {
        return;
    }
    let idx = ((y as u32 * pixmap.width() + x as u32) * 4) as usize;
    let data = pixmap.data_mut();
    let src_alpha = (glyph_alpha as f32 / 255.0) * (color[3] as f32 / 255.0);
    let dst_alpha = data[idx + 3] as f32 / 255.0;
    let out_alpha = src_alpha + dst_alpha * (1.0 - src_alpha);
    if out_alpha <= f32::EPSILON {
        return;
    }

    for channel in 0..3 {
        let src = color[channel] as f32 / 255.0;
        let dst = data[idx + channel] as f32 / 255.0;
        let out = (src * src_alpha + dst * dst_alpha * (1.0 - src_alpha)) / out_alpha;
        data[idx + channel] = (out * 255.0).round() as u8;
    }
    data[idx + 3] = (out_alpha * 255.0).round() as u8;
}

fn parse_color(value: &str) -> Option<[u8; 4]> {
    let hex = value.strip_prefix('#')?;
    let (rgb, alpha) = match hex.len() {
        6 => (hex, "FF"),
        8 => hex.split_at(6),
        _ => return None,
    };
    let r = u8::from_str_radix(&rgb[0..2], 16).ok()?;
    let g = u8::from_str_radix(&rgb[2..4], 16).ok()?;
    let b = u8::from_str_radix(&rgb[4..6], 16).ok()?;
    let a = u8::from_str_radix(alpha, 16).ok()?;
    Some([r, g, b, a])
}

fn write_png(path: impl AsRef<Path>, width: u32, height: u32, rgba: &[u8]) -> anyhow::Result<()> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut png_writer = encoder.write_header()?;
    png_writer.write_image_data(rgba)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use replay_core::default_sidebar_plan;

    #[test]
    fn wraps_without_splitting_emoji_grapheme() {
        let lines = wrap_text("hello 👩‍💻 world", 80, 18.0);
        assert!(lines.iter().any(|line| line.contains("👩‍💻")));
    }

    #[test]
    fn renders_png_smoke() {
        let plan = default_sidebar_plan(320, 180, 30.0, vec![], vec![]);
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("overlay.png");
        let report = render_overlay_png(&plan, 0, &output).unwrap();
        assert_eq!(report.width, 800);
        assert!(output.exists());
    }
}
