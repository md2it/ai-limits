use chrono::{DateTime, Datelike, Local};

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

pub fn format_local_datetime(dt: DateTime<Local>, reference: DateTime<Local>) -> String {
    if dt.date_naive() == reference.date_naive() {
        return format!("today {}", dt.format("%H:%M"));
    }

    let month = MONTHS.get(dt.month0() as usize).copied().unwrap_or("???");
    format!("{} {}, {}", month, dt.day(), dt.format("%H:%M"))
}

pub fn format_local_date(dt: DateTime<Local>) -> String {
    let month = MONTHS.get(dt.month0() as usize).copied().unwrap_or("???");
    format!("{} {}, {}", month, dt.day(), dt.year())
}

pub(super) fn strip_display_timezone_suffix(value: &str) -> String {
    let trimmed = value.trim();
    let without_named_timezone = if let Some(open) = trimmed.rfind('(') {
        if trimmed.ends_with(')') {
            trimmed[..open].trim()
        } else {
            trimmed
        }
    } else {
        trimmed
    };

    strip_utc_suffix(without_named_timezone).to_string()
}

fn strip_utc_suffix(value: &str) -> &str {
    let Some((body, suffix)) = value.rsplit_once(' ') else {
        return value;
    };
    if is_utc_suffix(suffix) {
        body.trim_end()
    } else {
        value
    }
}

fn is_utc_suffix(value: &str) -> bool {
    if value.eq_ignore_ascii_case("UTC") {
        return true;
    }

    let Some(offset) = value
        .strip_prefix("UTC+")
        .or_else(|| value.strip_prefix("UTC-"))
        .or_else(|| value.strip_prefix("utc+"))
        .or_else(|| value.strip_prefix("utc-"))
    else {
        return false;
    };

    let (hours, minutes) = offset.split_once(':').unwrap_or((offset, ""));
    !hours.is_empty()
        && hours.len() <= 2
        && hours.chars().all(|character| character.is_ascii_digit())
        && (minutes.is_empty()
            || (minutes.len() == 2 && minutes.chars().all(|character| character.is_ascii_digit())))
}
