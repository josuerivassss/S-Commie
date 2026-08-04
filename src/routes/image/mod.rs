mod badnews;
mod beautiful;
mod blur;
mod circle;
mod color;
mod communist;
mod deepfry;
mod delete;
mod discordjs;
mod facts;
mod grayscale;
mod invert;
mod mad;
mod mirror;
mod pixel;
mod rainbow;
mod rankcard;
mod santa;
mod ship;
mod simp;
mod sonic;
mod supreme;
mod sus;
mod thisis;
mod titan;
mod twoways;
mod walletcard;
mod welcomecard;
mod whoreallyare;

use crate::state::AppState;
use axum::{routing::get, Router};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/image/grayscale", get(grayscale::handler))
        .route("/image/invert", get(invert::handler))
        .route("/image/mirror", get(mirror::handler))
        .route("/image/blur", get(blur::handler))
        .route("/image/deepfry", get(deepfry::handler))
        .route("/image/pixel", get(pixel::handler))
        .route("/image/circle", get(circle::handler))
        .route("/image/color", get(color::handler))
        .route("/image/badnews", get(badnews::handler))
        .route("/image/discordjs", get(discordjs::handler))
        .route("/image/supreme", get(supreme::handler))
        .route("/image/santa", get(santa::handler))
        .route("/image/facts", get(facts::handler))
        .route("/image/sonic", get(sonic::handler))
        .route("/image/titan", get(titan::handler))
        .route("/image/twoways", get(twoways::handler))
        .route("/image/thisis", get(thisis::handler))
        .route("/image/beautiful", get(beautiful::handler))
        .route("/image/communist", get(communist::handler))
        .route("/image/rainbow", get(rainbow::handler))
        .route("/image/simp", get(simp::handler))
        .route("/image/sus", get(sus::handler))
        .route("/image/mad", get(mad::handler))
        .route("/image/delete", get(delete::handler))
        .route("/image/whoreallyare", get(whoreallyare::handler))
        .route("/image/ship", get(ship::handler))
        .route("/image/rankcard", get(rankcard::handler))
        .route("/image/walletcard", get(walletcard::handler))
        .route("/image/welcomecard", get(welcomecard::handler))
}
