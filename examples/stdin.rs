use std::io::stdin;

use chrono::Local;

fn main() {
    let mut buffer = String::new();
    let stdin = stdin();

    loop {
        buffer.clear();
        stdin.read_line(&mut buffer).unwrap();
        let time = match human_date_parser::from_human_time(&buffer) {
          Ok(time) => time,
          Err(e) => {
            println!("{e}");
            continue;
          }
        };
        let now = Local::now();
        println!("Time now: {now}");
        println!("Calculated: {time}\n");
    }
}
