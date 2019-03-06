extern crate chrono;

use itertools::Itertools;
use std::fs::File;
use time::Duration;
use std::ops::Add;

use std::io::{BufRead, BufReader, Result, Lines};
use chrono::{NaiveTime, Timelike};
use chrono::format::{Parsed, parse};
use chrono::format::StrftimeItems;

fn main() -> Result<()> {
    let file = File::open("sample.srt")?;
    let reader = BufReader::new(file);
    let offset_by = Duration {secs: 1, nanos: 1};

    let subtitle_entries = reader.lines()
        .batching(|lines| {

            let index = extract_index(&lines.next().unwrap().unwrap());
            let time_marks = extract_start_end_times(&lines.next().unwrap().unwrap());
            let subtitle_text_lines = extract_text(lines);

            Some(Entry {index, start_time: time_marks.0, end_time: time_marks.1, subtitle_text_lines}) })
        .map(|entry| entry.offset_by(offset_by))
        .for_each(|e| println!("{:?}", e));



    Ok(())
}

fn extract_index(line: &str) -> u32 {
    line.trim().parse::<u32>().expect("malformed file: missing index")
}

fn extract_start_end_times(line: &str) -> (Duration, Duration) {
    line.split("-->")
        .map(|x| x.trim())
        .map(|time_str| {
            let mut parsed = Parsed::new();
            parse(&mut parsed, time, StrftimeItems::new("%H:%M:%S,%f"));
            parsed
        })
        .map(|parsed_time| Duration {secs: time.hour(), nanos: time.})
        .tuple_windows::<(_, _)>()
        .next().unwrap()
}

fn extract_text(lines: &mut Lines<BufReader<File>>) -> Vec<String> {
    let mut text_lines: Vec<String> = Vec::new();

    while let Ok(line) = lines.next().unwrap() {
        match line.as_ref() {
            "" => break,
            line => text_lines.push(String::from(line))
        }
    };

    text_lines
}

#[derive(Debug)]
struct Entry {
    index: u32,
    start_time: Duration,
    end_time: Duration,
    subtitle_text_lines: Vec<String>
}

impl Entry {
    fn offset_by(self, duration: Duration) -> Entry {
        let new_start = self.start_time.add(duration);
        let new_end = self.end_time.add(duration);
        Entry {index: self.index, start_time: new_start, end_time: new_end, subtitle_text_lines: self.subtitle_text_lines}
    }
}