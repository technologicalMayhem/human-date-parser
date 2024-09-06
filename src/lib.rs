use std::fmt::Display;

use ast::{
    build_ast_from, Ago, Date, DateTime, Duration as AstDuration, In, IsoDate, Quantifier,
    RelativeSpecifier, Time, TimeUnit,
};
use chrono::{
    Datelike, Days, Duration as ChronoDuration, Month, Months, NaiveDate, NaiveDateTime,
    NaiveTime, Weekday,
};
use thiserror::Error;

mod ast;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Could not match input to any known format")]
    InvalidFormat,
    #[error("One or more errors occured when processing input")]
    ProccessingErrors(Vec<ProcessingError>),
    #[error(
        "An internal library error occured. This should not happen. Please report it. Error: {0}"
    )]
    InternalError(#[from] InternalError),
}

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("Could not build time from {hour}:{minute}")]
    TimeHourMinute { hour: u32, minute: u32 },
    #[error("Could not build time from {hour}:{minute}:{second}")]
    TimeHourMinuteSecond { hour: u32, minute: u32, second: u32 },
    #[error("Failed to add {count} {unit} to the current time")]
    AddToNow { unit: String, count: u32 },
    #[error("Failed to subtract {count} {unit} from the current time")]
    SubtractFromNow { unit: String, count: u32 },
    #[error("Failed to subtract {count} {unit} from {date}")]
    SubtractFromDate {
        unit: String,
        count: u32,
        date: NaiveDateTime,
    },
    #[error("Failed to add {count} {unit} to {date}")]
    AddToDate {
        unit: String,
        count: u32,
        date: NaiveDateTime,
    },
    #[error("{year}-{month}-{day} is not a valid date")]
    InvalidDate { year: i32, month: u32, day: u32 },
    #[error("Failed to parse inner human time: {0}")]
    InnerHumanTimeParse(Box<ParseError>),
}

#[derive(Debug, Error)]
pub enum InternalError {
    #[error("Failed to build AST. This is a bug.")]
    FailedToBuildAst,
}

#[derive(Debug)]
pub enum ParseResult {
    DateTime(NaiveDateTime),
    Date(NaiveDate),
    Time(NaiveTime),
}

impl Display for ParseResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseResult::DateTime(datetime) => write!(f, "{}", datetime),
            ParseResult::Date(date) => write!(f, "{}", date),
            ParseResult::Time(time) => write!(f, "{}", time),
        }
    }
}

/// Converts a human expression of a date into a more usable one.
///
/// # Errors
///
/// This function will return an error if the string contains values than can not be parsed into a date.
///
/// # Examples
/// ```
/// use chrono::Local;
/// use human_date_parser::{from_human_time, ParseResult};
/// let now = Local::now().naive_local();
/// let date = from_human_time("Last Friday at 19:45", now).unwrap();
/// match date {
///     ParseResult::DateTime(date) => println!("{date}"),
///     _ => unreachable!()
/// }
/// ```
pub fn from_human_time(str: &str, now: NaiveDateTime) -> Result<ParseResult, ParseError> {
    let lowercase = str.to_lowercase();
    let parsed = build_ast_from(&lowercase)?;

    parse_human_time(parsed, now)
}

fn parse_human_time(parsed: ast::HumanTime, now: NaiveDateTime) -> Result<ParseResult, ParseError> {
    match parsed {
        ast::HumanTime::DateTime(date_time) => {
            parse_date_time(date_time, &now).map(|dt| ParseResult::DateTime(dt))
        }
        ast::HumanTime::Date(date) => parse_date(date, &now)
            .map(|date| ParseResult::Date(date))
            .map_err(|err| ParseError::ProccessingErrors(vec![err])),
        ast::HumanTime::Time(time) => parse_time(time)
            .map(|time| ParseResult::Time(time))
            .map_err(|err| ParseError::ProccessingErrors(vec![err])),
        ast::HumanTime::In(in_ast) => parse_in(in_ast, &now)
            .map(|time| ParseResult::DateTime(time))
            .map_err(|err| ParseError::ProccessingErrors(vec![err])),
        ast::HumanTime::Ago(ago) => parse_ago(ago, &now)
            .map(|time| ParseResult::DateTime(time))
            .map_err(|err| ParseError::ProccessingErrors(vec![err])),
        ast::HumanTime::Now => Ok(ParseResult::DateTime(now)),
    }
}

