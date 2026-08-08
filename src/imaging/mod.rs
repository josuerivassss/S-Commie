pub mod badges;
pub mod color;
pub mod effects;
pub mod io;
pub mod text;

pub use badges::{draw_status_badge, PresenceStatus};
pub use color::{darken, parse_hex};
pub use effects::{dominant_colors, draw_gradient, ellipse_mask, paste, rounded_mask, rounded_rect_image};
pub use io::{open_image, prepare_png};