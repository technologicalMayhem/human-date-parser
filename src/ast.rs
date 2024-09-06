use chrono::Month;
use pest_consume::{match_nodes, Error, Parser as ConsumeParser};
use pest_derive::Parser;

use crate::{InternalError, ParseError};

type ParserResult<T> = std::result::Result<T, Error<Rule>>;
type Node<'i> = pest_consume::Node<'i, Rule, ()>;

pub fn build_ast_from(str: &str) -> Result<HumanTime, ParseError> {
    let result = DateTimeParser::parse(Rule::HumanTime, &str)
        .and_then(|result| result.single())
        .map_err(|_| ParseError::InvalidFormat)?;

    DateTimeParser::HumanTime(result)
        .map_err(|_| ParseError::InternalError(InternalError::FailedToBuildAst))
}

#[derive(Parser)]
#[grammar = "date_time.pest"]
struct DateTimeParser;

#[pest_consume::parser]
impl DateTimeParser {
    fn HumanTime(input: Node) -> ParserResult<HumanTime> {
        Ok(match_nodes!(input.into_children();
            [DateTime(dt)] => HumanTime::DateTime(dt),
            [Date(d)] => HumanTime::Date(d),
            [Time(t)] => HumanTime::Time(t),
            [In(i)] => HumanTime::In(i),
            [Ago(a)] => HumanTime::Ago(a),
            [Now(_)] => HumanTime::Now,
        ))
    }

    fn DateTime(input: Node) -> ParserResult<DateTime> {
        Ok(match_nodes!(input.into_children();
            [Date(date), Time(time)] => DateTime{ date, time },
            [Time(time), Date(date)] => DateTime{ date, time },
        ))
    }

    fn IsoDate(input: Node) -> ParserResult<IsoDate> {
        Ok(match_nodes!(input.into_children();
            [Num(year), Num(month), Num(day)] => IsoDate{year, month, day},
        ))
    }

    fn Date(input: Node) -> ParserResult<Date> {
        Ok(match_nodes!(input.into_children();
            [Today(_)] => Date::Today,
            [Tomorrow(_)] => Date::Tomorrow,
            [Overmorrow(_)] => Date::Overmorrow,
            [Yesterday(_)] => Date::Yesterday,
            [IsoDate(iso)] => Date::IsoDate(iso),
            [Num(d), Month_Name(m), Num(y)] => Date::DayMonthYear(d, m, y),
            [Num(d), Month_Name(m)] => Date::DayMonth(d, m),
            [RelativeSpecifier(r), Week(_), Weekday(wd)] => Date::RelativeWeekWeekday(r, wd),
            [RelativeSpecifier(r), TimeUnit(tu)] => Date::RelativeTimeUnit(r, tu),
            [RelativeSpecifier(r), Weekday(wd)] => Date::RelativeWeekday(r, wd),
            [Weekday(wd)] => Date::UpcomingWeekday(wd),
        ))
    }

    fn Week(input: Node) -> ParserResult<Week> {
        Ok(Week {})
    }

    fn Ago(input: Node) -> ParserResult<Ago> {
        Ok(match_nodes!(input.into_children();
            [Duration(d)] => Ago::AgoFromNow(d),
            [Duration(d), HumanTime(ht)] => Ago::AgoFromTime(d, Box::new(ht)),
        ))
    }

    fn Now(input: Node) -> ParserResult<Now> {
        Ok(Now {})
    }

    fn Today(input: Node) -> ParserResult<Today> {
        Ok(Today {})
    }

    fn Tomorrow(input: Node) -> ParserResult<Tomorrow> {
        Ok(Tomorrow {})
    }

    fn Yesterday(input: Node) -> ParserResult<Yesterday> {
        Ok(Yesterday {})
    }

    fn Overmorrow(input: Node) -> ParserResult<Overmorrow> {
        Ok(Overmorrow {})
    }

    fn Time(input: Node) -> ParserResult<Time> {
        Ok(match_nodes!(input.into_children();
            [Num(h), Num(m)] => Time::HourMinute(h, m),
            [Num(h), Num(m), Num(s)] => Time::HourMinuteSecond(h, m, s),
        ))
    }

    fn In(input: Node) -> ParserResult<In> {
        Ok(match_nodes!(input.into_children();
            [Duration(d)] => In(d),
        ))
    }

