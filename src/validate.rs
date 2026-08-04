use crate::response::ApiError;

/// Ensures a string's character count sits within [min, max], mirroring FastAPI's
/// `Query(min_length=..., max_length=...)` validation.
pub fn len(value: &str, min: usize, max: usize, field: &str) -> Result<(), ApiError> {
    let n = value.chars().count();
    if n < min || n > max {
        return Err(ApiError::validation(format!("{field} must be between {min} and {max} characters"), field));
    }
    Ok(())
}

/// Ensures a numeric value sits within [min, max].
pub fn range<T: PartialOrd + std::fmt::Display>(value: T, min: T, max: T, field: &str) -> Result<(), ApiError> {
    if value < min || value > max {
        return Err(ApiError::validation(format!("{field} must be between {min} and {max}"), field));
    }
    Ok(())
}

/// Validates a 3 or 6 digit hex color code.
pub fn hex_color(value: &str, field: &str) -> Result<(), ApiError> {
    let c = value.trim_start_matches('#');
    let ok = (c.len() == 3 || c.len() == 6) && c.chars().all(|ch| ch.is_ascii_hexdigit());
    if !ok {
        return Err(ApiError::validation("Invalid hex color code", field));
    }
    Ok(())
}
