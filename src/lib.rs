use std::{
    fmt::Display,
    ops::{Add, Sub},
    str::FromStr,
};

use chrono::{
    DateTime, Datelike, Duration, Local, Month, NaiveDate, NaiveDateTime, NaiveTime, Weekday,
};
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;
use thiserror::Error;

#[derive(Parser)]
#[grammar = "date_time.pest"]
struct DateTimeParser;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("The data has a invalid format")]
    InvalidFormat,
    #[error("The value {amount} is invalid.")]
    ValueInvalid { amount: String },
    #[error("The date you given is a impossible date.")]
    ImpossibleDate,
    #[error("You gave a value of {value}. It was only allowed to be between {lower} and {upper}.")]
    ValueOutOfRange {
        lower: String,
        upper: String,
        value: String,
    },
}

#[cfg(not(test))]
/// Returns the current time in the local timezone.
macro_rules! now {
    () => {
        Local::now()
    };
}

#[cfg(test)]
/// If we are in a test enviroment we will just pretend it is 2010-01-01 00:00:00 right now.
macro_rules! now {
    () => {
        NaiveDateTime::parse_from_str("2010-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
    };
}

#[derive(Debug)]
pub enum ParseResult {
    DateTime(DateTime<Local>),
    Date(NaiveDate),
    Time(NaiveTime),
}

trait PairHelper<'a> {
    fn vec(self) -> Vec<Pair<'a, Rule>>;
    fn clone_vec(&self) -> Vec<Pair<'a, Rule>>;
}
impl<'a> PairHelper<'a> for Pair<'a, Rule> {
    fn vec(self) -> Vec<Pair<'a, Rule>> {
        self.into_inner().collect()
    }

    fn clone_vec(&self) -> Vec<Pair<'a, Rule>> {
        self.clone().into_inner().collect()
    }
}