    fn Duration(input: Node) -> ParserResult<Duration> {
        Ok(match_nodes!(input.into_children();
            [Quantifier(q)..] => Duration(q.collect()),
            [SingleUnit(su)] => Duration(vec![su]),
        ))
    }

    fn SingleUnit(input: Node) -> ParserResult<Quantifier> {
        Ok(match_nodes!(input.into_children();
            [TimeUnit(u)] => match u {
                TimeUnit::Year => Quantifier::Year(1),
                TimeUnit::Month => Quantifier::Month(1),
                TimeUnit::Week => Quantifier::Week(1),
                TimeUnit::Day => Quantifier::Day(1),
                TimeUnit::Hour => Quantifier::Hour(1),
                TimeUnit::Minute => Quantifier::Minute(1),
                TimeUnit::Second => Quantifier::Second(1),
            }
        ))
    }

    fn RelativeSpecifier(input: Node) -> ParserResult<RelativeSpecifier> {
        Ok(match_nodes!(input.into_children();
            [This(_)] => RelativeSpecifier::This,
            [Next(_)] => RelativeSpecifier::Next,
            [Last(_)] => RelativeSpecifier::Last,
        ))
    }

    fn This(input: Node) -> ParserResult<This> {
        Ok(This {})
    }

    fn Next(input: Node) -> ParserResult<Next> {
        Ok(Next {})
    }

    fn Last(input: Node) -> ParserResult<Last> {
        Ok(Last {})
    }

    fn Num(input: Node) -> ParserResult<u32> {
        input.as_str().parse::<u32>().map_err(|e| input.error(e))
    }

    fn Quantifier(input: Node) -> ParserResult<Quantifier> {
        Ok(match_nodes!(input.into_children();
            [Num(n), TimeUnit(u)] => match u {
                TimeUnit::Year => Quantifier::Year(n),
                TimeUnit::Month => Quantifier::Month(n),
                TimeUnit::Week => Quantifier::Week(n),
                TimeUnit::Day => Quantifier::Day(n),
                TimeUnit::Hour => Quantifier::Hour(n),
                TimeUnit::Minute => Quantifier::Minute(n),
                TimeUnit::Second => Quantifier::Second(n),
            }
        ))
    }

    fn TimeUnit(input: Node) -> ParserResult<TimeUnit> {
        if let Some(rule) = input.children().next() {
            Ok(match rule.as_rule() {
                Rule::Year => TimeUnit::Year,
                Rule::Month => TimeUnit::Month,
                Rule::Week => TimeUnit::Week,
                Rule::Day => TimeUnit::Day,
                Rule::Hour => TimeUnit::Hour,
                Rule::Minute => TimeUnit::Minute,
                Rule::Second => TimeUnit::Second,
                _ => unreachable!(),
            })
        } else {
            Err(input.error("Unreachable"))
        }
    }

    fn Weekday(input: Node) -> ParserResult<Weekday> {
        if let Some(rule) = input.children().next() {
            Ok(match rule.as_rule() {
                Rule::Monday => Weekday::Monday,
                Rule::Tuesday => Weekday::Tuesday,
                Rule::Wednesday => Weekday::Wednesday,
                Rule::Thursday => Weekday::Thursday,
                Rule::Friday => Weekday::Friday,
                Rule::Saturday => Weekday::Saturday,
                Rule::Sunday => Weekday::Sunday,
                _ => unreachable!(),
            })
        } else {
            Err(input.error("Unreachable"))
        }
    }

    fn Month_Name(input: Node) -> ParserResult<Month> {
        if let Some(rule) = input.children().next() {
            Ok(match rule.as_rule() {
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
                _ => unreachable!(),
            })
        } else {
            Err(input.error("Unreachable"))
        }
    }
}

#[derive(Debug)]
pub enum HumanTime {
    DateTime(DateTime),
    Date(Date),
    Time(Time),
    In(In),
    Ago(Ago),
    Now,
}

#[derive(Debug)]
pub struct DateTime {
    pub date: Date,
    pub time: Time,
}

#[derive(Debug)]
pub struct IsoDate {
    pub year: u32,
    pub month: u32,
    pub day: u32,
}

