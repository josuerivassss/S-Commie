use crate::response::{ApiError, ApiOk, ApiResult};
use crate::extract::Query;
use once_cell::sync::Lazy;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

static RESPONSES: Lazy<HashMap<&'static str, Vec<&'static str>>> = Lazy::new(|| {
    HashMap::from([
        ("en", vec!["Yes.", "No.", "Maybe.", "Probably.", "Probably no.", "I don't know.", "Sure.", "Obviously no.", "I doubt it."]),
        ("es", vec!["Si.", "No.", "Tal vez.", "Probablemente.", "Probablemente no.", "No sé.", "Obvio si.", "Obvio no.", "Lo dudo."]),
        ("pt", vec!["Sim.", "Não.", "Talvez.", "Provavelmente.", "Provavelmente não.", "Não sei.", "Óbvio.", "Óbvio não.", "Duvido."]),
        ("fr", vec!["Oui", "Non.", "Peut-être.", "Probablement.", "Probablement non.", "Je ne sais pas.", "De toute évidence.", "Évidemment pas.", "J'en doute."]),
    ])
});

#[derive(Deserialize)]
pub struct EightBallQuery {
    text: String,
    #[serde(default = "default_idiom")]
    idiom: String,
}

fn default_idiom() -> String {
    "en".to_string()
}

#[derive(Serialize)]
struct EightBallOut {
    question: String,
    idiom: String,
    response: String,
}

/// GET /json/8ball?text=...&idiom=en|es|pt|fr
pub async fn handler(Query(q): Query<EightBallQuery>) -> ApiResult {
    if q.text.len() < 2 || q.text.len() > 1000 {
        return Err(ApiError::validation("text must be between 2 and 1000 characters", "text"));
    }
    let options = RESPONSES
        .get(q.idiom.as_str())
        .ok_or_else(|| ApiError::validation("idiom must be one of: en, es, pt, fr", "idiom"))?;

    let response = options.choose(&mut rand::thread_rng()).unwrap().to_string();
    Ok(ApiOk::new(EightBallOut { question: q.text, idiom: q.idiom, response }))
}
