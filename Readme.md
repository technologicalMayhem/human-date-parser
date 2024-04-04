# Human Date Parser

Parses strings that express dates in a human way into ones usable by code.

## Usage

Using it is as simple as calling `from_human_time` with a string slice. Like this:

```rust
use human_date_parser::from_human_time;

fn main() {
    let date = from_human_time("Last Friday at 19:45").unwrap();
    println!("{date}");
}
```

You can also use the example to try out a few dates and see what it can and can't parse. Simply run `cargo run --example stdin`.

## Formats

Currently the following kinds of formats are supported:

- Today 18:30
- 2022-11-07 13:25:30
- 15:20 Friday
- This Friday 17:00
- 13:25, Next Tuesday
- Last Friday at 19:45
- In 3 days
- In 2 hours
- 10 hours and 5 minutes ago
- 1 years ago
- A year ago
- A month ago
- A week ago
- A day ago
- An hour ago
- A minute ago
- A second ago
- Now
- Yesterday
- Tomorrow
- Overmorrow

## Issues

If you find issues or opportunities for improvement do let me know by creating a issues on this projects [GitHub](https://github.com/technologicalMayhem/human-date-parser) page.