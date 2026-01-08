use crate::app::cores::files::file_finder;
use std::error::Error;
use std::fs;
use std::path::Path;

static VALID_FORMAT: &[&str] = &["flac", "opus", "mp3", "m4a"];

pub fn work(
    filename: &str,
    music_file: &Path,
    format_name: &str,
    directory: &str,
    sanitize: bool,
) -> Result<(), Box<dyn Error>> {
    let lyrics_file = match file_finder(directory, filename, &["lrc"]) {
        Some(path) => path,
        None => {
            return Err("Lyrics file not found.".into());
        }
    };
    let lyrics = if sanitize {
        lyrics_cleaner(&fs::read_to_string(&lyrics_file)?)?
    } else {
        fs::read_to_string(&lyrics_file)?
    };
    if !lyrics.is_empty() && VALID_FORMAT.contains(&format_name) {
        use lofty::config::WriteOptions;
        use lofty::prelude::*;
        use lofty::probe::Probe;
        use lofty::tag::Tag;

        let mut tagged_file = Probe::open(music_file)?.read()?;

        let tag = match tagged_file.primary_tag_mut() {
            Some(primary_tag) => primary_tag,
            None => {
                if let Some(first_tag) = tagged_file.first_tag_mut() {
                    first_tag
                } else {
                    let tag_type = tagged_file.primary_tag_type();

                    tagged_file.insert_tag(Tag::new(tag_type));

                    tagged_file.primary_tag_mut().unwrap()
                }
            }
        };
        tag.insert_text(ItemKey::Lyrics, lyrics);
        tag.save_to_path(music_file, WriteOptions::default())?;

        fs::remove_file(&lyrics_file)?;
    }
    Ok(())
}

use std::fmt;

use regex::Regex;

fn lyrics_cleaner(lyrics: &str) -> Result<String, Box<dyn Error>> {
    let lines: Vec<&str> = lyrics.lines().filter(|line| !line.is_empty()).collect();
    let time_stamps = collect_time_stamp(lyrics)?;

    let lrclines = turn_into_lrcline(time_stamps, lines)?;
    let mut cleaned_lrc: Vec<&LrcLine> = vec![];

    let mut counter = 0;
    while counter < lrclines.len() {
        if counter == 0 {
            cleaned_lrc.push(&lrclines[counter]);
        };
        let last_cleaned = cleaned_lrc
            .last()
            .ok_or("cleaned_lrc: fail take the last cleaned_lrc")?;
        match last_cleaned.is_similar(&lrclines[counter])? {
            Similarity::Time => {
                let merged = Box::new(last_cleaned.merge(&lrclines[counter]));
                cleaned_lrc.pop();
                cleaned_lrc.push(Box::leak(merged));
            }
            Similarity::AinB => {
                let merged = Box::new(last_cleaned.merge(&lrclines[counter]));
                cleaned_lrc.pop();
                cleaned_lrc.push(Box::leak(merged));
            }
            Similarity::None => {
                cleaned_lrc.push(&lrclines[counter]);
            }
            _ => {}
        }
        counter += 1;
    }
    let mut cleaned_lrc_strings: Vec<String> = vec![];
    for lrc in cleaned_lrc {
        cleaned_lrc_strings.push(lrc.clone().clean().return_lrc());
    }
    Ok(cleaned_lrc_strings.join("\n"))
}

fn turn_into_lrcline(times: Vec<String>, lines: Vec<&str>) -> Result<Vec<LrcLine>, Box<dyn Error>> {
    let mut counter = 0;
    let mut without: Vec<LrcLine> = vec![];
    while counter < times.len() {
        let time = times[counter].trim_matches(|c| c == '[' || c == ']');
        let mut time_parts = time.split(':');
        let min = time_parts
            .next()
            .ok_or("turn_into_lrcline: fail to parse min")?
            .parse::<u8>()?;
        let sec = time_parts
            .next()
            .ok_or("turn_into_lrcline: fail to parse sec")?
            .parse::<f32>()?;
        let lrc = LrcLine {
            timestamp: TimeStamp {
                minutes: min,
                seconds: sec,
            },
            content: lines[counter + 2]
                .replace(&times[counter], "")
                .trim()
                .to_string(),
        };
        without.push(lrc);
        counter += 1
    }
    Ok(without)
}
enum Similarity {
    Content,
    AinB,
    BinA,
    Time,
    None,
}

#[derive(Debug, Clone)]
struct LrcLine {
    timestamp: TimeStamp,
    content: String,
}
impl LrcLine {
    fn return_lrc(&self) -> String {
        format!(
            "[{:02}:{:05.2}]{}",
            self.timestamp.minutes, self.timestamp.seconds, self.content
        )
    }
    fn clean(&mut self) -> LrcLine {
        let regex_patterns = [r"\\[A-Za-z]", r"</[A-Za-z]>", r"<[A-Za-z]>"];
        let mut cleaned_content = self.content.clone();
        for pattern in regex_patterns.iter() {
            let regex = regex::Regex::new(pattern).unwrap();
            cleaned_content = regex.replace_all(&cleaned_content, "").to_string();
        }
        self.content = cleaned_content;
        self.clone()
    }
    fn is_similar(&self, compared: &LrcLine) -> Result<Similarity, Box<dyn Error>> {
        let compared_in_self = self.content.contains(compared.content.trim());
        let self_in_compared = compared.content.contains(self.content.trim());
        let minustime = self.timestamp.minus(&compared.timestamp)?;

        let strict_similar_content =
            textdistance::nstr::cosine(&self.content, &compared.content) > 0.9;
        let content_check = textdistance::nstr::cosine(&self.content, &compared.content) > 0.85;

        let time_status =
            minustime.minutes == 0 && -0.5 <= minustime.seconds && minustime.seconds <= 0.0;
        let time_status_less_strict =
            minustime.minutes == 0 && -2.0 < minustime.seconds && minustime.seconds <= 0.0;
        // println!("{minustime}: {time_status}");

        if (content_check && time_status) || (strict_similar_content && time_status_less_strict) {
            Ok(Similarity::Content)
        } else if compared_in_self && time_status {
            Ok(Similarity::BinA)
        } else if self_in_compared && time_status {
            Ok(Similarity::AinB)
        } else if time_status {
            Ok(Similarity::Time)
        } else {
            Ok(Similarity::None)
        }
    }
    fn merge(&self, merge_item: &LrcLine) -> LrcLine {
        LrcLine {
            timestamp: self.timestamp.clone(),
            content: format!("{} {}", self.content, merge_item.content),
        }
    }
}

impl fmt::Display for LrcLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TimeStamp: {} Content: {}", self.timestamp, self.content)
    }
}
#[derive(Debug, Clone)]
struct TimeStamp {
    minutes: u8,
    seconds: f32,
}
impl TimeStamp {
    fn minus(&self, b: &TimeStamp) -> Result<TimeStamp, Box<dyn Error>> {
        let (minutes, seconds) = (self.minutes - b.minutes, self.seconds - b.seconds);
        Ok(TimeStamp { minutes, seconds })
    }
}
impl fmt::Display for TimeStamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{:02}:{:05.2}]", self.minutes, self.seconds)
    }
}

fn collect_time_stamp(lyrics: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let time_stamp_regex = Regex::new(r"\[[0-9]+:[0-9]+\.[0-9]+\]").unwrap();
    Ok(time_stamp_regex
        .find_iter(lyrics)
        .map(|time| time.as_str().to_string())
        .collect())
}
