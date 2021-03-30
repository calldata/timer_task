use std::fmt;

use nom::{Err, error::{Error, ErrorKind}, IResult};
use nom::branch::alt;
use nom::branch::permutation;
use nom::bytes::complete::tag;

use chrono::Local;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct TimerConfig {
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

pub enum ParseError {
    SecOutOfBound,
    MinOutOfBound,
    HourOutOfBound,
    MonthOutOfBound,
    WeekOfMonthOutOfBound,
    DayOfMonthOutOfBound,
    DayOfYearOutOfBound,
    DayOfWeekOutOfBound,
    WeekOfYearOutOfBound,

    ConfigSyntaxError,
}

#[derive(Debug)]
struct TimeRecord {
    sec: [u8; 60],
    min: [u8; 60],
    hour: [u8; 24],
    day_of_month: [u8; 31],
    month: [u8; 13],
    week_of_month: [u8; 4],
    day_of_week: [u8; 7],
    day_of_year: [u8; 366 + 31],
    week_of_year: [u8; 57],
    year: u32,
}

impl Default for TimeRecord {
    fn default() -> Self {
        Self {
            sec: [0; 60],
            min: [0; 60],
            hour: [0; 24],
            day_of_month: [0; 31],
            month: [0; 13],
            week_of_month: [0; 4],
            day_of_week: [0; 7],
            day_of_year: [0; 397],
            week_of_year: [0; 57],
            // TODO
            year: 0
        }
    }
}

struct TimeRecordBuilder {
    
}

impl TimeRecord {
    /// 通过解析配置构造TimeRecord
    fn from_config(config: TimerConfig) -> TimeRecord {
        todo!()
    }

    fn from_fake_config() -> TimeRecord {
        /// fake
        Self {
            sec: [0; 60],
            min: [1; 60],
            hour: [1; 24],
            day_of_month: [1; 31],
            month: [1; 13],
            week_of_month: [1; 4],
            day_of_year: [1; 397],
            day_of_week: [1; 7],
            week_of_year: [1; 57],
            year: 2021
        }
    }

    /// 未来触发的时间点
    fn upcomming(&mut self) {
    }

    /// 下一次触发时间点
    fn next(&mut self) {

    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ParseError::*;
        match self {
            SecOutOfBound => write!(f, "sec out of bound"),
            MinOutOfBound => write!(f, "min out of bound"),
            HourOutOfBound => write!(f, "hour out of bound"),
            MonthOutOfBound => write!(f, "month out of bound"),
            WeekOfMonthOutOfBound => write!(f, "week of month out of bound"),
            DayOfMonthOutOfBound => write!(f, "day of month out of bound"),
            DayOfYearOutOfBound => write!(f, "day of year out of bound"),
            DayOfWeekOutOfBound => write!(f, "day of week out of bound"),
            WeekOfYearOutOfBound => write!(f, "week of year out of bound"),
            ConfigSyntaxError => write!(f, "config syntax error"),
        }
    }
}

fn parse_sec(i: &str) -> Result<Vec<bool>, ParseError> {
    todo!()
}

fn parse_min(i: &str) -> Result<Vec<bool>, ParseError> {
    todo!()
}

fn parse_hour(i: &str) -> Result<Vec<bool>, ParseError> {
    todo!()
}

fn parse_month(i: &str) -> Result<Vec<bool>, ParseError> {
    todo!()
}

fn parse_week_of_month(i: &str) -> Result<Vec<bool>, ParseError> {
    todo!()
}

fn parse_day_of_month(i: &str) -> Result<Vec<bool>, ParseError> {
    todo!()
}

fn parse_day_of_year(i: &str) -> Result<Vec<bool>, ParseError> {
    todo!()
}

fn parse_day_of_week(i: &str) -> Result<Vec<bool>, ParseError> {
    todo!()
}

fn parse_week_of_year(i: &str) -> Result<Vec<bool>, ParseError> {
    todo!()
}

fn parse_year(i: &str) -> Result<Vec<bool>, ParseError> {
    todo!()
}

#[test]
fn test_next_time() {
    let local = Local::now();
    dbg!(local);
}
