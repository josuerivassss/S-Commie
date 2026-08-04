use image::RgbaImage;
use std::{collections::HashMap, path::Path};

/// Holds every local overlay/background image found in `static/assets`, loaded once at startup.
pub struct LocalImagesManager {
    images: HashMap<String, RgbaImage>,
}

impl LocalImagesManager {
    pub fn load(dir: &str) -> anyhow::Result<Self> {
        let mut images = HashMap::new();
        let path = Path::new(dir);
        if !path.is_dir() {
            anyhow::bail!("Assets directory not found: {dir}");
        }

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();
            let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
            if !["png", "jpg", "jpeg"].contains(&ext.as_str()) {
                continue;
            }
            let name = file_path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
            let img = image::open(&file_path)?.to_rgba8();
            images.insert(name, img);
        }

        Ok(Self { images })
    }

    /// Returns a clone (cheap-ish, avoids mutating the cached original).
    pub fn fetch(&self, name: &str) -> anyhow::Result<RgbaImage> {
        self.images
            .get(name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Image not found: {name}"))
    }
}
