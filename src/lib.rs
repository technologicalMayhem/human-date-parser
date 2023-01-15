use pest::{
    Parser,
};

extern crate pest;
#[macro_use]
extern crate pest_derive;

#[derive(Parser)]
#[grammar = "date_time.pest"]
struct DateTimeParser;

pub fn parse(str: &str) -> ! {
    let _parsed = match DateTimeParser::parse(Rule::HumanTime, str) {
        Ok(parsed) => parsed,
        Err(_) => {
            panic!("Could not parse!")
        }
    };

    todo!();
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use insta::{assert_snapshot, with_settings};

    use super::*;
    use concat_idents::concat_idents;

    macro_rules! generate_test_cases {
        ( $($section:ident ($( $inner:literal ),*) ),* ) => {

            $(
                generate_case!($section, $( $inner ),* );
            )*
        };
    }

    macro_rules! generate_case {
        ( $section:ident, $( $case:literal ),* ) => {
            $(concat_idents!(test_name = parse_, $section, _, $case, {
                #[test]
                fn test_name() {
                    let input = $case.to_lowercase();
                    let result = DateTimeParser::parse(Rule::HumanTime, &input).unwrap();
                    let output = format_pair(result.clone().next().unwrap(), 0, false);
                    with_settings!({
                        description => format!("Input: {}", &input),
                        omit_expression => true
                    },
                        { assert_snapshot!(output) }
                    )
                }
            });)*
        };
    }

    // Copied and adapted from:
    // https://github.com/pest-parser/site/blob/24bff8bcd0ac92137791da3dc488903d898c98de/src/lib.rs#L110-L134
    fn format_pair(
        pair: pest::iterators::Pair<Rule>,
        indent_level: usize,
        is_newline: bool,
    ) -> String {
        let indent = if is_newline {
            "  ".repeat(indent_level)
        } else {
            "".to_string()
        };

        let children: Vec<_> = pair.clone().into_inner().collect();
        let len = children.len();
        let children: Vec<_> = children
            .into_iter()
            .map(|pair| {
                format_pair(
                    pair,
                    if len > 1 {
                        indent_level + 1
                    } else {
                        indent_level
                    },
                    len > 1,
                )
            })
            .collect();

        let dash = if is_newline { "- " } else { "" };

        let token = match pair.clone().tokens().next().unwrap() {
            pest::Token::Start { rule, pos: _ } => rule,
            pest::Token::End { rule: _, pos: _ } => panic!("We should not be getting a end token."),
        };
        match len {
            0 => format!(
                "{}{}{:?}: {:?}",
                indent,
                dash,
                token,
                pair.as_span().as_str()
            ),
            1 => format!("{}{}{:?} > {}", indent, dash, token, children[0]),
            _ => format!("{}{}{:?}\n{}", indent, dash, token, children.join("\n")),
        }
    }

    generate_test_cases!(
        date_time(
            "Today 18:30",
            "15:20 Friday",
            "2022-11-07 13:25",
            "This Friday 17:00",
            "13:25, Next Tuesday",
            "Last Friday at 19:45"
        ),
        date(
            "Today",
            "Tommorow",
            "Yesterday",
            "2024-03-03",
            "07-11-2014",
            "02 03 2010",
            "1 2 2020",
            "15 Feb 2017",
            "13 November",
            "This Monday",
            "Next Friday",
            "Last Tuesday",
            "Monday"
        ),
        time("0:20", "12:30", "22:25", "15:55:25", "13:22:32", "10:30:22"),
        // after(),
        // from(),
        _in("In 3 days", "In 2 hours", "In 5 minutes and 30 seconds"),
        // before(),
        ago(
            "10 seconds ago",
            "10 hours and 5 minutes ago",
            "2 hours, 32 minutes and 7 seconds ago",
            "1 years, 2 months, 3 weeks, 5 days, 8 hours, 17 minutes and 45 seconds ago",
            "1 year, 1 month, 1 week, 1 day, 1 hour, 1 minute and 1 second ago"
        ),
        single_unit(
            "A year ago",
            "A month ago",
            "A week ago",
            "A day ago",
            "An hour ago",
            "A minute ago",
            "A second ago"
        ),
        now("now")
    );
}
