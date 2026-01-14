use std::error::Error;
use std::path::Path;
use url::form_urlencoded;

use lofty::{self, config::WriteOptions, prelude::*, probe::Probe, tag::Tag};
use serde::Deserialize;

use crate::app::cores::{string_cleaner, translate::translate};

pub fn lrclib_fetch(opt: &Path, lang: &str) -> Result<(), Box<dyn Error>> {
    let mut tagged_file = Probe::open(opt)?.read()?;

    let tag = match tagged_file.primary_tag_mut() {
        Some(primary_tag) => primary_tag,
        None => {
            if let Some(first_tag) = tagged_file.first_tag_mut() {
                first_tag
            } else {
                let tag_type = tagged_file.primary_tag_type();

                log::warn!("No tags found, creating a new tag of type `{tag_type:?}`");
                tagged_file.insert_tag(Tag::new(tag_type));

                tagged_file.primary_tag_mut().unwrap()
            }
        }
    };
    let artist = tag.artist().unwrap();
    let title = string_cleaner::clean_title_before_api_call(&tag.title().unwrap(), &artist);

    let artist: String = form_urlencoded::byte_serialize(artist.as_bytes()).collect();
    let title: String = form_urlencoded::byte_serialize(title.as_bytes()).collect();
    let query = format!(
        "https://lrclib.net/api/get?artist_name={}&track_name={}",
        artist, title
    );
    let ly = fetch(&query)?;
    log::info!("Query for lrclib: {}", query);
    if !ly.is_empty() {
        log::info!("Lyrics Found From lrclib");
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