fn parse_date_time(date_time: DateTime, now: &NaiveDateTime) -> Result<NaiveDateTime, ParseError> {
    let date = parse_date(date_time.date, now);
    let time = parse_time(date_time.time);

    match (date, time) {
        (Ok(date), Ok(time)) => Ok(NaiveDateTime::new(date, time)),
        (Ok(_), Err(time_error)) => Err(ParseError::ProccessingErrors(vec![time_error])),
        (Err(date_error), Ok(_)) => Err(ParseError::ProccessingErrors(vec![date_error])),
        (Err(date_error), Err(time_error)) => {
            Err(ParseError::ProccessingErrors(vec![date_error, time_error]))
        }
    }
}

fn parse_date(date: Date, now: &NaiveDateTime) -> Result<NaiveDate, ProcessingError> {
    match date {
        Date::Today => Ok(now.date()),
        Date::Tomorrow => {
            now.date()
                .checked_add_days(Days::new(1))
                .ok_or(ProcessingError::AddToNow {
                    unit: String::from("days"),
                    count: 1,
                })
        }
        Date::Overmorrow => {
            now.date()
                .checked_add_days(Days::new(2))
                .ok_or(ProcessingError::AddToNow {
                    unit: String::from("days"),
                    count: 2,
                })
        }
        Date::Yesterday => {
            now.date()
                .checked_sub_days(Days::new(1))
                .ok_or(ProcessingError::SubtractFromNow {
                    unit: String::from("days"),
                    count: 1,
                })
        }
        Date::IsoDate(iso_date) => parse_iso_date(iso_date),
        Date::DayMonthYear(day, month, year) => parse_day_month_year(day, month, year as i32),
        Date::DayMonth(day, month) => parse_day_month_year(day, month, now.year()),
        Date::RelativeWeekWeekday(relative, weekday) => {
            find_weekday_relative_week(relative, weekday.into(), now.date())
        }
        Date::RelativeWeekday(relative, weekday) => {
            find_weekday_relative(relative, weekday.into(), now.date())
        }
        Date::RelativeTimeUnit(relative, time_unit) => {
            Ok(relative_date_time_unit(relative, time_unit, now.clone())?.date())
        }
        Date::UpcomingWeekday(weekday) => {
            find_weekday_relative(RelativeSpecifier::Next, weekday.into(), now.date())
        }
    }
}

fn parse_iso_date(iso_date: IsoDate) -> Result<NaiveDate, ProcessingError> {
    let (year, month, day) = (iso_date.year as i32, iso_date.month, iso_date.day);
    NaiveDate::from_ymd_opt(year, month, day).ok_or(ProcessingError::InvalidDate {
        year,
        month,
        day,
    })
}

fn parse_day_month_year(day: u32, month: Month, year: i32) -> Result<NaiveDate, ProcessingError> {
    let month = month.number_from_month();
    NaiveDate::from_ymd_opt(year, month, day).ok_or(ProcessingError::InvalidDate {
        year,
        month,
        day,
    })
}

fn parse_time(time: Time) -> Result<NaiveTime, ProcessingError> {
    match time {
        Time::HourMinute(hour, minute) => NaiveTime::from_hms_opt(hour, minute, 0)
            .ok_or(ProcessingError::TimeHourMinute { hour, minute }),
        Time::HourMinuteSecond(hour, minute, second) => NaiveTime::from_hms_opt(
            hour, minute, second,
        )
        .ok_or(ProcessingError::TimeHourMinuteSecond {
            hour,
            minute,
            second,
        }),
    }
}

fn parse_in(in_ast: In, now: &NaiveDateTime) -> Result<NaiveDateTime, ProcessingError> {
    let dt = now.clone();
    apply_duration(in_ast.0, dt, Direction::Forwards)
}

