use std::io::stdin;

use chrono::{Local, NaiveDateTime};
use human_date_parser::ParseResult;

fn main() {
    let mut buffer = String::new();
    let stdin = stdin();

    loop {
        buffer.clear();
        stdin.read_line(&mut buffer).unwrap();
        let result = match human_date_parser::from_human_time(&buffer) {
            Ok(time) => time,
            Err(e) => {
                println!("{e}");
                continue;
            }
        };

        let now = Local::now();

        let result = match result {
            ParseResult::DateTime(datetime) => datetime,
            ParseResult::Date(date) => NaiveDateTime::new(date, now.time())
                .and_local_timezone(Local)
                .unwrap(),
            ParseResult::Time(time) => NaiveDateTime::new(now.date_naive(), time)
                .and_local_timezone(Local)
                .unwrap(),
        };

        println!("Time now: {now}");
        println!("Calculated: {result}\n");
    }
}
