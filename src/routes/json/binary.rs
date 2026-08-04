use crate::response::{ApiError, ApiOk, ApiResult};
use crate::extract::Query;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct BinaryQuery {
    body: String,
}

#[derive(Serialize)]
struct BinaryOut {
    original: String,
    converted: String,
}

fn text_to_bits(text: &str) -> String {
    text.bytes().map(|b| format!("{b:08b}")).collect()
}

fn text_from_bits(bits: &str) -> Option<String> {
    if bits.is_empty() || bits.len() % 8 != 0 || !bits.bytes().all(|b| b == b'0' || b == b'1') {
        return None;
    }
    let bytes: Vec<u8> = bits
        .as_bytes()
        .chunks(8)
        .map(|c| u8::from_str_radix(std::str::from_utf8(c).ok()?, 2).ok())
        .collect::<Option<Vec<u8>>>()?;
    String::from_utf8(bytes).ok()
}

/// GET /json/binary?body=... — encodes text to binary, or decodes binary back to text
/// when `body` is made up entirely of digits (mirrors the Python `str.isdigit()` check).
pub async fn handler(Query(q): Query<BinaryQuery>) -> ApiResult {
    if q.body.len() < 2 || q.body.len() > 1000 {
        return Err(ApiError::validation("body must be between 2 and 1000 characters", "body"));
    }

    let converted = if q.body.chars().all(|c| c.is_ascii_digit()) {
        text_from_bits(&q.body).ok_or_else(|| ApiError::bad_request("Your body was unable to be decoded"))?
    } else {
        text_to_bits(&q.body)
    };

    Ok(ApiOk::new(BinaryOut { original: q.body, converted }))
}
