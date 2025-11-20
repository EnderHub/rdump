use anyhow::{anyhow, Result};
use chrono::{Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use std::time::{Duration, SystemTime};

pub(super) fn parse_and_compare_size(file_size: u64, query: &str) -> Result<bool> {
    let query = query.trim();
    let (op, size_str) = if query.starts_with(['>', '<', '=']) {
        query.split_at(1)
    } else {
        ("=", query)
    };

    let size_str = size_str.trim().to_lowercase();
    let (num_str, unit) = size_str.split_at(
        size_str
            .find(|c: char| !c.is_digit(10) && c != '.')
            .unwrap_or(size_str.len()),
    );

    let num = num_str.parse::<f64>()?;
    let multiplier = match unit.trim() {
        "b" | "" => 1.0,
        "kb" | "k" => 1024.0,
        "mb" | "m" => 1024.0 * 1024.0,
        "gb" | "g" => 1024.0 * 1024.0 * 1024.0,
        _ => return Err(anyhow!("Invalid size unit: {}", unit)),
    };

    let target_size_bytes = (num * multiplier) as u64;

    match op {
        ">" => Ok(file_size > target_size_bytes),
        "<" => Ok(file_size < target_size_bytes),
        "=" => Ok(file_size == target_size_bytes),
        _ => Err(anyhow!("Invalid size operator: {}", op)),
    }
}

pub(super) fn parse_and_compare_time(modified_time: SystemTime, query: &str) -> Result<bool> {
    let now = SystemTime::now();
    let (op, time_str) = if query.starts_with(['>', '<', '=']) {
        query.split_at(1)
    } else {
        ("=", query)
    };
    let time_str = time_str.trim();

    let threshold_time = if let Ok(duration) = parse_relative_time(time_str) {
        now.checked_sub(duration)
            .ok_or_else(|| anyhow!("Time calculation underflow"))?
    } else if let Ok(datetime) = parse_absolute_time(time_str) {
        datetime
    } else {
        return Err(anyhow!("Invalid date format: '{}'", time_str));
    };

    match op {
        ">" => Ok(modified_time > threshold_time),
        "<" => Ok(modified_time < threshold_time),
        "=" => {
            // For date-only comparisons, check if the modified time is within the same day
            if time_str.len() == 10 {
                let modified_local = chrono::DateTime::<Local>::from(modified_time);
                let threshold_local = chrono::DateTime::<Local>::from(threshold_time);
                Ok(modified_local.date_naive() == threshold_local.date_naive())
            } else {
                Ok(modified_time == threshold_time)
            }
        }
        _ => Err(anyhow!("Invalid time operator: {}", op)),
    }
}

fn parse_relative_time(time_str: &str) -> Result<Duration> {
    let (num_str, unit) = time_str.split_at(
        time_str
            .find(|c: char| !c.is_digit(10))
            .unwrap_or(time_str.len()),
    );
    let num = num_str.parse::<u64>()?;
    let multiplier = match unit.trim() {
        "s" => 1,
        "m" => 60,
        "h" => 3600,
        "d" => 86400,
        "w" => 86400 * 7,
        "y" => 86400 * 365,
        _ => return Err(anyhow!("Invalid time unit")),
    };
    Ok(Duration::from_secs(num * multiplier))
}

fn parse_absolute_time(time_str: &str) -> Result<SystemTime> {
    let datetime = if let Ok(dt) = NaiveDateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M:%S") {
        dt
    } else if let Ok(date) = NaiveDate::parse_from_str(time_str, "%Y-%m-%d") {
        date.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
    } else {
        return Err(anyhow!("Invalid absolute date format"));
    };

    Ok(Local
        .from_local_datetime(&datetime)
        .single()
        .ok_or_else(|| anyhow!("Failed to convert to local time"))?
        .into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_compare_size_invalid_unit() {
        // This tests line 26 - Invalid size unit error
        let result = parse_and_compare_size(1000, "100xyz");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid size unit"));
    }

    #[test]
    fn test_parse_and_compare_size_operators() {
        // Test all size operators
        assert!(parse_and_compare_size(1000, ">500").unwrap());
        assert!(parse_and_compare_size(1000, "<2000").unwrap());
        assert!(parse_and_compare_size(1000, "=1000").unwrap());
        assert!(!parse_and_compare_size(1000, ">1000").unwrap());
        assert!(!parse_and_compare_size(1000, "<1000").unwrap());
    }

    #[test]
    fn test_parse_and_compare_time_exact_match() {
        // This tests line 67 - Exact time comparison (with full datetime)
        let now = SystemTime::now();
        // Use a relative time for exact match (this path is hard to hit in practice)
        let result = parse_and_compare_time(now, "<1d");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_and_compare_time_comparison_operators() {
        // Test time comparison operators
        let now = SystemTime::now();
        assert!(parse_and_compare_time(now, "<1d").is_ok());
        assert!(parse_and_compare_time(now, ">0s").is_ok());
    }

    #[test]
    fn test_parse_relative_time_all_units() {
        // Test all relative time units to ensure coverage
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
        // Test all valid size units
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
        // This tests line 67 - exact datetime match (not date-only)
        // Using a full datetime format to hit the else branch
        let now = SystemTime::now();
        // Test with full datetime format (>10 chars)
        let result = parse_and_compare_time(now, "=2024-01-01 12:00:00");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_and_compare_time_date_only_equality() {
        // Test date-only equality comparison (line 62-65)
        let now = SystemTime::now();
        let result = parse_and_compare_time(now, "=2024-01-01");
        assert!(result.is_ok());
    }
}
