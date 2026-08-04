use crate::managers::{EmojiCache, FontManager, GifCollection, LocalImagesManager};
use ocrs::{OcrEngine, OcrEngineParams};
use rten::Model;
use std::{path::Path, sync::Arc, time::Duration};

/// Cloneable handle shared across every request (all heavy data is loaded once
/// at startup and wrapped in `Arc`, so cloning `AppState` is just a few pointer copies).
#[derive(Clone)]
pub struct AppState {
    pub http: reqwest::Client,
    pub fonts: Arc<FontManager>,
    pub images: Arc<LocalImagesManager>,
    pub gifs: Arc<GifCollection>,
    pub emojis: Arc<EmojiCache>,
    /// `None` when the .rten model files aren't present in `static/models/` —
    /// in that case `/json/ocr` returns a clean 500 instead of the server refusing to start.
    pub ocr: Option<Arc<OcrEngine>>,
}

impl AppState {
    pub fn init() -> anyhow::Result<Self> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("SCOMMIE/3.0 (+https://github.com/)")
            .build()?;

        Ok(Self {
            http,
            fonts: Arc::new(FontManager::load("static/fonts")?),
            images: Arc::new(LocalImagesManager::load("static/assets")?),
            gifs: Arc::new(GifCollection::load("static/gifs.json")?),
            emojis: Arc::new(EmojiCache::new()),
            ocr: load_ocr_engine(),
        })
    }
}

/// Loads the OCR engine from `static/models/{text-detection,text-recognition}.rten`.
/// Missing/broken models are logged and degrade to `None` rather than crashing startup.
fn load_ocr_engine() -> Option<Arc<OcrEngine>> {
    let detection_path = "static/models/text-detection.rten";
    let recognition_path = "static/models/text-recognition.rten";

    if !Path::new(detection_path).exists() || !Path::new(recognition_path).exists() {
        tracing::warn!(
            "OCR models not found in static/models/ (need text-detection.rten and \
             text-recognition.rten) — /json/ocr will return 500 until you add them"
        );
        return None;
    }

    let engine = (|| -> anyhow::Result<OcrEngine> {
        let detection_model = Model::load_file(detection_path)?;
        let recognition_model = Model::load_file(recognition_path)?;
        let engine = OcrEngine::new(OcrEngineParams {
            detection_model: Some(detection_model),
            recognition_model: Some(recognition_model),
            ..Default::default()
        })?;
        Ok(engine)
    })();

    match engine {
        Ok(engine) => {
            tracing::info!("OCR engine loaded");
            Some(Arc::new(engine))
        }
        Err(e) => {
            tracing::error!("Failed to load OCR models: {e}");
            None
        }
    }
}