#[derive(Debug)]
pub enum Date {
    Today,
    Tomorrow,
    Overmorrow,
    Yesterday,
    IsoDate(IsoDate),
    DayMonthYear(u32, Month, u32),
    DayMonth(u32, Month),
    RelativeWeekWeekday(RelativeSpecifier, Weekday),
    RelativeTimeUnit(RelativeSpecifier, TimeUnit),
    RelativeWeekday(RelativeSpecifier, Weekday),
    UpcomingWeekday(Weekday),
}

#[derive(Debug)]
struct Today;
#[derive(Debug)]
struct Tomorrow;
#[derive(Debug)]
struct Yesterday;
#[derive(Debug)]
struct Overmorrow;

#[derive(Debug)]
pub enum Time {
    HourMinute(u32, u32),
    HourMinuteSecond(u32, u32, u32),
}

#[derive(Debug)]
pub struct In(pub Duration);

#[derive(Debug)]
pub enum Ago {
    AgoFromNow(Duration),
    AgoFromTime(Duration, Box<HumanTime>),
}

#[derive(Debug, PartialEq)]
pub struct Duration(pub Vec<Quantifier>);

#[derive(Debug)]
struct Now;

#[derive(Debug)]
pub enum RelativeSpecifier {
    This,
    Next,
    Last,
}

#[derive(Debug)]
struct This;
#[derive(Debug)]
struct Next;
#[derive(Debug)]
struct Last;

#[derive(PartialEq, Eq, Debug)]
pub enum Quantifier {
    Year(u32),
    Month(u32),
    Week(u32),
    Day(u32),
    Hour(u32),
    Minute(u32),
    Second(u32),
}

#[derive(PartialEq, Eq, Debug)]
pub enum TimeUnit {
    Year,
    Month,
    Week,
    Day,
    Hour,
    Minute,
    Second,
}

#[derive(Debug)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl From<Weekday> for chrono::Weekday {
    fn from(value: Weekday) -> Self {
        match value {
            Weekday::Monday => chrono::Weekday::Mon,
            Weekday::Tuesday => chrono::Weekday::Tue,
            Weekday::Wednesday => chrono::Weekday::Wed,
            Weekday::Thursday => chrono::Weekday::Thu,
            Weekday::Friday => chrono::Weekday::Fri,
            Weekday::Saturday => chrono::Weekday::Sat,
            Weekday::Sunday => chrono::Weekday::Sun,
        }
    }
}

#[derive(Debug)]
struct Week {}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use concat_idents::concat_idents;
    use pest_consume::Parser;

    use super::{DateTimeParser, Rule};

    macro_rules! generate_test_cases {
        ( $( $case:literal ),* ) => {
            $(
                concat_idents!(fn_name = parse_, $case {
                    #[test]
                    fn fn_name () {
                        let input = $case.to_lowercase();
                        let result = DateTimeParser::parse(Rule::HumanTime, &input)
                            .and_then(|result| result.single())
                            .unwrap();

                        DateTimeParser::HumanTime(result).unwrap();
                    }
                });
            )*
        };
    }

    generate_test_cases!(
        "Today 18:30",
        "Yesterday 18:30",
        "Tomorrow 18:30",
        "Overmorrow 18:30",
        "2022-11-07 13:25:30",
        "15:20 Friday",
        "This Friday 17:00",
        "Next Friday 17:00",
        "13:25, Next Tuesday",
        "Last Friday at 19:45",
        "Next week",
        "This week",
        "Last week",
        "Next week Monday",
        "This week Friday",
        "This week Monday",
        "Last week Tuesday",
        "In 3 days",
        "In 2 hours",
        "In 5 minutes and 30 seconds",
        "10 seconds ago",
        "10 hours and 5 minutes ago",
        "2 hours, 32 minutes and 7 seconds ago",
        "1 years, 2 months, 3 weeks, 5 days, 8 hours, 17 minutes and 45 seconds ago",
        "1 year, 1 month, 1 week, 1 day, 1 hour, 1 minute and 1 second ago",
        "A year ago",
        "A month ago",
        "3 months ago",
        "6 months ago",
        "7 months ago",
        "In 7 months",
        "A week ago",
        "A day ago",
        "An hour ago",
        "A minute ago",
        "A second ago",
        "now",
        "Overmorrow",
        "7 days ago at 04:00",
        "12 hours ago at 04:00",
        "12 hours ago at today",
        "12 hours ago at 7 days ago",
        "7 days ago at 7 days ago"
    );
}
