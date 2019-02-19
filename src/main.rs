use std::fs::File;
use std::time::Duration;
use std::io::{BufRead, BufReader, Result};

fn main() -> Result<()> {
    let file = File::open("sample.srt")?;
    let reader = BufReader::new(file);

    //TODO: Use https://docs.rs/itertools/0.6.5/itertools/trait.Itertools.html#method.batching
    let mut lines = reader.lines().map(|l| l.unwrap());

    let index = loop {
        match lines.next().unwrap().trim().parse::<u32>() {
            Ok(idx) => break idx,
            _ => ()
        }
    };

    let time_marks = lines.next().unwrap();
    let time_marks: Vec<&str> = time_marks.split("-->").map(|x| x.trim()).collect();
    //TODO: Parse times using  NaiveTime::parse_from_str("23:56:04", "%H:%M:%S%,f")

    let mut text_lines: Vec<String> = Vec::new();

    while let Some(line) = lines.next() {
        match line.as_ref() {
            "" => break,
            line => text_lines.push(String::from(line))
        }
    }

    println!("{:?}", text_lines);

    Ok(())
}


struct Entry {
    index: u32,
    start_time: Duration,
    end_time: Duration,
    text_lines: Vec<String>
}