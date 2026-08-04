use serde::Deserialize;
use std::collections::HashMap;

/// static/gifs.json shape: { "ANGRY": ["url1", "url2", ...], "BAKA": [...], ... }
/// Keys are uppercase, mirroring the original `static/gifs.py` Collection.
#[derive(Deserialize)]
pub struct GifCollection {
    #[serde(flatten)]
    pub data: HashMap<String, Vec<String>>,
}

impl GifCollection {
    pub fn load(file: &str) -> anyhow::Result<Self> {
        let raw = std::fs::read_to_string(file)
            .map_err(|e| anyhow::anyhow!("Could not read {file}: {e}"))?;
        let collection: GifCollection = serde_json::from_str(&raw)?;
        Ok(collection)
    }

    pub fn get(&self, style: &str) -> Option<&Vec<String>> {
        self.data.get(&style.to_uppercase())
    }
}
