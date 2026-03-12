use anyhow::{anyhow, Result};
use chrono::{Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PredicateOperator {
    GreaterThan,
    LessThan,
    Equal,
}

impl PredicateOperator {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            PredicateOperator::GreaterThan => ">",
            PredicateOperator::LessThan => "<",
            PredicateOperator::Equal => "=",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct ParsedSizePredicate {
    pub operator: PredicateOperator,
    pub raw_value: String,
    pub numeric_value: f64,
    pub unit: String,
    pub target_size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum ParsedTimeValue {
    Relative {
        amount: u64,
        unit: String,
        seconds: u64,
    },
    Absolute {
        raw: String,
        granularity: String,
        unix_millis: i64,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct ParsedModifiedPredicate {
    pub operator: PredicateOperator,
    pub raw_value: String,
    pub value: ParsedTimeValue,
}

pub(crate) fn parse_and_compare_size(file_size: u64, query: &str) -> Result<bool> {
    let parsed = parse_size_predicate(query)?;

    match parsed.operator {
        PredicateOperator::GreaterThan => Ok(file_size > parsed.target_size_bytes),
        PredicateOperator::LessThan => Ok(file_size < parsed.target_size_bytes),
        PredicateOperator::Equal => Ok(file_size == parsed.target_size_bytes),
    }
}

pub(crate) fn parse_and_compare_time(modified_time: SystemTime, query: &str) -> Result<bool> {
    let parsed = parse_modified_predicate(query)?;
    let threshold_time = threshold_system_time(&parsed.value)?;

    match parsed.operator {
        PredicateOperator::GreaterThan => Ok(modified_time > threshold_time),
        PredicateOperator::LessThan => Ok(modified_time < threshold_time),
        PredicateOperator::Equal => {
            if matches!(
                parsed.value,
                ParsedTimeValue::Absolute {
                    ref granularity, ..
                } if granularity == "date"
            ) {
                let modified_local = chrono::DateTime::<Local>::from(modified_time);
                let threshold_local = chrono::DateTime::<Local>::from(threshold_time);
                Ok(modified_local.date_naive() == threshold_local.date_naive())
            } else {
                Ok(modified_time == threshold_time)
            }
        }
    }
}

pub(crate) fn parse_size_predicate(query: &str) -> Result<ParsedSizePredicate> {
    let query = query.trim();
    let (operator, size_str) = parse_operator_prefix(query)?;
    let size_str = size_str.trim().to_lowercase();
    let (num_str, unit) = size_str.split_at(
        size_str
            .find(|c: char| !c.is_ascii_digit() && c != '.')
            .unwrap_or(size_str.len()),
    );

    let numeric_value = num_str.parse::<f64>()?;
    let unit = unit.trim().to_string();
    let multiplier = match unit.as_str() {
        "b" | "" => 1.0,
        "kb" | "k" => 1024.0,
        "mb" | "m" => 1024.0 * 1024.0,
        "gb" | "g" => 1024.0 * 1024.0 * 1024.0,
        _ => return Err(anyhow!("Invalid size unit: {unit}")),
    };

    Ok(ParsedSizePredicate {
        operator,
        raw_value: query.to_string(),
        numeric_value,
        unit,
        target_size_bytes: (numeric_value * multiplier) as u64,
    })
}

pub(crate) fn parse_modified_predicate(query: &str) -> Result<ParsedModifiedPredicate> {
    let query = query.trim();
    let (operator, time_str) = parse_operator_prefix(query)?;
    let time_str = time_str.trim();

    let value = if let Ok((duration, amount, unit)) = parse_relative_time(time_str) {
        ParsedTimeValue::Relative {
            amount,
            unit,
            seconds: duration.as_secs(),
        }
    } else if let Ok((datetime, granularity)) = parse_absolute_time(time_str) {
        ParsedTimeValue::Absolute {
            raw: time_str.to_string(),
            granularity,
            unix_millis: datetime
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_millis() as i64)
                .map_err(|_| anyhow!("Absolute date must not be before the unix epoch"))?,
        }
    } else {
        return Err(anyhow!("Invalid date format: '{time_str}'"));
    };

    Ok(ParsedModifiedPredicate {
        operator,
        raw_value: query.to_string(),
        value,
    })
}

fn parse_operator_prefix(query: &str) -> Result<(PredicateOperator, &str)> {
    let (op, remainder) = if query.starts_with(['>', '<', '=']) {
        query.split_at(1)
    } else {
        ("=", query)
    };

    let operator = match op {
        ">" => PredicateOperator::GreaterThan,
        "<" => PredicateOperator::LessThan,
        "=" => PredicateOperator::Equal,
        _ => return Err(anyhow!("Invalid operator: {op}")),
    };

    Ok((operator, remainder))
}

fn parse_relative_time(time_str: &str) -> Result<(Duration, u64, String)> {
    let (num_str, unit) = time_str.split_at(
        time_str
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(time_str.len()),
    );
    let amount = num_str.parse::<u64>()?;
    let unit = unit.trim().to_string();
    let multiplier = match unit.as_str() {
        "s" => 1,
        "m" => 60,
        "h" => 3600,
        "d" => 86400,
        "w" => 86400 * 7,
        "y" => 86400 * 365,
        _ => return Err(anyhow!("Invalid time unit")),
    };
    Ok((Duration::from_secs(amount * multiplier), amount, unit))
}

fn parse_absolute_time(time_str: &str) -> Result<(SystemTime, String)> {
    let (datetime, granularity) =
        if let Ok(dt) = NaiveDateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M:%S") {
            (dt, "datetime".to_string())
        } else if let Ok(date) = NaiveDate::parse_from_str(time_str, "%Y-%m-%d") {
            (
                date.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap()),
                "date".to_string(),
            )
        } else {
            return Err(anyhow!("Invalid absolute date format"));
        };

    Ok((
        Local
            .from_local_datetime(&datetime)
            .single()
            .ok_or_else(|| anyhow!("Failed to convert to local time"))?
            .into(),
        granularity,
    ))
}

fn threshold_system_time(value: &ParsedTimeValue) -> Result<SystemTime> {
    Ok(match value {
        ParsedTimeValue::Relative { seconds, .. } => SystemTime::now()
            .checked_sub(Duration::from_secs(*seconds))
            .ok_or_else(|| anyhow!("Time calculation underflow"))?,
        ParsedTimeValue::Absolute { unix_millis, .. } => {
            UNIX_EPOCH + Duration::from_millis(*unix_millis as u64)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_compare_size_invalid_unit() {
        let result = parse_and_compare_size(1000, "100xyz");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid size unit"));
    }

    #[test]
    fn test_parse_and_compare_size_operators() {
        assert!(parse_and_compare_size(1000, ">500").unwrap());
        assert!(parse_and_compare_size(1000, "<2000").unwrap());
        assert!(parse_and_compare_size(1000, "=1000").unwrap());
        assert!(!parse_and_compare_size(1000, ">1000").unwrap());
        assert!(!parse_and_compare_size(1000, "<1000").unwrap());
    }

    #[test]
    fn test_parse_and_compare_time_exact_match() {
        let now = SystemTime::now();
        let result = parse_and_compare_time(now, "<1d");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_and_compare_time_comparison_operators() {
        let now = SystemTime::now();
        assert!(parse_and_compare_time(now, "<1d").is_ok());
        assert!(parse_and_compare_time(now, ">0s").is_ok());
    }

    #[test]
    fn test_parse_relative_time_all_units() {
        let now = SystemTime::now();
        assert!(parse_and_compare_time(now, "<10s").is_ok());
        assert!(parse_and_compare_time(now, "<10m").is_ok());
        assert!(parse_and_compare_time(now, "<10h").is_ok());
        assert!(parse_and_compare_time(now, "<10d").is_ok());
        assert!(parse_and_compare_time(now, "<10w").is_ok());
        assert!(parse_and_compare_time(now, "<1y").is_ok());
    }

    #[test]
    fn test_parse_and_compare_size_all_units() {
        assert!(parse_and_compare_size(1000, "=1000b").unwrap());
        assert!(parse_and_compare_size(1024, "=1kb").unwrap());
        assert!(parse_and_compare_size(1024, "=1k").unwrap());
        assert!(parse_and_compare_size(1024 * 1024, "=1mb").unwrap());
        assert!(parse_and_compare_size(1024 * 1024, "=1m").unwrap());
        assert!(parse_and_compare_size(1024 * 1024 * 1024, "=1gb").unwrap());
        assert!(parse_and_compare_size(1024 * 1024 * 1024, "=1g").unwrap());
    }

    #[test]
    fn test_parse_and_compare_time_with_exact_datetime() {
        let now = SystemTime::now();
        let result = parse_and_compare_time(now, "=2024-01-01 12:00:00");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_and_compare_time_date_only_equality() {
        let now = SystemTime::now();
        let result = parse_and_compare_time(now, "=2024-01-01");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_size_predicate_retains_units() {
        let parsed = parse_size_predicate(">1.5mb").unwrap();
        assert_eq!(parsed.operator.as_str(), ">");
        assert_eq!(parsed.numeric_value, 1.5);
        assert_eq!(parsed.unit, "mb");
        assert_eq!(parsed.target_size_bytes, 1_572_864);
    }

    #[test]
    fn test_parse_modified_predicate_reports_relative_shape() {
        let parsed = parse_modified_predicate("<7d").unwrap();
        assert_eq!(parsed.operator.as_str(), "<");
        match parsed.value {
            ParsedTimeValue::Relative { amount, unit, .. } => {
                assert_eq!(amount, 7);
                assert_eq!(unit, "d");
            }
            other => panic!("expected relative time, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_modified_predicate_reports_absolute_shape() {
        let parsed = parse_modified_predicate("=2024-01-01").unwrap();
        match parsed.value {
            ParsedTimeValue::Absolute { granularity, .. } => {
                assert_eq!(granularity, "date");
            }
            other => panic!("expected absolute time, got {other:?}"),
        }
    }
}
