use crate::managers::EmojiCache;
use image::{imageops, Rgba, RgbaImage};
use imageproc::drawing::draw_text_mut;
use rusttype::{point, Font, Scale};

/// Horizontal alignment for `draw()`. Center/Right need `TextStyle::align`'s `box_width`
/// to know what to align against; each line is measured & aligned independently.
pub enum Align {
    Left,
    Center,
    #[allow(dead_code)]
    Right,
}

/// Bundles every drawing option so `draw()` doesn't need a dozen positional args.
/// Build with `TextStyle::new(&font, size)` then chain the setters you need.
pub struct TextStyle<'a> {
    pub font: &'a Font<'a>,
    pub size: f32,
    pub color: Rgba<u8>,
    pub stroke_width: i32,
    pub stroke_color: Option<Rgba<u8>>,
    pub align: Align,
    pub box_width: Option<i32>,
    pub emoji_scale: f32,
}

impl<'a> TextStyle<'a> {
    pub fn new(font: &'a Font<'a>, size: f32) -> Self {
        Self {
            font,
            size,
            color: Rgba([255, 255, 255, 255]),
            stroke_width: 0,
            stroke_color: None,
            align: Align::Left,
            box_width: None,
            emoji_scale: 1.0,
        }
    }

    pub fn color(mut self, color: Rgba<u8>) -> Self {
        self.color = color;
        self
    }

    pub fn stroke(mut self, width: i32, color: Rgba<u8>) -> Self {
        self.stroke_width = width;
        self.stroke_color = Some(color);
        self
    }

    /// `box_width` is the width (starting at the `xy` passed to `draw`) that Center/Right
    /// align against — typically the canvas width when `xy.0 == 0`.
    pub fn align(mut self, align: Align, box_width: i32) -> Self {
        self.align = align;
        self.box_width = Some(box_width);
        self
    }
    #[allow(dead_code)]
    pub fn emoji_scale(mut self, scale: f32) -> Self {
        self.emoji_scale = scale;
        self
    }
}

/// Word-wraps `text` so no line exceeds `width` *characters* (Unicode scalar count,
/// not bytes — matters once emoji/accented text is involved). Mirrors `textwrap.fill`.
pub fn wrap(text: &str, width: usize) -> String {
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        let mut line = String::new();
        let mut line_len = 0usize;
        for word in paragraph.split_whitespace() {
            let word_len = word.chars().count();
            let candidate_len = if line.is_empty() { word_len } else { line_len + 1 + word_len };
            if candidate_len > width && !line.is_empty() {
                lines.push(std::mem::take(&mut line));
                line_len = 0;
            }
            if !line.is_empty() {
                line.push(' ');
                line_len += 1;
            }
            line.push_str(word);
            line_len += word_len;
        }
        lines.push(line);
    }
    lines.join("\n")
}

/// Splits a line into consecutive (run, is_emoji) segments so text and emoji glyphs
/// can be measured/drawn independently. Checked per Unicode scalar (not full grapheme
/// clusters), so a compound ZWJ/flag sequence may render as separate glyphs — an
/// accepted simplification for meme-style captions, same tradeoff the Python original made.
fn segment(line: &str) -> Vec<(String, bool)> {
    let mut segments = Vec::new();
    let mut buffer = String::new();
    for ch in line.chars() {
        if emojis::get(&ch.to_string()).is_some() {
            if !buffer.is_empty() {
                segments.push((std::mem::take(&mut buffer), false));
            }
            segments.push((ch.to_string(), true));
        } else {
            buffer.push(ch);
        }
    }
    if !buffer.is_empty() {
        segments.push((buffer, false));
    }
    segments
}

fn line_height(font: &Font, scale: Scale) -> i32 {
    let vm = font.v_metrics(scale);
    (vm.ascent - vm.descent + vm.line_gap).ceil() as i32
}

fn text_width(font: &Font, scale: Scale, text: &str) -> i32 {
    let glyphs: Vec<_> = font.layout(text, scale, point(0.0, 0.0)).collect();
    let width = glyphs
        .last()
        .map(|g| g.position().x + g.unpositioned().h_metrics().advance_width)
        .unwrap_or(0.0);
    width.ceil() as i32
}

/// Width of one line accounting for emoji cells (reserved at `emoji_px + 2` regardless
/// of whether the glyph ends up fetched successfully, so layout never shifts either way).
fn segment_line_width(font: &Font, scale: Scale, emoji_px: i32, line: &str) -> i32 {
    segment(line)
        .iter()
        .map(|(s, is_emoji)| if *is_emoji { emoji_px + 2 } else { text_width(font, scale, s) })
        .sum()
}

/// Measures a (possibly multiline, emoji-aware) block of text, returning (width, height) in pixels.
/// Safe to call without network access — emoji width is a fixed cell size, no fetch needed.
pub fn measure(font: &Font, size: f32, text: &str, emoji_scale: f32) -> (i32, i32) {
    let scale = Scale::uniform(size);
    let emoji_px = (size * emoji_scale) as i32;
    let lh = line_height(font, scale);
    let mut max_w = 0;
    let mut lines = 0;
    for line in text.split('\n') {
        max_w = max_w.max(segment_line_width(font, scale, emoji_px, line));
        lines += 1;
    }
    (max_w, lh * lines)
}

/// Draws (possibly multiline, emoji-aware) text at `xy` per `style`. Each line is
/// measured and aligned on its own, so mixed-width lines all center/right-align correctly.
/// Async because emoji glyphs are fetched (and cached) over HTTP on first use.
pub async fn draw(canvas: &mut RgbaImage, http: &reqwest::Client, emoji_cache: &EmojiCache, xy: (i32, i32), text: &str, style: &TextStyle<'_>) {
    let scale = Scale::uniform(style.size);
    let lh = line_height(style.font, scale);
    let emoji_px = (style.size * style.emoji_scale) as i32;

    for (i, line) in text.split('\n').enumerate() {
        let y = xy.1 + lh * i as i32;
        let line_w = segment_line_width(style.font, scale, emoji_px, line);

        let mut x = match (&style.align, style.box_width) {
            (Align::Center, Some(bw)) => xy.0 + (bw - line_w) / 2,
            (Align::Right, Some(bw)) => xy.0 + (bw - line_w),
            _ => xy.0,
        };

        for (seg, is_emoji) in segment(line) {
            if is_emoji {
                let ch = seg.chars().next().expect("emoji segment is never empty");
                if let Some(glyph) = emoji_cache.get(http, ch, emoji_px.max(1) as u32).await {
                    let gy = y + (style.size as i32 - emoji_px) / 2;
                    imageops::overlay(canvas, glyph.as_ref(), x as i64, gy as i64);
                }
                x += emoji_px + 2;
            } else {
                if style.stroke_width > 0 {
                    let stroke_color = style.stroke_color.unwrap_or(Rgba([0, 0, 0, 255]));
                    for dx in -style.stroke_width..=style.stroke_width {
                        for dy in -style.stroke_width..=style.stroke_width {
                            if dx == 0 && dy == 0 {
                                continue;
                            }
                            draw_text_mut(canvas, stroke_color, x + dx, y + dy, scale, style.font, &seg);
                        }
                    }
                }
                draw_text_mut(canvas, style.color, x, y, scale, style.font, &seg);
                x += text_width(style.font, scale, &seg);
            }
        }
    }
}