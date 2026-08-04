use crate::response::{ApiError, ApiOk, ApiResult};
use crate::extract::Query;
use chrono::{Datelike, Local, NaiveDate};
use serde::Deserialize;

const MONTH_NAMES: [&str; 12] = [
    "January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December",
];
const WEEKDAY_NAMES: [&str; 7] = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];

#[derive(Deserialize)]
pub struct CalendarQuery {
    year: Option<i32>,
    month: Option<u32>,
}

fn days_in_month(year: i32, month: u32) -> Option<u32> {
    let first = NaiveDate::from_ymd_opt(year, month, 1)?;
    let next = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)?
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)?
    };
    Some((next - first).num_days() as u32)
}

/// Renders a plain-text month calendar, Monday-first (mirrors Python's `calendar.month()`).
fn format_month(year: i32, month: u32) -> String {
    let first = NaiveDate::from_ymd_opt(year, month, 1).expect("validated above");
    let first_weekday = first.weekday().num_days_from_monday();
    let ndays = days_in_month(year, month).unwrap_or(28);

    const WIDTH: usize = 20; // 7 columns * 3 chars - 1
    let header = format!("{} {}", MONTH_NAMES[(month - 1) as usize], year);

    let mut lines = vec![format!("{header:^WIDTH$}"), WEEKDAY_NAMES.join(" ")];

    let mut row: Vec<String> = (0..first_weekday).map(|_| "  ".to_string()).collect();
    for day in 1..=ndays {
        row.push(format!("{day:>2}"));
        if row.len() == 7 {
            lines.push(row.join(" "));
            row.clear();
        }
    }
    if !row.is_empty() {
        lines.push(row.join(" "));
    }

    lines.join("\n")
}

/// GET /json/calendar?year=&month= — defaults to the current year/month.
pub async fn handler(Query(q): Query<CalendarQuery>) -> ApiResult {
    let year = q.year.unwrap_or_else(|| Local::now().year());
    let month = q.month.unwrap_or_else(|| Local::now().month());

    if !(1000..=2100).contains(&year) {
        return Err(ApiError::validation("year must be between 1000 and 2100", "year"));
    }
    if !(1..=12).contains(&month) {
        return Err(ApiError::validation("month must be between 1 and 12", "month"));
    }

    Ok(ApiOk::new(format_month(year, month)))
}