fn parse_ago(ago: Ago, now: &NaiveDateTime) -> Result<NaiveDateTime, ProcessingError> {
    match ago {
        Ago::AgoFromNow(ago) => {
            let dt = now.clone();
            apply_duration(ago, dt, Direction::Backwards)
        }
        Ago::AgoFromTime(ago, time) => {
            let human_time = parse_human_time(*time, now.clone())
                .map_err(|e| ProcessingError::InnerHumanTimeParse(Box::new(e)))?;
            let dt = match human_time {
                ParseResult::DateTime(dt) => dt,
                ParseResult::Date(date) => NaiveDateTime::new(date, now.time()),
                ParseResult::Time(time) => NaiveDateTime::new(now.date(), time),
            };
            apply_duration(ago, dt, Direction::Backwards)
        }
    }
}

#[derive(PartialEq, Eq)]
enum Direction {
    Forwards,
    Backwards,
}

fn apply_duration(
    duration: AstDuration,
    mut dt: NaiveDateTime,
    direction: Direction,
) -> Result<NaiveDateTime, ProcessingError> {
    for quant in duration.0 {
        match quant {
            Quantifier::Year(years) => {
                let years = years as i32;
                if direction == Direction::Forwards {
                    dt = dt
                        .with_year(dt.year() + years)
                        .ok_or(ProcessingError::InvalidDate {
                            year: dt.year() + years,
                            month: dt.month(),
                            day: dt.day(),
                        })?;
                } else {
                    dt = dt
                        .with_year(dt.year() - years)
                        .ok_or(ProcessingError::InvalidDate {
                            year: dt.year() - years,
                            month: dt.month(),
                            day: dt.day(),
                        })?;
                }
            }
            Quantifier::Month(months) => {
                if direction == Direction::Forwards {
                    dt = dt.checked_add_months(Months::new(months)).ok_or(
                        ProcessingError::AddToDate {
                            unit: "months".to_string(),
                            count: months,
                            date: dt,
                        },
                    )?
                } else {
                    dt = dt.checked_sub_months(Months::new(months)).ok_or(
                        ProcessingError::SubtractFromDate {
                            unit: "months".to_string(),
                            count: months,
                            date: dt,
                        },
                    )?
                }
            }
            Quantifier::Week(weeks) => {
                if direction == Direction::Forwards {
                    dt = dt.checked_add_days(Days::new(weeks as u64 * 7)).ok_or(
                        ProcessingError::AddToDate {
                            unit: "weeks".to_string(),
                            count: weeks,
                            date: dt,
                        },
                    )?
                } else {
                    dt = dt.checked_sub_days(Days::new(weeks as u64 * 7)).ok_or(
                        ProcessingError::AddToDate {
                            unit: "weeks".to_string(),
                            count: weeks,
                            date: dt,
                        },
                    )?
                }
            }
            Quantifier::Day(days) => {
                if direction == Direction::Forwards {
                    dt = dt.checked_add_days(Days::new(days as u64)).ok_or(
                        ProcessingError::AddToDate {
                            unit: "days".to_string(),
                            count: days,
                            date: dt,
                        },
                    )?
                } else {
                    dt = dt.checked_sub_days(Days::new(days as u64)).ok_or(
                        ProcessingError::AddToDate {
                            unit: "days".to_string(),
                            count: days,
                            date: dt,
                        },
                    )?
                }
            }
            Quantifier::Hour(hours) => {
                if direction == Direction::Forwards {
                    dt = dt + ChronoDuration::hours(hours as i64)
                } else {
                    dt = dt - ChronoDuration::hours(hours as i64)
                }
            }
            Quantifier::Minute(minutes) => {
                if direction == Direction::Forwards {
                    dt = dt + ChronoDuration::minutes(minutes as i64)
                } else {
                    dt = dt - ChronoDuration::minutes(minutes as i64)
                }
            }
            Quantifier::Second(seconds) => {
                if direction == Direction::Forwards {
                    dt = dt + ChronoDuration::seconds(seconds as i64)
                } else {
                    dt = dt - ChronoDuration::seconds(seconds as i64)
                }
            }
        };
    }

    Ok(dt)
}

