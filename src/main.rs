extern crate time;
extern crate chrono;

use itertools::Itertools;
use std::fs::File;
use std::ops::Add;
use std::io::{BufRead, BufReader, Result, Lines};

use chrono::{Timelike};
use chrono::NaiveTime;
use time::Duration;

fn main() -> Result<()> {
    let file = File::open("sample.srt")?;
    let reader = BufReader::new(file);
    let offset_duration = Duration::milliseconds(1001);

    let subtitle_entries = reader.lines()
        .batching(|lines| consume_parsing_entries(lines))
        .map(|sub_entry| sub_entry.offset_by(offset_duration))
        .for_each(|e| println!("{:?}", e));


    Ok(())
}

fn consume_parsing_entries(lines: &mut Lines<BufReader<File>>) -> Option<Subtitle> {
    match lines.next() {
        Some(Ok(index_line)) => {
            let index = extract_index(&index_line);
            let time_marks = extract_start_end_times(&lines.next()?.unwrap());
            let subtitle_text_lines = extract_text(lines);
            Some(Subtitle {index, start_time: time_marks.0, end_time: time_marks.1, subtitle_text_lines})
        },
        _ => None
    }
}

fn extract_index(line: &str) -> u32 {
    line.trim().parse::<u32>().expect("malformed file: missing index")
}

fn extract_start_end_times(line: &str) -> (Duration, Duration) {
    line.split("-->")
        .map(|x| x.trim())
        .map(|time_str| NaiveTime::parse_from_str(time_str, "%H:%M:%S,%f").unwrap())
        .map(|parsed_time| {
            let secs = parsed_time.hour() * 3600 + parsed_time.minute() * 60 + parsed_time.second();
            let nanos = parsed_time.nanosecond() as i64;
            Duration::seconds(secs as i64).add(Duration::nanoseconds(nanos))
        })
        .tuple_windows::<(_, _)>()
        .next().unwrap()
}

fn extract_text(lines: &mut Lines<BufReader<File>>) -> Vec<String> {
    let mut text_lines: Vec<String> = Vec::new();

    while let Some(Ok(line)) = lines.next() {
        match line.as_ref() {
            "" => break,
            line => text_lines.push(String::from(line))
        }
    };

    text_lines
}

#[derive(Debug)]
struct Subtitle {
    index: u32,
    start_time: Duration,
    end_time: Duration,
    subtitle_text_lines: Vec<String>
}

impl Subtitle {
    fn offset_by(self, duration: Duration) -> Subtitle {
        let new_start = self.start_time.add(duration);
        let new_end = self.end_time.add(duration);
        Subtitle {index: self.index, start_time: new_start, end_time: new_end, subtitle_text_lines: self.subtitle_text_lines}
    }
}