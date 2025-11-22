use lofty::config::WriteOptions;
use lofty::picture::{MimeType, Picture, PictureType};
use lofty::prelude::*;
use lofty::probe::Probe;
use lofty::tag::Tag;

use std::error::Error;
use std::path::Path;
use std::time::Duration;
use ureq::Agent;

use crate::app::shares::string_cleaner;

pub fn work(opt: &Path, similarity_rate: i8) -> Result<(), Box<dyn Error>> {
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
    use url::form_urlencoded;

    let artist = tag.artist().unwrap();
    let title = string_cleaner::clean_title_before_api_call(&tag.title().unwrap(), &artist);

    let artist: String = form_urlencoded::byte_serialize(artist.as_bytes()).collect();
    let title: String = form_urlencoded::byte_serialize(title.as_bytes()).collect();
    let query = format!(
        "https://musicbrainz.org/ws/2/recording?query={}%20AND%20artist:{}&fmt=json",
        title, artist
    );
    println!("musicbrain_work query: {query}");
    let _ = fetch_musicbrainzapi(&query, opt, similarity_rate, tag);
    Ok(())
}
fn fetch_musicbrainzapi(
    q: &str,
    opt: &Path,
    similarity_rate: i8,
    tag: &mut Tag,
) -> Result<(), Box<dyn Error>> {
    let config = Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(5)))
        .build();

    let agent: Agent = config.into();
    let resp = agent
        .get(q)
        .header(
            "User-Agent",
            "Azulbox (https://github.com/tahosol/azul-box)",
        )
        .call()?
        .body_mut()
        .read_json::<ApiResponseMusicBrainz>()?;
    if !resp.recordings.is_empty() && (resp.recordings[0].score > similarity_rate) {
        let record = resp.recordings[0].clone();
        println!("Record ID: {}", record.id);
        println!("Record Title: {}", record.title);
        tag.set_title(record.title);
        let query_with_id = format!(
            "https://musicbrainz.org/ws/2/recording/{}?inc=artist-credits+isrcs+releases+release-groups+discids&fmt=json",
            record.id
        );
        println!("Query_with_id: {query_with_id}");
        let mut re_for_id = agent
            .get(query_with_id)
            .header(
                "User-Agent",
                "Azulbox (https://github.com/tahosol/azul-box)",
            )
            .call()?;
        let data = re_for_id.body_mut().read_json::<IDAPI>()?;
        if let Some(artists) = data.artist_credit {
            println!("Artist: {}", artists[0].name);
            tag.set_artist(artists[0].name.clone());
        }
        if let Some(isrcs) = data.isrcs {
            if !isrcs.is_empty() {
                println!("ISRCS: {}", isrcs[0]);
                tag.insert_text(ItemKey::Isrc, isrcs[0].clone());
            }
        }
        if let Some(releases) = data.releases {
            if !releases.is_empty() {
                let release_id = &releases[0].id;
                if let Some(date) = &releases[0].date {
                    let years = &date.split("-").next().unwrap();
                    let year: u32 = years.parse::<u32>().unwrap();
                    tag.set_year(year);
                    tag.insert_text(ItemKey::ReleaseDate, date.clone());
                }
                tag.set_album(releases[0].title.clone());
                if let Some(media) = &releases[0].media {
                    tag.set_disk(media[0].position);
                    tag.set_track(media[0].position);
                    tag.set_track_total(media[0].track_count);
                    tag.set_disk_total(media[0].track_count);
                }

                println!("Release ID: {release_id}");
                if tag.save_to_path(opt, WriteOptions::default()).is_ok() {
                    println!("Musicbrainz Metadata Embedded Sucsess");
                } else {
                    eprintln!("Fail To Embedd Metadata From MusicBrainz")
                }
                let que = format!("https://coverartarchive.org/release/{}", release_id);
                println!("Cover Art Link: {que}");
                let mut res = agent
                    .get(que)
                    .header(
                        "User-Agent",
                        "Azulbox (https://github.com/tahosol/azul-box)",
                    )
                    .call()?;
                let callfocover = res.body_mut().read_json::<ApiResponseCover>()?;
                if let Some(images) = callfocover.images {
                    println!("{}", images[0].image);
                    let img_req = agent
                        .get(&images[0].image)
                        .header(
                            "User-Agent",
                            "Azulbox (https://github.com/tahosol/azul-box)",
                        )
                        .call()?;
                    let data: Vec<u8> = img_req.into_body().read_to_vec()?;

                    let picture = Picture::new_unchecked(
                        PictureType::CoverFront,
                        Some(MimeType::Jpeg),
                        None,
                        data,
                    );
                    println!("Cover Image Found!");
                    if tag.picture_count() > 0 {
                        tag.remove_picture(0);
                    }
                    tag.push_picture(picture);
                    if tag.save_to_path(opt, WriteOptions::default()).is_ok() {
                        println!("Musicbrainz Cover Embedded Sucsess");
                    } else {
                        eprintln!("Fail To Embedd Cover From MusicBrainz")
                    }
                } else {
                    eprintln!("Fail To Find Cover Art");
                }
            } else {
                eprintln!("Fail To Find Releases Data");
            }
        }
    } else {
        eprintln!("Fail To Find Musicbrainz Data");
    }
    Ok(())
}
use serde::Deserialize;
#[derive(Debug, Deserialize)]
struct IDAPI {
    #[serde(rename = "artist-credit")]
    artist_credit: Option<Vec<ArtistCredit>>,
    releases: Option<Vec<Release>>,
    isrcs: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct ApiResponseCover {
    images: Option<Vec<Image>>,
}
#[derive(Debug, Deserialize)]
struct Image {
    image: String,
}

#[derive(Debug, Deserialize)]
struct ApiResponseMusicBrainz {
    recordings: Vec<Recording>,
}
#[derive(Debug, Deserialize, Clone)]
struct Recording {
    id: String,
    score: i8,
    title: String,
}
#[derive(Debug, Deserialize, Clone)]
struct ArtistCredit {
    name: String,
}
#[derive(Debug, Deserialize, Clone)]
struct Release {
    id: String,
    title: String,
    media: Option<Vec<Media>>,
    date: Option<String>,
}
#[derive(Debug, Deserialize, Clone)]
struct Media {
    position: u32,
    #[serde(rename = "track-count")]
    track_count: u32,
}
