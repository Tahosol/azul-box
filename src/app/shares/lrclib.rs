use std::error::Error;
use std::path::Path;

use lofty::config::WriteOptions;
use lofty::prelude::*;
use lofty::probe::Probe;
use lofty::tag::Tag;
use serde::Deserialize;

pub fn lrclib_fetch(opt: &Path, lang: &str) -> Result<(), Box<dyn Error>> {
    let mut tagged_file = Probe::open(&opt)?.read()?;

    let tag = match tagged_file.primary_tag_mut() {
        Some(primary_tag) => primary_tag,
        None => {
            if let Some(first_tag) = tagged_file.first_tag_mut() {
                first_tag
            } else {
                let tag_type = tagged_file.primary_tag_type();

                eprintln!("WARN: No tags found, creating a new tag of type `{tag_type:?}`");
                tagged_file.insert_tag(Tag::new(tag_type));

                tagged_file.primary_tag_mut().unwrap()
            }
        }
    };
    let artist = tag.artist().unwrap();
    let title = tag.title().unwrap();
    let artist: String = form_urlencoded::byte_serialize(artist.as_bytes()).collect();
    let title: String = form_urlencoded::byte_serialize(title.as_bytes()).collect();
    let query = format!(
        "https://lrclib.net/api/get?artist_name={}&track_name={}",
        artist, title
    );
    let ly = fetch(&query)?;
    println!("Query for lrclib: {}", query);
    if !ly.is_empty() {
        println!("Lyrics Found From lrclib");
        let lyric_final = translate(lang, &ly)?;
        if !lyric_final.is_empty() {
            tag.insert_text(ItemKey::Lyrics, lyric_final);
            tag.save_to_path(opt, WriteOptions::default())?;
        }
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    #[serde(rename = "plainLyrics")]
    plain_lyrics: String,
    #[serde(rename = "syncedLyrics")]
    synced_lyrics: String,
}

fn fetch(query: &str) -> Result<String, Box<dyn Error>> {
    let lyr = ureq::get(query)
        .header(
            "User-Agent",
            "Azulbox (https://github.com/tahosol/azul-box)",
        )
        .call()?
        .body_mut()
        .read_json::<ApiResponse>()?;
    if !lyr.synced_lyrics.is_empty() {
        Ok(lyr.synced_lyrics)
    } else if !lyr.plain_lyrics.is_empty() {
        Ok(lyr.plain_lyrics)
    } else {
        Ok(String::new())
    }
}

use serde_json::Value;
use url::form_urlencoded;

fn translate(to: &str, text: &str) -> Result<String, Box<dyn std::error::Error>> {
    let en_text: String = form_urlencoded::byte_serialize(text.as_bytes()).collect();
    let url = format!(
        "https://translate.googleapis.com/translate_a/single?client=gtx&sl=auto&tl={}&dt=t&q={}",
        to, en_text
    );

    let mut translated_text = text.to_string();

    let json_as_string = ureq::get(&url).call()?.body_mut().read_to_string()?;
    let values = serde_json::from_str::<Value>(&json_as_string)?;
    if let Some(value) = values.get(0) {
        println!("{:?}", value.as_str());
        if let Some(list) = value.as_array() {
            let lyrics: Vec<String> = list
                .iter()
                .filter_map(|v| v.get(0).and_then(|v| v.as_str()))
                .map(|s| s.to_string())
                .collect();
            translated_text = lyrics.join("");
            println!("Translate success!");
        }
    }
    Ok(translated_text)
}