fn relative_date_time_unit(
    relative: RelativeSpecifier,
    time_unit: TimeUnit,
    now: NaiveDateTime,
) -> Result<NaiveDateTime, ProcessingError> {
    let quantifier = match time_unit {
        TimeUnit::Year => Quantifier::Year(1),
        TimeUnit::Month => Quantifier::Month(1),
        TimeUnit::Week => Quantifier::Week(1),
        TimeUnit::Day => Quantifier::Day(1),
        TimeUnit::Hour | TimeUnit::Minute | TimeUnit::Second => {
            unreachable!("Non-date time units should never be used in this function.")
        }
    };


    match relative {
        RelativeSpecifier::This => Ok(now),
        RelativeSpecifier::Next => apply_duration(AstDuration(vec![quantifier]), now, Direction::Forwards),
        RelativeSpecifier::Last => apply_duration(AstDuration(vec![quantifier]), now, Direction::Backwards),
    }
}

fn find_weekday_relative_week(
    relative: RelativeSpecifier,
    weekday: Weekday,
    now: NaiveDate,
) -> Result<NaiveDate, ProcessingError> {
    let day_offset = -(now.weekday().num_days_from_monday() as i64);
    let week_offset = match relative {
        RelativeSpecifier::This => 0,
        RelativeSpecifier::Next => 1,
        RelativeSpecifier::Last => -1,
    } * 7;
    let offset = day_offset + week_offset;

    let now = if offset.is_positive() {
        now.checked_add_days(Days::new(offset.unsigned_abs()))
            .ok_or(ProcessingError::AddToNow {
                unit: "days".to_string(),
                count: offset.unsigned_abs() as u32,
            })?
    } else {
        now.checked_sub_days(Days::new(offset.unsigned_abs()))
            .ok_or(ProcessingError::SubtractFromNow {
                unit: "days".to_string(),
                count: offset.unsigned_abs() as u32,
            })?
    };

    find_weekday_relative(RelativeSpecifier::This, weekday, now)
}

