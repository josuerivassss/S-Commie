use image::Rgba;

/// Parses a 3 or 6 digit hex color code (without '#') into an opaque RGBA color.
pub fn parse_hex(code: &str) -> Rgba<u8> {
    let c = code.trim_start_matches('#');
    let (r, g, b) = if c.len() == 3 {
        let mut chars = c.chars();
        let r = chars.next().unwrap_or('0');
        let g = chars.next().unwrap_or('0');
        let b = chars.next().unwrap_or('0');
        (
            u8::from_str_radix(&format!("{r}{r}"), 16).unwrap_or(0),
            u8::from_str_radix(&format!("{g}{g}"), 16).unwrap_or(0),
            u8::from_str_radix(&format!("{b}{b}"), 16).unwrap_or(0),
        )
    } else {
        (
            u8::from_str_radix(c.get(0..2).unwrap_or("00"), 16).unwrap_or(0),
            u8::from_str_radix(c.get(2..4).unwrap_or("00"), 16).unwrap_or(0),
            u8::from_str_radix(c.get(4..6).unwrap_or("00"), 16).unwrap_or(0),
        )
    };
    Rgba([r, g, b, 255])
}

/// Darkens a color by a given factor in [0, 1] (used for progress-bar track colors).
pub fn darken(color: Rgba<u8>, factor: f32) -> Rgba<u8> {
    let f = 1.0 - factor.clamp(0.0, 1.0);
    Rgba([
        (color[0] as f32 * f) as u8,
        (color[1] as f32 * f) as u8,
        (color[2] as f32 * f) as u8,
        255,
    ])
}
