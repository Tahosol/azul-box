use std::path::Path;
use std::{error::Error, fs};
use url::form_urlencoded;

use lofty::{
    self,
    config::WriteOptions,
    prelude::*,
    probe::Probe,
    tag::{Tag, TagType},
};
use serde::Deserialize;

use crate::app::cores::files::change_ext;
use crate::app::cores::{string_cleaner, translate::translate};

pub fn lrclib_fetch(musicfile: &Path, lang: &str, keep_lrc: bool) -> Result<(), Box<dyn Error>> {
    let mut tagged_file = Probe::open(musicfile)?.read()?;

    let tag = match tagged_file.primary_tag_mut() {
        Some(primary_tag) => primary_tag,
        None => {
            if let Some(first_tag) = tagged_file.first_tag_mut() {
                first_tag
            } else {
                let tag_type = tagged_file.primary_tag_type();

                log::warn!("No tags found, creating a new tag of type `{tag_type:?}`");
                tagged_file.insert_tag(Tag::new(tag_type));

                tagged_file.primary_tag_mut().ok_or("Fail to open tag")?
            }
        }
    };
    let artist = tag.artist().ok_or("Fail to open tag title")?;
    let title = string_cleaner::clean_title_before_api_call(
        &tag.title().ok_or("Fail to open tag title")?,
        &artist,
    );

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

        if keep_lrc {
            fs::write(change_ext(&musicfile, "lrc"), &lyric_final)?;
            log::info!("Written lrc file from lrclib");
        }

        if !lyric_final.is_empty() {
            if tag.tag_type() == TagType::Id3v2 {
                tag.insert_text(ItemKey::UnsyncLyrics, lyric_final);
            } else {
                tag.insert_text(ItemKey::Lyrics, lyric_final);
            }
            tag.save_to_path(musicfile, WriteOptions::default())?;
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
