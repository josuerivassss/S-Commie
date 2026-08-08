mod badnews;
mod blur;
mod circle;
mod color;
mod communist;
mod deepfry;
mod grayscale;
mod invert;
mod mad;
mod mirror;
mod pixel;
mod rainbow;
mod rankcard;
mod ship;
mod sonic;
mod supreme;
mod sus;
mod thisis;
mod titan;
mod twoways;
mod welcomecard;
mod caught;
mod discordprofile;

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
        .route("/image/supreme", get(supreme::handler))
        .route("/image/sonic", get(sonic::handler))
        .route("/image/titan", get(titan::handler))
        .route("/image/twoways", get(twoways::handler))
        .route("/image/thisis", get(thisis::handler))
        .route("/image/communist", get(communist::handler))
        .route("/image/rainbow", get(rainbow::handler))
        .route("/image/sus", get(sus::handler))
        .route("/image/mad", get(mad::handler))
        .route("/image/caught", get(caught::handler))
        .route("/image/ship", get(ship::handler))
        .route("/image/rankcard", get(rankcard::handler))
        .route("/image/welcomecard", get(welcomecard::handler))
        .route("/image/discordprofile", get(discordprofile::handler))
}
