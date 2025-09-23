use std::{env::args, fs::read_to_string};

use chrono::Local;
use human_date_parser::ParseResult;

fn main() {
    let path = args().skip(1).next().expect("No file given");
    let lines: Vec<String> = read_to_string(path)
        .unwrap()
        .split("\n")
        .filter(filter_string)
        .map(remove_comments)
        .collect();
    for line in lines {
        println!("Input: {line}");
        let now = Local::now().naive_local();
        let result = match human_date_parser::from_human_time(&line, now) {
            Ok(time) => time,
            Err(e) => {
                println!("{e}");
                continue;
            }
        };

        let now = Local::now();

        match result {
            ParseResult::DateTime(datetime) => {
                println!("Time now: {now}");
                println!("Time then: {datetime}\n");
            }
            ParseResult::Date(date) => println!("Date: {date}\n"),
            ParseResult::Time(time) => println!("Time: {time}\n"),
            ParseResult::DateTimeTz(date_time) => println!("Time: {date_time}"),
        };
    }
}

fn filter_string(str: &&str) -> bool {
    !str.is_empty() && !str.starts_with("#")
}

fn remove_comments(str: &str) -> String {
    str.split("#").next().unwrap_or_default().to_string()
}