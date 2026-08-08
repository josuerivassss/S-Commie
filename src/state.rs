use crate::managers::{EmojiCache, FontManager, GifCollection, LocalImagesManager};
use crate::throttle::Throttle;
use crate::apikeys::{ApiKeyCache, ApiKeyRepository, QuotaTracker, SharedApiKeyCache};
use crate::circuit::CircuitBreaker;
use ocrs::{OcrEngine, OcrEngineParams};
use rten::Model;
use std::{path::Path, sync::Arc, time::Duration};
use mongodb::Client as MongoClient;

/// Cloneable handle shared across every request (all heavy data is loaded once
/// at startup and wrapped in `Arc`, so cloning `AppState` is just a few pointer copies).
#[derive(Clone)]
pub struct AppState {
    pub http: reqwest::Client,
    pub fonts: Arc<FontManager>,
    pub images: Arc<LocalImagesManager>,
    pub gifs: Arc<GifCollection>,
    pub emojis: Arc<EmojiCache>,
    pub ocr: Option<Arc<OcrEngine>>,
    pub translate_throttle: Arc<Throttle>,
    pub api_keys: SharedApiKeyCache,
    pub api_key_repo: Arc<ApiKeyRepository>,
    pub quotas: Arc<QuotaTracker>,
    pub external_breakers: ExternalBreakers,
}

impl AppState {
    pub async fn init() -> anyhow::Result<Self> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("SCOMMIE/3.0 (+https://github.com/)")
            .build()?;

        let mongo_uri = std::env::var("MONGO_URI").unwrap_or_else(|_| "mongodb://localhost:27017".into());
        let mongo_db_name = std::env::var("MONGO_DB_NAME").unwrap_or_else(|_| "bcommie".into());
        let mongo_client = MongoClient::with_uri_str(&mongo_uri).await?;
        let api_key_repo = Arc::new(ApiKeyRepository::new(&mongo_client, &mongo_db_name));
        Ok(Self {
            http,
            fonts: Arc::new(FontManager::load("static/fonts")?),
            images: Arc::new(LocalImagesManager::load("static/assets")?),
            gifs: Arc::new(GifCollection::load("static/gifs.json")?),
            emojis: Arc::new(EmojiCache::new()),
            ocr: load_ocr_engine(),
            translate_throttle: Arc::new(Throttle::new(Duration::from_millis(500))),
            api_keys: Arc::new(ApiKeyCache::empty()),
            api_key_repo,
            quotas: Arc::new(QuotaTracker::new()),
            external_breakers: ExternalBreakers::new(),
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

#[derive(Clone)]
pub struct ExternalBreakers {
    pub translate: Arc<CircuitBreaker>,
    pub weather: Arc<CircuitBreaker>,
    pub imagesearch: Arc<CircuitBreaker>,
}

impl ExternalBreakers {
    fn new() -> Self {
        Self {
            translate: Arc::new(CircuitBreaker::new(5, 60)),
            weather: Arc::new(CircuitBreaker::new(3, 90)),
            imagesearch: Arc::new(CircuitBreaker::new(4, 120)),
        }
    }
}