fn find_weekday_relative(
    relative: RelativeSpecifier,
    weekday: Weekday,
    now: NaiveDate,
) -> Result<NaiveDate, ProcessingError> {
    match relative {
        RelativeSpecifier::This | RelativeSpecifier::Next => {
            if matches!(relative, RelativeSpecifier::This) && now.weekday() == weekday {
                return Ok(now.clone());
            }

            let current_weekday = now.weekday().num_days_from_monday();
            let target_weekday = weekday.num_days_from_monday();

            let offset = if target_weekday > current_weekday {
                target_weekday - current_weekday
            } else {
                7 - current_weekday + target_weekday
            };

            now.checked_add_days(Days::new(offset as u64))
                .ok_or(ProcessingError::AddToNow {
                    unit: "days".to_string(),
                    count: offset,
                })
        }
        RelativeSpecifier::Last => {
            let current_weekday = now.weekday().num_days_from_monday();
            let target_weekday = weekday.num_days_from_monday();

            let offset = if target_weekday >= current_weekday {
                7 + current_weekday - target_weekday
            } else {
                current_weekday - target_weekday
            };

            now.checked_sub_days(Days::new(offset as u64))
                .ok_or(ProcessingError::SubtractFromNow {
                    unit: "days".to_string(),
                    count: offset,
                })
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
    use concat_idents::concat_idents;

    /// Generates the test cases to remove a bunch of boilerplate code for the test setup.
    macro_rules! generate_test_cases {
        ( $( $case:literal = $expected:literal ),* ) => {
            $(
                concat_idents!(fn_name = parse_, $case {
                    #[test]
                    fn fn_name () {
                        let input = $case.to_lowercase();
                        let now = NaiveDateTime::new(NaiveDate::from_ymd_opt(2010, 1, 1).unwrap(), NaiveTime::from_hms_opt(0, 0, 0).unwrap());
                        let result = from_human_time(&input, now).unwrap();
                        let expected = NaiveDateTime::parse_from_str( $expected , "%Y-%m-%d %H:%M:%S").unwrap();

                        let result = match result {
                            ParseResult::DateTime(datetime) => datetime,
                            ParseResult::Date(date) => NaiveDateTime::new(date, now.time()),
                            ParseResult::Time(time) => NaiveDateTime::new(now.date(), time),
                        };

                        println!("Result: {result}\nExpected: {expected}\nNote: Maximum difference between these values allowed is 10ms.");
                        assert!((result - expected).abs() < chrono::Duration::milliseconds(10));
                    }
                });
            )*
        };
    }

    /// Variant of aboce to check if parsing fails gracefully
    macro_rules! generate_test_cases_error {
        ( $( $case:literal ),* ) => {
            $(
                concat_idents!(fn_name = fail_parse_, $case {
                    #[test]
                    fn fn_name () {
                        let input = $case.to_lowercase();
                        let now = NaiveDateTime::new(NaiveDate::from_ymd_opt(2010, 1, 1).unwrap(), NaiveTime::from_hms_opt(0, 0, 0).unwrap());
                        let result = from_human_time(&input, now);

                        println!("Result: {result:#?}\nExpected: Error");
                        assert!(result.is_err());
                    }
                });
            )*
        };
    }

    generate_test_cases!(
        "15:10" = "2010-01-01 15:10:00",
        "Today 18:30" = "2010-01-01 18:30:00",
        "Yesterday 18:30" = "2009-12-31 18:30:00",
        "Tomorrow 18:30" = "2010-01-02 18:30:00",
        "Overmorrow 18:30" = "2010-01-03 18:30:00",
        "2022-11-07 13:25:30" = "2022-11-07 13:25:30",
        "07 February 2015" = "2015-02-07 00:00:00",
        "07 February" = "2010-02-07 00:00:00",
        "15:20 Friday" = "2010-01-08 15:20:00",
        "This Friday 17:00" = "2010-01-01 17:00:00",
        "Next Friday 17:00" = "2010-01-08 17:00:00",
        "13:25, Next Tuesday" = "2010-01-05 13:25:00",
        "Last Friday at 19:45" = "2009-12-25 19:45:00",
        "Next week" = "2010-01-08 00:00:00",
        "This week" = "2010-01-01 00:00:00",
        "Last week" = "2009-12-25 00:00:00",
        "Next week Monday" = "2010-01-04 00:00:00",
        "This week Friday" = "2010-01-01 00:00:00",
        "This week Monday" = "2009-12-28 00:00:00",
        "Last week Tuesday" = "2009-12-22 00:00:00",
        "This Friday" = "2010-01-01 00:00:00",
        "Next Friday" = "2010-01-08 00:00:00",
        "Last Friday" = "2009-12-25 00:00:00",
        "In 3 days" = "2010-01-04 00:00:00",
        "In 2 hours" = "2010-01-01 02:00:00",
        "In 5 minutes and 30 seconds" = "2010-01-01 00:05:30",
        "10 seconds ago" = "2009-12-31 23:59:50",
        "10 hours and 5 minutes ago" = "2009-12-31 13:55:00",
        "2 hours, 32 minutes and 7 seconds ago" = "2009-12-31 21:27:53",
        "1 years, 2 months, 3 weeks, 5 days, 8 hours, 17 minutes and 45 seconds ago" =
            "2008-10-05 15:42:15",
        "1 year, 1 month, 1 week, 1 day, 1 hour, 1 minute and 1 second ago" = "2008-11-22 22:58:59",
        "A year ago" = "2009-01-01 00:00:00",
        "A month ago" = "2009-12-01 00:00:00",
        "3 months ago" = "2009-10-01 00:00:00",
        "6 months ago" = "2009-07-01 00:00:00",
        "7 months ago" = "2009-06-01 00:00:00",
        "In 7 months" = "2010-08-01 00:00:00",
        "A week ago" = "2009-12-25 00:00:00",
        "A day ago" = "2009-12-31 00:00:00",
        "An hour ago" = "2009-12-31 23:00:00",
        "A minute ago" = "2009-12-31 23:59:00",
        "A second ago" = "2009-12-31 23:59:59",
        "now" = "2010-01-01 00:00:00",
        "Overmorrow" = "2010-01-03 00:00:00",
        "7 days ago at 04:00" = "2009-12-25 04:00:00",
        "12 hours ago at 04:00" = "2009-12-31 16:00:00",
        "12 hours ago at today" = "2009-12-31 12:00:00",
        "12 hours ago at 7 days ago" = "2009-12-24 12:00:00",
        "7 days ago at 7 days ago" = "2009-12-18 00:00:00"
    );

    generate_test_cases_error!("2023-11-31");
}
