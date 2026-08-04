use image::{imageops, Rgba, RgbaImage};
use imageproc::drawing::{draw_filled_ellipse_mut, draw_filled_rect_mut};
use imageproc::rect::Rect;

/// Pastes `top` onto `base` at (x, y), alpha-blending pixel by pixel.
pub fn paste(base: &mut RgbaImage, top: &RgbaImage, x: i64, y: i64) {
    imageops::overlay(base, top, x, y);
}

/// Replaces the image's alpha channel with an ellipse inscribed in its bounding box
/// (mirrors Python's `ellipse()`: circular/oval crop via alpha masking).
pub fn ellipse_mask(mut img: RgbaImage) -> RgbaImage {
    let (w, h) = img.dimensions();
    let mut mask = RgbaImage::from_pixel(w, h, Rgba([0, 0, 0, 0]));
    let cx = w as i32 / 2;
    let cy = h as i32 / 2;
    draw_filled_ellipse_mut(&mut mask, (cx, cy), w as i32 / 2, h as i32 / 2, Rgba([255, 255, 255, 255]));

    for (px, mpx) in img.pixels_mut().zip(mask.pixels()) {
        px.0[3] = ((px.0[3] as u16 * mpx.0[3] as u16) / 255) as u8;
    }
    img
}

/// Replaces the image's alpha channel with a rounded-rectangle mask
/// (mirrors Python's `apply_rounded_borders()`).
pub fn rounded_mask(mut img: RgbaImage, radius: i32) -> RgbaImage {
    let (w, h) = img.dimensions();
    let mut mask = RgbaImage::from_pixel(w, h, Rgba([0, 0, 0, 0]));
    let white = Rgba([255, 255, 255, 255]);
    let r = radius.clamp(0, (w.min(h) / 2) as i32);

    // center cross (covers the rectangle minus the 4 rounded corners)
    draw_filled_rect_mut(&mut mask, Rect::at(r, 0).of_size((w as i32 - 2 * r).max(0) as u32, h), white);
    draw_filled_rect_mut(&mut mask, Rect::at(0, r).of_size(w, (h as i32 - 2 * r).max(0) as u32), white);
    // 4 corners
    for (cx, cy) in [(r, r), (w as i32 - r, r), (r, h as i32 - r), (w as i32 - r, h as i32 - r)] {
        draw_filled_ellipse_mut(&mut mask, (cx, cy), r, r, white);
    }

    for (px, mpx) in img.pixels_mut().zip(mask.pixels()) {
        px.0[3] = ((px.0[3] as u16 * mpx.0[3] as u16) / 255) as u8;
    }
    img
}

/// Builds a standalone rounded-rectangle filled with `color` (can be translucent).
/// Useful to stamp panels/bars/pills onto a canvas via `paste()` (alpha-blended).
pub fn rounded_rect_image(w: u32, h: u32, radius: i32, color: Rgba<u8>) -> RgbaImage {
    let solid = RgbaImage::from_pixel(w, h, color);
    rounded_mask(solid, radius)
}

/// Draws a linear gradient inside `rect` = ((x, y), (w, h)) using the given color stops.
pub fn draw_gradient(base: &mut RgbaImage, rect: ((i32, i32), (u32, u32)), colors: &[[u8; 3]], vertical: bool) {
    let ((x0, y0), (w, h)) = rect;
    if colors.len() < 2 {
        return;
    }
    let steps = if vertical { h } else { w }.max(1);

    for i in 0..steps {
        let t = i as f32 / steps as f32 * (colors.len() - 1) as f32;
        let idx = t.floor() as usize;
        let idx = idx.min(colors.len() - 2);
        let ratio = t - idx as f32;

        let a = colors[idx];
        let b = colors[idx + 1];
        let r = (a[0] as f32 * (1.0 - ratio) + b[0] as f32 * ratio) as u8;
        let g = (a[1] as f32 * (1.0 - ratio) + b[1] as f32 * ratio) as u8;
        let bch = (a[2] as f32 * (1.0 - ratio) + b[2] as f32 * ratio) as u8;
        let color = Rgba([r, g, bch, 255]);

        if vertical {
            draw_filled_rect_mut(base, Rect::at(x0, y0 + i as i32).of_size(w, 1), color);
        } else {
            draw_filled_rect_mut(base, Rect::at(x0 + i as i32, y0).of_size(1, h), color);
        }
    }
}

/// Simple k-means over a downsampled copy of the image, returning the `k` most
/// representative colors ordered by (cluster size / closeness to other clusters).
/// Lightweight on purpose: this runs on every welcomecard/walletcard/thisis request.
pub fn dominant_colors(img: &RgbaImage, k: usize) -> Vec<[u8; 3]> {
    let small = imageops::resize(img, (img.width() / 4).max(1), (img.height() / 4).max(1), imageops::FilterType::Nearest);
    let pixels: Vec<[f32; 3]> = small.pixels().map(|p| [p[0] as f32, p[1] as f32, p[2] as f32]).collect();
    if pixels.is_empty() {
        return vec![[0, 0, 0]; k];
    }

    // seed centroids evenly across the pixel list (deterministic, no RNG needed)
    let mut centroids: Vec<[f32; 3]> = (0..k)
        .map(|i| pixels[i * pixels.len() / k.max(1)])
        .collect();

    for _ in 0..12 {
        let mut sums = vec![[0f32; 3]; k];
        let mut counts = vec![0u32; k];

        for p in &pixels {
            let (mut best, mut best_dist) = (0, f32::MAX);
            for (ci, c) in centroids.iter().enumerate() {
                let d = dist2(p, c);
                if d < best_dist {
                    best_dist = d;
                    best = ci;
                }
            }
            sums[best][0] += p[0];
            sums[best][1] += p[1];
            sums[best][2] += p[2];
            counts[best] += 1;
        }

        let mut changed = false;
        for i in 0..k {
            if counts[i] == 0 {
                continue;
            }
            let new_c = [sums[i][0] / counts[i] as f32, sums[i][1] / counts[i] as f32, sums[i][2] / counts[i] as f32];
            if dist2(&new_c, &centroids[i]) > 1.0 {
                changed = true;
            }
            centroids[i] = new_c;
        }
        if !changed {
            break;
        }
    }

    centroids.iter().map(|c| [c[0] as u8, c[1] as u8, c[2] as u8]).collect()
}

fn dist2(a: &[f32; 3], b: &[f32; 3]) -> f32 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
}
