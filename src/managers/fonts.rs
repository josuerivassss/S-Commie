use rusttype::Font;
use std::{collections::HashMap, path::Path};

/// Holds every typeface found in `static/fonts`, loaded once at startup.
/// File naming convention: `<Family>.ttf` (style = "Regular") or `<Family>_<Style>.ttf`.
pub struct FontManager {
    fonts: HashMap<String, HashMap<String, Font<'static>>>,
}

impl FontManager {
    pub fn load(dir: &str) -> anyhow::Result<Self> {
        let mut fonts: HashMap<String, HashMap<String, Font<'static>>> = HashMap::new();
        let path = Path::new(dir);
        if !path.is_dir() {
            anyhow::bail!("Fonts directory not found: {dir}");
        }

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();
            let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
            if ext != "ttf" && ext != "otf" {
                continue;
            }
            let stem = file_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let mut parts = stem.splitn(2, '_');
            let family = parts.next().unwrap_or(stem).to_string();
            let style = parts.next().unwrap_or("Regular").to_string();

            let bytes = std::fs::read(&file_path)?;
            let font = Font::try_from_vec(bytes)
                .ok_or_else(|| anyhow::anyhow!("Invalid font file: {}", file_path.display()))?;
            fonts.entry(family).or_default().insert(style, font);
        }

        Ok(Self { fonts })
    }

    /// Fetch a typeface by family + style. Cloning a `rusttype::Font` is cheap (Arc-backed).
    pub fn fetch(&self, name: &str, style: &str) -> anyhow::Result<Font<'static>> {
        self.fonts
            .get(name)
            .and_then(|styles| styles.get(style))
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Typeface {name} with style {style} not found"))
    }
}
