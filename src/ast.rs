use pest_consume::{match_nodes, Error};
use pest_derive::Parser;

type Result<T> = std::result::Result<T, Error<Rule>>;
type Node<'i> = pest_consume::Node<'i, Rule, ()>;

#[derive(Parser)]
#[grammar = "date_time.pest"]
struct DateTimeParser;

#[pest_consume::parser]
impl DateTimeParser {
    fn HumanTime(input: Node) -> Result<HumanTime> {
        Ok(match_nodes!(input.into_children();
            [DateTime(dt)] => HumanTime::DateTime(dt),
            [Date(d)] => HumanTime::Date(d),
            [Time(t)] => HumanTime::Time(t),
            [In(i)] => HumanTime::In(i),
            [Ago(a)] => HumanTime::Ago(a),
            [Now(_)] => HumanTime::Now,
        ))
    }

    fn DateTime(input: Node) -> Result<DateTime> {
        Ok(match_nodes!(input.into_children();
            [Date(d), Time(t)] => DateTime(d, t),
            [Time(t), Date(d)] => DateTime(d, t),
        ))
    }

    fn IsoDate(input: Node) -> Result<IsoDate> {
        Ok(match_nodes!(input.into_children();
            [Num(d), Num(m), Num(y)] => IsoDate(d, m, y),
        ))
    }

    fn Date(input: Node) -> Result<Date> {
        Ok(match_nodes!(input.into_children();
            [Today(_)] => Date::Today,
            [Tomorrow(_)] => Date::Tomorrow,
            [Overmorrow(_)] => Date::Overmorrow,
            [Yesterday(_)] => Date::Yesterday,
            [IsoDate(iso)] => Date::IsoDate(iso),
            [Num(d), Month(m), Num(y)] => Date::DayMonthYear(d, m, y),
            [Num(d), Month(m)] => Date::DayMonth(d, m),
            [RelativeSpecifier(r), Week(_), Weekday(wd)] => Date::RelativeWeekWeekday(r, wd),
            [RelativeSpecifier(r), TimeUnit(tu)] => Date::RelativeTimeUnit(r, tu),
            [RelativeSpecifier(r), Weekday(wd)] => Date::RelativeWeekday(r, wd),
            [Weekday(wd)] => Date::UpcomingWeekday(wd),
        ))
    }

    fn Week(input: Node) -> Result<Week> {
        Ok(Week {})
    }

    fn Ago(input: Node) -> Result<Ago> {
        Ok(match_nodes!(input.into_children();
            [Duration(d)] => Ago::AgoFromNow(d),
            [Duration(d), HumanTime(ht)] => Ago::AgoFromTime(d, Box::new(ht)),
        ))
    }

    fn Now(input: Node) -> Result<Now> {
        Ok(Now {})
    }

    fn Today(input: Node) -> Result<Today> {
        Ok(Today {})
    }

    fn Tomorrow(input: Node) -> Result<Tomorrow> {
        Ok(Tomorrow {})
    }

    fn Yesterday(input: Node) -> Result<Yesterday> {
        Ok(Yesterday {})
    }

    fn Overmorrow(input: Node) -> Result<Overmorrow> {
        Ok(Overmorrow {})
    }

    fn Time(input: Node) -> Result<Time> {
        Ok(match_nodes!(input.into_children();
            [Num(h), Num(m)] => Time::HourMinute(h, m),
            [Num(h), Num(m), Num(s)] => Time::HourMinuteSecond(h, m, s),
        ))
    }

    fn In(input: Node) -> Result<In> {
        Ok(match_nodes!(input.into_children();
            [Duration(d)] => In(d),
        ))
    }

    fn Duration(input: Node) -> Result<Duration> {
        Ok(match_nodes!(input.into_children();
            [Quantifier(q)..] => Duration(q.collect()),
            [SingleUnit(su)] => Duration(vec![su]),
        ))
    }

    fn SingleUnit(input: Node) -> Result<Quantifier> {
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

    fn RelativeSpecifier(input: Node) -> Result<RelativeSpecifier> {
        Ok(match_nodes!(input.into_children();
            [This(_)] => RelativeSpecifier::This,
            [Next(_)] => RelativeSpecifier::Next,
            [Last(_)] => RelativeSpecifier::Last,
        ))
    }

    fn This(input: Node) -> Result<This> {
        Ok(This {})
    }

    fn Next(input: Node) -> Result<Next> {
        Ok(Next {})
    }

    fn Last(input: Node) -> Result<Last> {
        Ok(Last {})
    }

    fn Num(input: Node) -> Result<i32> {
        input.as_str().parse::<i32>().map_err(|e| input.error(e))
    }

    fn Quantifier(input: Node) -> Result<Quantifier> {
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

    fn TimeUnit(input: Node) -> Result<TimeUnit> {
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

    fn Weekday(input: Node) -> Result<Weekday> {
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

    fn Month(input: Node) -> Result<Month> {
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
enum HumanTime {
    DateTime(DateTime),
    Date(Date),
    Time(Time),
    In(In),
    Ago(Ago),
    Now,
}

#[derive(Debug)]
struct DateTime(Date, Time);

#[derive(Debug)]
struct IsoDate(i32, i32, i32);

#[derive(Debug)]
enum Date {
    Today,
    Tomorrow,
    Overmorrow,
    Yesterday,
    IsoDate(IsoDate),
    DayMonthYear(i32, Month, i32),
    DayMonth(i32, Month),
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
enum Time {
    HourMinute(i32, i32),
    HourMinuteSecond(i32, i32, i32),
}

#[derive(Debug)]
struct In(Duration);

#[derive(Debug)]
enum Ago {
    AgoFromNow(Duration),
    AgoFromTime(Duration, Box<HumanTime>),
}

#[derive(Debug, PartialEq)]
struct Duration(Vec<Quantifier>);

#[derive(Debug)]
struct Now;

#[derive(Debug)]
enum RelativeSpecifier {
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
enum Quantifier {
    Year(i32),
    Month(i32),
    Week(i32),
    Day(i32),
    Hour(i32),
    Minute(i32),
    Second(i32),
}

#[derive(PartialEq, Eq, Debug)]
enum TimeUnit {
    Year,
    Month,
    Week,
    Day,
    Hour,
    Minute,
    Second,
}

#[derive(Debug)]
enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

#[derive(Debug)]
enum Month {
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
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