fn rules(v: &[Pair<'_, Rule>]) -> Vec<Rule> {
    v.iter().map(|pair| pair.as_rule()).collect()
}

/// Converts a human expression of a date into a more usable one.
///
/// # Errors
///
/// This function will return an error if the string contains values than can not be parsed into a date.
///
/// # Examples
/// ```
/// let date = from_human_time("Last Friday at 19:45").unwrap();
/// match date {
///     ParseResult::DateTime(date) => println!("{date}"),
///     _ => unreachable!()
/// }
/// ```
pub fn from_human_time(str: &str) -> Result<ParseResult, ParseError> {
    let lowercase = str.to_lowercase();
    let mut parsed = match DateTimeParser::parse(Rule::HumanTime, &lowercase) {
        Ok(parsed) => parsed,
        Err(_) => return Err(ParseError::InvalidFormat),
    };

    let head = parsed.next().unwrap();
    let rule = head.as_rule();
    let result: ParseResult = match rule {
        Rule::DateTime => ParseResult::DateTime(parse_datetime(head)?),
        Rule::Date => ParseResult::Date(parse_date(head)?),
        Rule::Time => ParseResult::Time(parse_time(head)?),
        Rule::In | Rule::Ago => ParseResult::DateTime(parse_in_or_ago(head, rule)?),
        Rule::Now => ParseResult::DateTime(now!()),
        _ => unreachable!(),
    };

    Ok(result)
}

/// Parse a string DateTime element into it's chrono equivalent.
///
/// # Errors
///
/// This function will return an error if the pair contains values than can not be parsed into a date.
fn parse_datetime(head: Pair<Rule>) -> Result<DateTime<Local>, ParseError> {
    let date;
    let time;
    let mut iter = head.into_inner();
    let first = iter.next().unwrap();
    let second = iter.next().unwrap();

    if first.as_rule() == Rule::Date {
        date = parse_date(first)?;
        time = parse_time(second)?;
    } else {
        date = parse_date(second)?;
        time = parse_time(first)?;
    }

    let date_time = NaiveDateTime::new(date, time);
    let date_time = date_time.and_local_timezone(Local).unwrap();
    Ok(date_time)
}

/// Parses a string in the 'In...' or '...ago' format into a valid DateTime.
///
/// # Errors
///
/// This function will return an error if the pair contains values than can not be parsed into a date.
fn parse_in_or_ago(head: Pair<Rule>, rule: Rule) -> Result<DateTime<Local>, ParseError> {
    let durations = collect_durations(head.into_inner().next().unwrap())?;
    let mut full_duration = Duration::zero();
    for duration in durations {
        full_duration = full_duration.add(duration);
    }
    Ok(match rule {
        Rule::In => now!() + full_duration,
        Rule::Ago => now!() - full_duration,
        _ => unreachable!(),
    })
}

/// Parses the date component of the string into a `NaiveDate`.
///
/// # Errors
///
/// This function will return an error if the pair contains values than can not be parsed into a `NaiveDate`.
fn parse_date(pair: Pair<Rule>) -> Result<NaiveDate, ParseError> {
    let date = pair.vec();
    match rules(&date)[..] {
        [Rule::Today] => Ok(now!().date_naive()),
        [Rule::Tomorrow] => Ok(now!().add(Duration::days(1)).date_naive()),
        [Rule::Overmorrow] => Ok(now!().add(Duration::days(2)).date_naive()),
        [Rule::Yesterday] => Ok(now!().sub(Duration::days(1)).date_naive()),
        [Rule::IsoDate] => NaiveDate::from_str(date[0].as_str()).map_err(|e| match e.kind() {
            chrono::format::ParseErrorKind::Impossible => ParseError::ImpossibleDate,
            _ => ParseError::InvalidFormat,
        }),
        [Rule::Num, Rule::Month_Name] | [Rule::Num, Rule::Month_Name, Rule::Num] => {
            let day = parse_in_range(date[0].as_str(), 1, 31)?;

            let month_rule = date[1].clone_vec()[0].as_rule();
            let month = month_from_rule(month_rule).number_from_month();

            let year = match date.get(2) {
                Some(rule) => parse_in_range(rule.as_str(), 0, 10000)?,
                None => now!().year(),
            };

            let date = match NaiveDate::from_ymd_opt(year, month, day) {
                Some(date) => date,
                None => return Err(ParseError::InvalidFormat),
            };
            Ok(date)
        }
        [Rule::RelativeSpecifier, Rule::TimeUnit] => {
            let unit = date[1].clone_vec()[0].as_rule();
            let duration = create_duration(unit, 1)?;
            match date[0].clone_vec()[0].as_rule() {
                Rule::This => Ok(now!().date_naive()),
                Rule::Next => Ok(now!().add(duration).date_naive()),
                Rule::Last => Ok(now!().sub(duration).date_naive()),
                _ => unreachable!(),
            }
        }
        [Rule::RelativeSpecifier, Rule::Weekday]
        | [Rule::RelativeSpecifier, Rule::Week, Rule::Weekday] => {
            let specifier = date[0].clone_vec()[0].as_rule();

            let specific_weekday = if date[1].as_rule() == Rule::Weekday {
                date[1].clone_vec()[0].as_rule()
            } else {
                date[2].clone_vec()[0].as_rule()
            };

            let weekday = weekday_from_rule(specific_weekday);
            let now = now!().date_naive();
            match specifier {
                Rule::This => Ok(find_weekday(now, weekday)),
                Rule::Next => Ok(find_weekday(now.add(Duration::days(7)), weekday)),
                Rule::Last => Ok(find_weekday(now.sub(Duration::days(7)), weekday)),
                _ => unreachable!(),
            }
        }
        [Rule::Weekday] => {
            let specific_weekday = date[0].clone_vec()[0].as_rule();
            let weekday = weekday_from_rule(specific_weekday);
            Ok(find_next_weekday_occurence(now!().date_naive(), weekday))
        }
        _ => unreachable!(),
    }
}

/// Finds the date for a given Weekday, either as this, next or last occurence of it.
fn find_weekday(date: NaiveDate, weekday: Weekday) -> NaiveDate {
    let diff = date.weekday().num_days_from_monday() as i64 - weekday.num_days_from_monday() as i64;
    date.sub(Duration::days(diff))
}

/// Finds the next occurence of the weekday
fn find_next_weekday_occurence(date: NaiveDate, weekday: Weekday) -> NaiveDate {
    let current = now!().weekday().num_days_from_monday();
    let next = weekday.num_days_from_monday();

    let days_to_add = if current < next {
        (next - current).into()
    } else {
        (7 - (current - next)).into()
    };

    date.add(Duration::days(days_to_add))
}

/// Parsed the time component into a `NaiveTime`.
///
/// # Errors
///
/// This function will return an error if the pair contains values than can not be parsed into `NaiveTime`.
fn parse_time(pair: Pair<Rule>) -> Result<NaiveTime, ParseError> {
    let time = match NaiveTime::parse_from_str(pair.as_str(), "%H:%M:%S") {
        Ok(time) => time,
        Err(_) => match NaiveTime::parse_from_str(pair.as_str(), "%H:%M") {
            Ok(time) => time,
            Err(_) => return Err(ParseError::InvalidFormat),
        },
    };
    Ok(time)
}

/// Parses a `str` into a number that is clamped withing the given lower and upper bound.
///
/// # Errors
///
/// This function will return an error if `str` could not be parsed or it is outside the given bounds.
fn parse_in_range<T>(str: &str, lower: T, upper: T) -> Result<T, ParseError>
where
    T: FromStr + PartialOrd<T> + Display,
{
    let value: T = match str.parse() {
        Ok(val) => val,
        Err(_) => return Err(ParseError::ValueInvalid { amount: str.into() }),
    };

    if value < lower || value > upper {
        return Err(ParseError::ValueOutOfRange {
            lower: lower.to_string(),
            upper: upper.to_string(),
            value: str.to_string(),
        });
    }

    Ok(value)
}

/// Parses all the durations in the Duration component and returns them as `Vec<Duration>`.
///
/// # Errors
///
/// This function will return an error if the pair contains invalid durations.
fn collect_durations(duration_rule: Pair<Rule>) -> Result<Vec<Duration>, ParseError> {
    let mut durations = Vec::new();

    for rule in duration_rule.into_inner() {
        match rule.as_rule() {
            Rule::Quantifier => {
                let mut amount: i64 = 0;
                let mut time_type = Rule::Minute;

                for inner in rule.into_inner() {
                    match inner.as_rule() {
                        Rule::Num => {
                            amount = match inner.as_str().parse() {
                                Ok(num) => num,
                                Err(_) => {
                                    return Err(ParseError::ValueInvalid {
                                        amount: inner.as_str().into(),
                                    })
                                }
                            }
                        }
                        Rule::TimeUnit => time_type = inner.into_inner().next().unwrap().as_rule(),
                        _ => unreachable!(),
                    }
                }

                durations.push(create_duration(time_type, amount)?);
            }
            Rule::SingleUnit => {
                for inner in rule.into_inner() {
                    if inner.as_rule() == Rule::TimeUnit {
                        durations.push(create_duration(
                            inner.into_inner().next().unwrap().as_rule(),
                            1,
                        )?);
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    Ok(durations)
}

/// Combines the rule and amount into a `Duration`.
///
/// # Errors
///
/// This function will return an error if the pair contains values than can not be parsed into a `Duration`.
fn create_duration(rule: Rule, amount: i64) -> Result<Duration, ParseError> {
    let dur = match rule {
        Rule::Year => {
            let now = now!();
            let years: i32 = match amount.try_into() {
                Ok(years) => years,
                Err(_) => {
                    return Err(ParseError::ValueInvalid {
                        amount: amount.to_string(),
                    })
                }
            };
            let next_year = match now.with_year(now.year() + years) {
                Some(year) => year,
                None => {
                    return Err(ParseError::ValueInvalid {
                        amount: amount.to_string(),
                    })
                }
            };
            next_year - now
        }
        Rule::Month => {
            let now = now!();
            let months: u32 = match amount.try_into() {
                Ok(months) => months,
                Err(_) => {
                    return Err(ParseError::ValueInvalid {
                        amount: amount.to_string(),
                    })
                }
            };
            let next_month = match now.with_month0((now.month0() + months) % 12) {
                Some(month) => month,
                None => {
                    return Err(ParseError::ValueInvalid {
                        amount: amount.to_string(),
                    })
                }
            };

            next_month - now
        }
        Rule::Week => Duration::days(amount * 7),
        Rule::Day => Duration::days(amount),
        Rule::Hour => Duration::hours(amount),
        Rule::Minute => Duration::minutes(amount),
        Rule::Second => Duration::seconds(amount),
        _ => unreachable!(),
    };

    Ok(dur)
}

/// Returns the `chrono::month` equivalent of a parser rule.
///
/// # Panics
///
/// Panics if the given rule does not correspond to a month.
fn month_from_rule(rule: Rule) -> Month {
    match rule {
        Rule::January => Month::January,
        Rule::February => Month::February,
        Rule::March => Month::March,
        Rule::April => Month::April,
        Rule::May => Month::May,
        Rule::June => Month::June,
        Rule::July => Month::July,
        Rule::August => Month::August,
        Rule::September => Month::September,
        Rule::October => Month::October,
        Rule::November => Month::November,
        Rule::December => Month::December,
        _ => panic!("Tried to convert something that isn't a month to a month. This is a bug. Tried to convert: {:?}", rule),
    }
}

/// Returns the `chrono::weekday` equivalent of a parser rule.
///
/// # Panics
///
/// Panics if the given rule does not correspond to a weekday.
fn weekday_from_rule(rule: Rule) -> Weekday {
    match rule {
        Rule::Monday => Weekday::Mon,
        Rule::Tuesday => Weekday::Tue,
        Rule::Wednesday => Weekday::Wed,
        Rule::Thursday => Weekday::Thu,
        Rule::Friday => Weekday::Fri,
        Rule::Saturday => Weekday::Sat,
        Rule::Sunday => Weekday::Sun,
        _ => {
            panic!("Tried to convert {rule:?} to a weekday, which is not possible. This is a bug.")
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
                        let result = from_human_time(&input).unwrap();
                        let expected = NaiveDateTime::parse_from_str( $expected , "%Y-%m-%d %H:%M:%S").unwrap().and_local_timezone(Local).unwrap();

                        let result = match result {
                            ParseResult::DateTime(datetime) => datetime,
                            ParseResult::Date(date) => NaiveDateTime::new(date, now!().time()).and_local_timezone(Local).unwrap(),
                            ParseResult::Time(time) => NaiveDateTime::new(now!().date_naive(), time).and_local_timezone(Local).unwrap(),
                        };

                        println!("Result: {result}\nExpected: {expected}\nNote: Maximum difference between these values allowed is 10ms.");
                        assert!(result - expected < Duration::milliseconds(10));
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
                        let result = from_human_time(&input);

                        println!("Result: {result:#?}\nExpected: Error");
                        assert!(result.is_err());
                    }
                });
            )*
        };
    }

    generate_test_cases!(
        "Today 18:30" = "2010-01-01 18:30:00",
        "Yesterday 18:30" = "2009-12-31 18:30:00",
        "Tomorrow 18:30" = "2010-01-02 18:30:00",
        "Overmorrow 18:30" = "2010-01-03 18:30:00",
        "2022-11-07 13:25:30" = "2022-11-07 13:25:30",
        "15:20 Friday" = "2010-01-08 15:20:00",
        "This Friday 17:00" = "2010-01-08 17:00:00",
        "13:25, Next Tuesday" = "2010-01-12 13:25:00",
        "Last Friday at 19:45" = "2009-12-25 19:45:00",
        "Next week" = "2010-01-08 00:00:00",
        "This week" = "2010-01-01 00:00:00",
        "Last week" = "2009-12-25 00:00:00",
        "Next week Monday" = "2010-01-04 00:00:00",
        "This week Friday" = "2010-01-01 00:00:00",
        "This week Monday" = "2009-12-28 00:00:00",
        "Last week Tuesday" = "2009-12-22 00:00:00",
        "In 3 days" = "2010-01-04 00:00:00",
        "In 2 hours" = "2010-01-01 02:00:00",
        "In 5 minutes and 30 seconds" = "2010-01-01 00:05:30",
        "10 seconds ago" = "2009-12-31 23:59:50",
        "10 hours and 5 minutes ago" = "2009-12-31 13:55:00",
        "2 hours, 32 minutes and 7 seconds ago" = "2009-12-31 21:27:53",
        "1 years, 2 months, 3 weeks, 5 days, 8 hours, 17 minutes and 45 seconds ago" =
            "2008-10-07 16:42:15",
        "1 year, 1 month, 1 week, 1 day, 1 hour, 1 minute and 1 second ago" = "2008-11-23 22:58:59",
        "A year ago" = "2009-01-01 00:00:00",
        "A month ago" = "2009-12-01 00:00:00",
        "A week ago" = "2009-12-25 00:00:00",
        "A day ago" = "2009-12-31 00:00:00",
        "An hour ago" = "2009-12-31 23:00:00",
        "A minute ago" = "2009-12-31 23:59:00",
        "A second ago" = "2009-12-31 23:59:59",
        "now" = "2010-01-01 00:00:00",
        "Overmorrow" = "2010-01-03 00:00:00"
    );

    generate_test_cases_error!("2023-11-31");
}
