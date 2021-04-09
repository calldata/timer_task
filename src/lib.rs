use std::num;
use std::str::FromStr;
use std::{fmt, usize};

use chrono::{DateTime, Datelike, Duration, TimeZone, Timelike, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TimerConfig {
    sec: String,
    min: String,
    hour: String,
    day_of_month: String,
    month: String,
    week_of_month: String,
    day_of_week: String,
    day_of_year: String,
    week_of_year: String,
    year: String,
}

impl FromStr for TimerConfig {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s).map_err(|_| ParseError::InvalidValue)
    }
}

impl TimerConfig {
    pub fn parse<TZ: TimeZone>(&self, dt: &DateTime<TZ>) -> Result<DateTime<TZ>, ParseError> {
        let tz = dt.timezone();
        let mut next = Utc.from_local_datetime(&dt.naive_local()).unwrap() + Duration::seconds(1);

        // month
        let month = parse_field(&self.month, 1, 12)?;
        // day of month
        let do_m = parse_field(&self.day_of_month, 1, 31)?;
        // day of week
        let do_w = parse_field(&self.day_of_week, 0, 6)?;
        // day of year
        let do_y = parse_field(&self.day_of_year, 1, 366)?;
        // week of month
        let wo_m = parse_field(&self.week_of_month, 1, 4)?;
        // week of year
        let wo_y = parse_field(&self.week_of_year, 1, 53)?;
        // hour
        let hour = parse_field(&self.hour, 0, 23)?;
        // minute
        let minute = parse_field(&self.min, 0, 59)?;
        // second
        let seconds = parse_field(&self.sec, 0, 59)?;

        let result = loop {
            // 最多10年
            if next.year() - dt.year() > 10 {
                return Err(ParseError::InvalidValue);
            }

            if !month.get(next.month() as usize) {
                if next.month() == 12 {
                    next = Utc.ymd(next.year() + 1, 1, 1).and_hms(0, 0, 0);
                } else {
                    next = Utc.ymd(next.year(), next.month() + 1, 1).and_hms(0, 0, 0);
                }
                continue;
            }

            if !do_m.get(next.day() as usize) {
                next = next + Duration::days(1);
                next = Utc
                    .ymd(next.year(), next.month(), next.day())
                    .and_hms(0, 0, 0);
                continue;
            }

            if !do_w.get(next.weekday().num_days_from_sunday() as usize) {
                next = next + Duration::days(1);
                next = Utc
                    .ymd(next.year(), next.month(), next.day())
                    .and_hms(0, 0, 0);
                continue;
            }

            // dbg!(&do_y);
            if !do_y.get(next.day() as usize) {
                next = next + Duration::days(1);
                next = Utc
                    .ymd(next.year(), next.month(), next.day())
                    .and_hms(0, 0, 0);
                continue;
            }

            if !wo_m.get(next.day() as usize / 7) {
                next = next + Duration::days(7);
                next = Utc
                    .ymd(next.year(), next.month(), next.day())
                    .and_hms(0, 0, 0);
                continue;
            }

            if !wo_y.get(next.iso_week().week() as usize) {
                next = next + Duration::days(7 - next.weekday() as i64);
                next = Utc
                    .ymd(next.year(), next.month(), next.day())
                    .and_hms(0, 0, 0);
                continue;
            }

            if !hour.get(next.hour() as usize) {
                next = next + Duration::hours(1);
                next = Utc
                    .ymd(next.year(), next.month(), next.day())
                    .and_hms(next.hour(), 0, 0);
                continue;
            }

            if !minute.get(next.minute() as usize) {
                next = next + Duration::minutes(1);
                next = Utc.ymd(next.year(), next.month(), next.day()).and_hms(
                    next.hour(),
                    next.minute(),
                    0,
                );
                continue;
            }

            if !seconds.get(next.second() as usize) {
                next = next + Duration::seconds(1);
                continue;
            }

            if let Some(dt) = tz.from_local_datetime(&next.naive_local()).latest() {
                break dt;
            }

            next = next + Duration::seconds(1);
        };

        Ok(result)
    }

    /// TOOD: 农历
    pub fn parse_lunar_date<TZ: TimeZone>(&self) -> Result<DateTime<TZ>, ParseError> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum ParseError {
    InvalidValue,
    InvalidRange,
    ParseIntError(num::ParseIntError),
    TryFromIntError(num::TryFromIntError),
}

enum Dow {
    Sun = 0,
    Mon = 1,
    Tue = 2,
    Wed = 3,
    Thu = 4,
    Fri = 5,
    Sat = 6,
}

impl FromStr for Dow {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "SUN" => Ok(Self::Sun),
            "MON" => Ok(Self::Mon),
            "TUE" => Ok(Self::Tue),
            "WED" => Ok(Self::Wed),
            "THU" => Ok(Self::Thu),
            "FRI" => Ok(Self::Fri),
            "SAT" => Ok(Self::Sat),
            _ => Err(()),
        }
    }
}

