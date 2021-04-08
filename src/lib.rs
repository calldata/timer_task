use std::{collections::BTreeSet, ops::Add};
use std::fmt;
use std::num;
use std::str::FromStr;

use chrono::{
    DateTime, Datelike, Duration, FixedOffset, Local, NaiveDate, NaiveDateTime, Offset, TimeZone,
    Timelike, Utc,
};
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

impl TimerConfig {
    pub fn parse<TZ: TimeZone>(&self, dt: &DateTime<TZ>) -> Result<DateTime<TZ>, ParseError> {
        let tz = dt.timezone();
        let mut next = Utc.from_local_datetime(&dt.naive_local()).unwrap() + Duration::minutes(1);

        next =
            Utc.ymd(next.year(), next.month(), next.day())
                .and_hms(next.hour(), next.minute(), 0);

        let result = loop {
            // 最多10年
            if next.year() - dt.year() > 10 {
                return Err(ParseError::InvalidValue);
            }

            // month
            let month = parse_field(&self.month, 1, 12)?;
            // dbg!(&month);
            if !month.contains(&next.month()) {
                if next.month() == 12 {
                    next = Utc.ymd(next.year() + 1, 1, 1).and_hms(0, 0, 0);
                } else {
                    next = Utc.ymd(next.year(), next.month() + 1, 1).and_hms(0, 0, 0);
                }
                continue;
            }

            // day of month
            let do_m = parse_field(&self.day_of_month, 1, 31)?;
            // dbg!(&do_m);
            if !do_m.contains(&next.day()) {
                next = next + Duration::days(1);
                next = Utc
                    .ymd(next.year(), next.month(), next.day())
                    .and_hms(0, 0, 0);
                continue;
            }

            // TODO: day of year
            let do_y = parse_field(&self.day_of_year, 1, 366)?;
            // dbg!(&do_y);
            // let du = next.signed_duration_since::<Local>(DateTime::from_utc(
            //     NaiveDateTime::from_timestamp(next.timestamp(), 0),
            //     FixedOffset::east(8 * 60 * 60),
            // ));
            // println!("ddddd {:?}", du.num_days());
            if !do_y.contains(&6) {
                next = Utc
                    .ymd(next.year(), next.month(), next.day() + 1)
                    .and_hms(0, 0, 0);
                continue;
            }
            // hour
            let hour = parse_field(&self.hour, 0, 23)?;
            // dbg!(&hour);
            if !hour.contains(&next.hour()) {
                next = next + Duration::hours(1);
                next = Utc
                    .ymd(next.year(), next.month(), next.day())
                    .and_hms(next.hour(), 0, 0);
                continue;
            }

            // minute
            let minute = parse_field(&self.min, 0, 59)?;
            // dbg!(&minute);
            if !minute.contains(&next.minute()) {
                next = next + Duration::minutes(1);
                next = Utc.ymd(next.year(), next.month(), next.day()).and_hms(
                    next.hour(),
                    next.minute(),
                    next.second(),
                );
                continue;
            }

            // second
            let seconds = parse_field(&self.sec, 0, 59)?;
            // dbg!(&seconds);
            if !seconds.contains(&next.second()) {
                next = next + Duration::seconds(1);
                continue;
            }

            // TODO: week of month

            // day of week
            let do_w = parse_field(&self.day_of_week, 0, 6)?;
            // dbg!(&do_w);
            if !do_w.contains(&next.weekday().num_days_from_sunday()) {
                next = next + Duration::days(1);
                next = Utc
                    .ymd(next.year(), next.month(), next.day())
                    .and_hms(0, 0, 0);
                continue;
            }

            if let Some(dt) = tz.from_local_datetime(&next.naive_local()).latest() {
                break dt;
            }

            next = next + Duration::minutes(1);
        };

        Ok(result)
    }

    /// 农历
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
        match &*s.to_uppercase() {
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
pub fn parse_field(field: &str, min: u32, max: u32) -> Result<BTreeSet<u32>, ParseError> {
    let mut values = BTreeSet::<u32>::new();

    let fields: Vec<&str> = field.split(',').filter(|s| !s.is_empty()).collect();

    for field in fields {
        match field {
            // 匹配任意值
            "*" => {
                for i in min..=max {
                    values.insert(i);
                }
            }
            // 步进值
            f if field.contains('/') => {
                let tmp: Vec<&str> = f.split('/').collect();
                if tmp.len() != 2 {
                    return Err(ParseError::InvalidValue);
                }

                let start = tmp[0].parse::<u32>()?;
                if start < min {
                    return Err(ParseError::InvalidValue);
                }
                let step = tmp[1].parse::<u32>()?;

                for i in (start..=max).step_by(step as usize) {
                    values.insert(i);
                }
            }
            //范围值
            f if f.contains('-') => {
                let tmp_fields: Vec<&str> = f.split('-').collect();
                if tmp_fields.len() != 2 {
                    return Err(ParseError::InvalidRange);
                }

                let mut fields: Vec<u32> = Vec::new();

                if let Ok(dow) = Dow::from_str(tmp_fields[0]) {
                    fields.push(dow as u32);
                } else {
                    fields.push(tmp_fields[0].parse::<u32>()?);
                };

                if let Ok(dow) = Dow::from_str(tmp_fields[1]) {
                    fields.push(dow as u32);
                } else {
                    fields.push(tmp_fields[1].parse::<u32>()?);
                }

                if fields[0] > fields[1] || fields[1] > max {
                    return Err(ParseError::InvalidRange);
                }
                for i in (fields[0]..=fields[1]).collect::<Vec<u32>>() {
                    values.insert(i);
                }
            }
            // TODO
            _ => {
                if let Ok(dow) = Dow::from_str(field) {
                    values.insert(dow as u32);
                } else {
                    let f = field.parse::<u32>()?;
                    if f > max {
                        return Err(ParseError::InvalidValue);
                    }
                    values.insert(f);
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
    fn test2() {
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
                "day_of_year": "222-223",
                "week_of_year": "*",
                "year": "*"
            }
        "#;
        // let config = r#"
        //     {
        //         "sec": "1/2",
        //         "min": "*",
        //         "hour": "*",
        //         "day_of_month": "*",
        //         "month": "*",
        //         "week_of_month": "*",
        //         "day_of_week": "*",
        //         "day_of_year": "*",
        //         "week_of_year": "*",
        //         "year": "*"
        //     }
        // "#;

        let c: TimerConfig = serde_json::from_str(config).unwrap();
        println!("config ==== {:?}", c);
        let dt = Utc::now().with_timezone(&Local);
        match c.parse(&dt) {
            Ok(res) => println!("res = {:?}", res),
            Err(e) => println!("no such date == {:?}", e.to_string())
        }
    }

    #[test]
    fn test_parse() {
        let x = "1/2";
        let res = parse_field(x, 1, 31).unwrap();
        println!("res = {:?}", res);
    }
}
