use crate::app::cores::{string_cleaner, translate::translate};
use base64::prelude::*;
use lofty::{self, config::WriteOptions, prelude::*, probe::Probe, tag::Tag};
use serde::Deserialize;
use std::error::Error;
use std::path::Path;

pub fn get(path: &Path, lang: &str) -> Result<(), Box<dyn Error>> {
    let mut tagged_file = Probe::open(path)?.read()?;

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
    let title = string_cleaner::clean_title_before_api_call(&tag.title().unwrap(), &artist);

    let data = kugou_search(&title)?;

    if let Some(info) = data.data.info {
        if let Some(best_match) = info.first() {
            let lyrics = kugou_get_lyrics(&best_match.hash, lang)?;
            if !lyrics.is_empty() {
                tag.insert_text(ItemKey::Lyrics, lyrics);
                tag.save_to_path(path, WriteOptions::default())?;
            }
        }
    }
    Ok(())
}

fn kugou_get_lyrics(songhash: &str, lang: &str) -> Result<String, Box<dyn Error>> {
    let request = ureq::get(format!(
        "https://krcs.kugou.com/search?ver=1&man=yes&client=mobi&hash={songhash}"
    ))
    .call()?
    .body_mut()
    .read_json::<KugouGetData>()?;
    if let Some(candidates) = request.candidates {
        if let Some(first_candidate) = candidates.first() {
            let request_lyrics = ureq::get(format!("https://krcs.kugou.com/download?ver=1&man=yes&client=pc&fmt=lrc&id={}&accesskey={}", first_candidate.id, first_candidate.accesskey))
                .call()?
                .body_mut()
                .read_json::<Lyrics>()?;
            let u8_lyric = BASE64_STANDARD.decode(request_lyrics.content)?;
            let str_lyric = str::from_utf8(&u8_lyric)?;
            let translated_lyrics = translate(lang, str_lyric)?;
            return Ok(translated_lyrics);
        }
    }
    Err("Nothing exist".into())
}

#[derive(Deserialize)]
struct KugouGetData {
    candidates: Option<Vec<Candidate>>,
}

#[derive(Deserialize)]
struct Candidate {
    id: String,
    accesskey: String,
}

#[derive(Deserialize)]
struct Lyrics {
    content: String,
}

fn kugou_search(music_name: &str) -> Result<KuGouSearchAPI, Box<dyn Error>> {
    let encoded_music_name: String =
        url::form_urlencoded::byte_serialize(music_name.as_bytes()).collect();
    let request = ureq::get(format!(
        "http://msearchcdn.kugou.com/api/v3/search/song?&plat=0&keyword={encoded_music_name}&version=9108"
    ))
    .call()?
    .body_mut()
    .read_json::<KuGouSearchAPI>()?;
    Ok(request)
}

#[derive(Deserialize, Clone)]
struct KuGouSearchAPI {
    data: SearchData,
}

#[derive(Deserialize, Clone)]
struct SearchData {
    info: Option<Vec<DataInfo>>,
}

#[derive(Deserialize, Clone)]
struct DataInfo {
    hash: String,
}
