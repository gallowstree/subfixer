extern crate chrono;
extern crate time;

use itertools::Itertools;
use std::fs::File;
use std::io::{BufRead, BufReader, Lines, Result};
use std::ops::Add;

use chrono::NaiveTime;
use std::env;
use std::io::Write;
use time::Duration;

const TIME_FORMAT: &str = "%H:%M:%S,%3f";

fn main() -> Result<()> {
    let input_file_path = env::args().next().expect("Missing input file");
    let output_file_path = env::args().next().expect("Missing output file");

    let offset_millis = env::args()
        .next()
        .expect("Missing offset millis")
        .parse::<i64>()
        .expect("Invalid offset millis");

    let file = File::open(input_file_path)?;
    let mut output_file = File::create(output_file_path)?;
    let reader = BufReader::new(file);
    let offset_duration = Duration::milliseconds(offset_millis);

    let offset_entries = reader
        .lines()
        .batching(|lines| parse_subtitle(lines))
        .map(|sub_entry| sub_entry.offset_by(offset_duration));

    for e in offset_entries {
        output_file
            .write_all(e.to_srt_format().as_ref())
            .expect("Failed to write")
    }

    Ok(())
}

fn parse_subtitle(lines: &mut Lines<BufReader<File>>) -> Option<Subtitle> {
    match lines.next() {
        Some(Ok(index_line)) => {
            let index = index_line
                .trim()
                .parse::<u32>()
                .expect("malformed file: missing index");
            let time_marks =
                parse_start_end_times(&lines.next()?.expect(
                    format!("malformed file: missing time marks at index {}", index).as_ref(),
                ));
            let subtitle_text = next_text_block(lines);
            Some(Subtitle {
                index,
                start_time: time_marks.0,
                end_time: time_marks.1,
                subtitle_text,
            })
        }
        _ => None,
    }
}

fn parse_start_end_times(line: &str) -> (NaiveTime, NaiveTime) {
    line.split("-->")
        .map(|x| x.trim())
        .map(|time_str| {
            NaiveTime::parse_from_str(time_str, TIME_FORMAT).expect("Invalid time format")
        })
        .tuple_windows::<(_, _)>()
        .next()
        .expect("Malformed time marks")
}

fn next_text_block(lines: &mut Lines<BufReader<File>>) -> Vec<String> {
    let mut text_lines: Vec<String> = Vec::new();

    while let Some(Ok(line)) = lines.next() {
        match line.as_ref() {
            "" => break,
            line => text_lines.push(String::from(line)),
        }
    }

    text_lines
}

#[derive(Debug)]
struct Subtitle {
    index: u32,
    start_time: NaiveTime,
    end_time: NaiveTime,
    subtitle_text: Vec<String>,
}

impl Subtitle {
    fn offset_by(self, duration: Duration) -> Subtitle {
        let new_start = self.start_time.add(duration);
        let new_end = self.end_time.add(duration);

        Subtitle {
            index: self.index,
            start_time: new_start,
            end_time: new_end,
            subtitle_text: self.subtitle_text,
        }
    }

    fn to_srt_format(&self) -> String {
        let start = self.start_time.format(TIME_FORMAT);
        let end = self.end_time.format(TIME_FORMAT);
        let formatted_time_marks = format!("{} --> {}", start, end);
        let lines = self.subtitle_text.join("\n");

        format!("{}\n{}\n{}\n\n", self.index, formatted_time_marks, lines)
    }
}
