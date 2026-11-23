use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TimestampParseError {
    #[error("timestamp is empty")]
    Empty,
    #[error("timestamp is negative")]
    Negative,
    #[error("timestamp format is unsupported")]
    Unsupported,
    #[error("timestamp component is invalid")]
    InvalidComponent,
}

pub fn parse_timestamp_ms(value: &str) -> Result<i64, TimestampParseError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(TimestampParseError::Empty);
    }
    if trimmed.starts_with('-') {
        return Err(TimestampParseError::Negative);
    }

    if let Some(stripped) = trimmed.strip_suffix("ms") {
        return parse_non_negative_i64(stripped.trim());
    }
    if let Some(stripped) = trimmed.strip_suffix('s') {
        let seconds = parse_decimal_seconds(stripped.trim())?;
        return Ok(seconds);
    }
    if trimmed.contains(':') {
        return parse_colon_timestamp(trimmed);
    }
    if trimmed.contains('.') {
        return parse_decimal_seconds(trimmed);
    }
    parse_non_negative_i64(trimmed)
}

fn parse_non_negative_i64(value: &str) -> Result<i64, TimestampParseError> {
    let parsed = value
        .parse::<i64>()
        .map_err(|_| TimestampParseError::InvalidComponent)?;
    if parsed < 0 {
        return Err(TimestampParseError::Negative);
    }
    Ok(parsed)
}

fn parse_decimal_seconds(value: &str) -> Result<i64, TimestampParseError> {
    let parsed = value
        .parse::<f64>()
        .map_err(|_| TimestampParseError::InvalidComponent)?;
    if !parsed.is_finite() {
        return Err(TimestampParseError::InvalidComponent);
    }
    if parsed < 0.0 {
        return Err(TimestampParseError::Negative);
    }
    Ok((parsed * 1000.0).round() as i64)
}

fn parse_colon_timestamp(value: &str) -> Result<i64, TimestampParseError> {
    let parts: Vec<&str> = value.split(':').collect();
    if !(2..=3).contains(&parts.len()) {
        return Err(TimestampParseError::Unsupported);
    }

    let seconds_part = parts.last().ok_or(TimestampParseError::Unsupported)?;
    let seconds_ms = parse_decimal_seconds(seconds_part)?;
    let minutes = parts[parts.len() - 2]
        .parse::<i64>()
        .map_err(|_| TimestampParseError::InvalidComponent)?;
    let hours = if parts.len() == 3 {
        parts[0]
            .parse::<i64>()
            .map_err(|_| TimestampParseError::InvalidComponent)?
    } else {
        0
    };
    if minutes < 0 || hours < 0 {
        return Err(TimestampParseError::Negative);
    }
    Ok(hours * 3_600_000 + minutes * 60_000 + seconds_ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_milliseconds() {
        assert_eq!(parse_timestamp_ms("1234").unwrap(), 1234);
        assert_eq!(parse_timestamp_ms("1234ms").unwrap(), 1234);
    }

    #[test]
    fn parses_seconds() {
        assert_eq!(parse_timestamp_ms("1.5s").unwrap(), 1500);
        assert_eq!(parse_timestamp_ms("1.5").unwrap(), 1500);
    }

    #[test]
    fn parses_mm_ss_and_hh_mm_ss() {
        assert_eq!(parse_timestamp_ms("01:02").unwrap(), 62_000);
        assert_eq!(parse_timestamp_ms("01:02:03.5").unwrap(), 3_723_500);
    }

    #[test]
    fn rejects_invalid_and_negative_values() {
        assert_eq!(
            parse_timestamp_ms("").unwrap_err(),
            TimestampParseError::Empty
        );
        assert_eq!(
            parse_timestamp_ms("-1").unwrap_err(),
            TimestampParseError::Negative
        );
        assert_eq!(
            parse_timestamp_ms("nope").unwrap_err(),
            TimestampParseError::InvalidComponent
        );
    }
}
