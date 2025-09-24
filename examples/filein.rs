use std::{env::args, fs::read_to_string};

use chrono::Local;
use human_date_parser::ParseResult;

const RED: &str = "\x1B[31m";
const GREEN: &str = "\x1B[32m";
const RESET: &str = "\x1B[0m";

fn main() {
    let now = Local::now().naive_local();
    println!("Now: {now}\n");

    let path = args().skip(1).next().expect("No file given");
    let lines: Vec<String> = read_to_string(path)
        .unwrap()
        .split("\n")
        .filter(filter_string)
        .map(remove_comments)
        .collect();
    for line in lines {
        print!("{line}");
        let result = match human_date_parser::from_human_time(&line, now) {
            Ok(time) => time,
            Err(e) => {
                println!(" -> {RED}{e}{RESET}");
                continue;
            }
        };

        print!(" -> {GREEN}");
        match result {
            ParseResult::DateTime(datetime) => print!("Datetime: {datetime}"),
            ParseResult::Date(date) => print!("Date: {date}"),
            ParseResult::Time(time) => print!("Time: {time}"),
            ParseResult::DateTimeTz(date_time) => print!("Datetime: {date_time}"),
        };
        println!("{RESET}");
    }
}

fn filter_string(str: &&str) -> bool {
    !str.is_empty() && !str.starts_with("#")
}

fn remove_comments(str: &str) -> String {
    str.split("#").next().unwrap_or_default().to_string()
}