impl std::error::Error for ParseError {}

impl From<num::ParseIntError> for ParseError {
    fn from(err: num::ParseIntError) -> Self {
        Self::ParseIntError(err)
    }
}

impl From<num::TryFromIntError> for ParseError {
    fn from(err: num::TryFromIntError) -> Self {
        Self::TryFromIntError(err)
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ParseError::*;
        match self {
            InvalidRange => write!(f, "invalid range"),
            InvalidValue => write!(f, "invalid value"),
            ParseIntError(err) => err.fmt(f),
            TryFromIntError(err) => err.fmt(f),
        }
    }
}

/// 解析每一个配置项
pub fn parse_field(
    field: &str,
    min: usize,
    max: usize,
) -> Result<bitmaps::Bitmap<typenum::U367>, ParseError> {
    let mut values = bitmaps::Bitmap::<typenum::U367>::new();

    let fields: Vec<&str> = field.split(',').filter(|s| !s.is_empty()).collect();

    for field in fields {
        match field {
            // 匹配任意值
            "*" => {
                for i in min..=max {
                    values.set(i, true);
                }
            }
            // 步进值
            f if field.contains('/') => {
                let tmp: Vec<&str> = f.split('/').collect();
                if tmp.len() != 2 {
                    return Err(ParseError::InvalidValue);
                }

                let start = tmp[0].parse::<usize>()?;
                if start < min {
                    return Err(ParseError::InvalidValue);
                }
                let step = tmp[1].parse::<usize>()?;

                for i in (start..=max).step_by(step) {
                    values.set(i, true);
                }
            }
            //范围值
            f if f.contains('-') => {
                let tmp_fields: Vec<&str> = f.split('-').collect();
                if tmp_fields.len() != 2 {
                    return Err(ParseError::InvalidRange);
                }

                let mut fields: Vec<usize> = Vec::new();

                if let Ok(dow) = Dow::from_str(tmp_fields[0]) {
                    fields.push(dow as usize);
                } else {
                    fields.push(tmp_fields[0].parse::<usize>()?);
                };

                if let Ok(dow) = Dow::from_str(tmp_fields[1]) {
                    fields.push(dow as usize);
                } else {
                    fields.push(tmp_fields[1].parse::<usize>()?);
                }

                if fields[0] > fields[1] || fields[1] > max {
                    return Err(ParseError::InvalidRange);
                }
                for i in (fields[0]..=fields[1]).collect::<Vec<usize>>() {
                    values.set(i, true);
                }
            }
            // 字符串形式的星期表示
            _ => {
                if let Ok(dow) = Dow::from_str(field) {
                    values.set(dow as usize, true);
                } else {
                    let f = field.parse::<usize>()?;
                    if f > max {
                        return Err(ParseError::InvalidValue);
                    }
                    values.set(f as usize, true);
                }
            }
        }
    }
    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;

    #[test]
    fn test_value_list() {
        let res = parse_field("1,2,3", 1, 31).unwrap();
        assert_eq!(vec![1, 2, 3], res.into_iter().collect::<Vec<_>>());
    }

    #[test]
    fn test_range() {
        let res = parse_field("3-8", 1, 31).unwrap();
        assert_eq!(vec![3, 4, 5, 6, 7, 8], res.into_iter().collect::<Vec<_>>());
    }

    #[test]
    fn test_step() {
        let res = parse_field("2/2", 1, 10).unwrap();
        assert_eq!(vec![2, 4, 6, 8, 10], res.into_iter().collect::<Vec<_>>());
    }

    #[test]
    fn test_week() {
        let res = parse_field("SAT,TUE,WED", 0, 6).unwrap();
        assert_eq!(vec![2, 3, 6], res.into_iter().collect::<Vec<_>>());
    }

    #[test]
    fn test_any() {
        let res = parse_field("*", 1, 5).unwrap();
        assert_eq!(vec![1, 2, 3, 4, 5], res.into_iter().collect::<Vec<_>>());
    }

    #[test]
    fn test_combined() {
        let res = parse_field("1,2,3,7-10,18/3", 1, 31).unwrap();
        assert_eq!(
            vec![1, 2, 3, 7, 8, 9, 10, 18, 21, 24, 27, 30],
            res.into_iter().collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_schedul() {
        // 每周星期二八点
        let config = r#"
            {
                "sec": "0",
                "min": "0",
                "hour": "8",
                "day_of_month": "*",
                "month": "*",
                "week_of_month": "*",
                "day_of_week": "2",
                "day_of_year": "*",
                "week_of_year": "*",
                "year": "*"
            }
        "#;

        let c: TimerConfig = serde_json::from_str(config).unwrap();
        let dt = Utc::now().with_timezone(&Local);
        match c.parse(&dt) {
            Ok(res) => println!(
                "res = {:?}, res mills = {:?}",
                res,
                res.signed_duration_since(dt).num_milliseconds()
            ),
            Err(e) => println!("no such date == {:?}", e.to_string()),
        }
    }
}
