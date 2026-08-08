use image::{Rgba, RgbaImage};
use imageproc::drawing::{draw_filled_ellipse_mut, draw_filled_rect_mut};
use imageproc::rect::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresenceStatus {
    Online,
    Dnd,
    Idle,
    Offline,
    Streaming,
}

impl PresenceStatus {
    pub fn from_str(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "dnd" => PresenceStatus::Dnd,
            "idle" => PresenceStatus::Idle,
            "offline" => PresenceStatus::Offline,
            "streaming" => PresenceStatus::Streaming,
            _ => PresenceStatus::Online,
        }
    }

    fn color(&self) -> Rgba<u8> {
        match self {
            PresenceStatus::Online => Rgba([35, 165, 90, 255]),
            PresenceStatus::Dnd => Rgba([242, 63, 66, 255]),
            PresenceStatus::Idle => Rgba([240, 178, 50, 255]),
            PresenceStatus::Offline => Rgba([128, 132, 142, 255]),
            PresenceStatus::Streaming => Rgba([89, 54, 149, 255]),
        }
    }
}

/// Draws a Discord-style presence badge centered at `center`. `background`
/// is painted as a ring around the badge first, mirroring how Discord cuts
/// the badge into whatever sits behind it (avatar/banner).
pub fn draw_status_badge(canvas: &mut RgbaImage, status: PresenceStatus, center: (i32, i32), radius: i32, background: Rgba<u8>) {
    let border = (radius as f32 * 0.28).ceil() as i32;
    draw_filled_ellipse_mut(canvas, center, radius + border, radius + border, background);
    draw_filled_ellipse_mut(canvas, center, radius, radius, status.color());

    match status {
        PresenceStatus::Offline => {
            let inner = (radius as f32 * 0.55) as i32;
            draw_filled_ellipse_mut(canvas, center, inner, inner, background);
        }
        PresenceStatus::Idle => {
            let bite_radius = (radius as f32 * 0.82) as i32;
            let offset = (radius as f32 * 0.55) as i32;
            draw_filled_ellipse_mut(canvas, (center.0 - offset, center.1 - offset), bite_radius, bite_radius, background);
        }
        PresenceStatus::Dnd => {
            let bar_w = (radius as f32 * 1.1) as i32;
            let bar_h = (radius as f32 * 0.32).max(2.0) as i32;
            draw_filled_rect_centered(canvas, center, bar_w, bar_h, Rgba([255, 255, 255, 255]));
        }
        PresenceStatus::Streaming => {
            draw_play_triangle(canvas, center, (radius as f32 * 0.9) as i32, Rgba([255, 255, 255, 255]));
        }
        PresenceStatus::Online => {}
    }
}

fn draw_filled_rect_centered(canvas: &mut RgbaImage, center: (i32, i32), w: i32, h: i32, color: Rgba<u8>) {
    let x = center.0 - w / 2;
    let y = center.1 - h / 2;
    draw_filled_rect_mut(canvas, Rect::at(x, y).of_size(w.max(1) as u32, h.max(1) as u32), color);
}

/// Rasterizes a small filled triangle via barycentric fill -- avoids
/// depending on imageproc's filled-polygon API, unavailable in the pinned
/// 0.23 release. Cheap: badge icons are tiny (~20px), so a per-pixel scan
/// over the bounding box costs nothing measurable.
fn draw_play_triangle(canvas: &mut RgbaImage, center: (i32, i32), size: i32, color: Rgba<u8>) {
    let half = size / 2;
    let p0 = (center.0 - half / 2, center.1 - half);
    let p1 = (center.0 - half / 2, center.1 + half);
    let p2 = (center.0 + half, center.1);

    let min_x = p0.0.min(p1.0).min(p2.0).max(0);
    let max_x = p0.0.max(p1.0).max(p2.0).min(canvas.width() as i32 - 1);
    let min_y = p0.1.min(p1.1).min(p2.1).max(0);
    let max_y = p0.1.max(p1.1).max(p2.1).min(canvas.height() as i32 - 1);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if point_in_triangle((x, y), p0, p1, p2) {
                canvas.put_pixel(x as u32, y as u32, color);
            }
        }
    }
}

fn sign(p1: (i32, i32), p2: (i32, i32), p3: (i32, i32)) -> i32 {
    (p1.0 - p3.0) * (p2.1 - p3.1) - (p2.0 - p3.0) * (p1.1 - p3.1)
}

fn point_in_triangle(pt: (i32, i32), v1: (i32, i32), v2: (i32, i32), v3: (i32, i32)) -> bool {
    let d1 = sign(pt, v1, v2);
    let d2 = sign(pt, v2, v3);
    let d3 = sign(pt, v3, v1);
    let has_neg = d1 < 0 || d2 < 0 || d3 < 0;
    let has_pos = d1 > 0 || d2 > 0 || d3 > 0;
    !(has_neg && has_pos)
}