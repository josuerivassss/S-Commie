use dashmap::DashMap;
use image::{imageops::FilterType, RgbaImage};
use std::sync::{atomic::{AtomicUsize, Ordering}, Arc};

const TWEMOJI_CDN: &str = "https://cdn.jsdelivr.net/gh/twitter/twemoji@latest/assets/72x72/";
/// Hard cap on distinct cached glyphs (codepoint x size). Each entry is a small
/// RGBA square (a handful of KB), so this bounds worst-case memory instead of
/// growing forever across arbitrary text inputs.
const MAX_CACHED_ENTRIES: usize = 500;

/// Lazily fetches and caches Twemoji PNGs. A miss/fetch-failure never errors the
/// caller — text rendering just skips that glyph, it never fails the whole request.
pub struct EmojiCache {
    cache: DashMap<String, Arc<RgbaImage>>,
    len: AtomicUsize,
}

impl EmojiCache {
    pub fn new() -> Self {
        Self { cache: DashMap::new(), len: AtomicUsize::new(0) }
    }

    /// Returns the emoji glyph for `ch` resized to `size`x`size`, or `None` if
    /// `ch` isn't a recognized emoji or the CDN fetch fails.
    pub async fn get(&self, http: &reqwest::Client, ch: char, size: u32) -> Option<Arc<RgbaImage>> {
        emojis::get(&ch.to_string())?;

        let key = format!("{:x}_{size}", ch as u32);
        if let Some(cached) = self.cache.get(&key) {
            return Some(cached.clone());
        }

        let codepoint = format!("{:x}", ch as u32);
        let bytes = http.get(format!("{TWEMOJI_CDN}{codepoint}.png")).send().await.ok()?.bytes().await.ok()?;
        let img = image::load_from_memory(&bytes).ok()?.to_rgba8();
        let img = image::imageops::resize(&img, size.max(1), size.max(1), FilterType::Lanczos3);
        let img = Arc::new(img);

        if self.len.load(Ordering::Relaxed) < MAX_CACHED_ENTRIES {
            self.cache.insert(key, img.clone());
            self.len.fetch_add(1, Ordering::Relaxed);
        }

        Some(img)
    }
}