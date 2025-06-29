use lofty::config::WriteOptions;
use lofty::picture::{MimeType, Picture, PictureType};
use lofty::prelude::*;
use lofty::probe::Probe;
use lofty::tag::Tag;

use std::error::Error;
use std::path::Path;
use std::time::Duration;
use ureq::Agent;

pub fn musicbrain_work(opt: &Path, similarity_rate: i8) {
    let mut tagged_file = Probe::open(&opt)
        .expect("ERROR: Bad path provided!")
        .read()
        .expect("ERROR: Failed to read file!");

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
    let title = title.to_string().replace(" ", "%20");
    let artist = artist.to_string().replace(" ", "%20");
    let query = format!(
        "https://musicbrainz.org/ws/2/recording?query={}%20AND%20artist:{}&fmt=json",
        title, artist
    );
    println!("{query}");
    let _ = fetch_musicbrainzapi(&query, opt, similarity_rate, tag);
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
    let re = agent
        .get(q)
        .header("User-Agent", "Azulbox (https://github.com/musdx/azul-box)")
        .call()?
        .body_mut()
        .read_json::<ApiResponseMusicBrainz>();
    match re {
        Ok(resp) => {
            if !resp.recordings.is_empty() && (resp.recordings[0].score > similarity_rate) {
                let record = resp.recordings[0].clone();
                println!("{}", record.id);
                println!("{}", record.title);
                tag.set_title(record.title);
                let query_with_id = format!(
                    "https://musicbrainz.org/ws/2/recording/{}?inc=artist-credits+isrcs+releases+release-groups+discids&fmt=json",
                    record.id
                );
                let re_for_id = agent
                    .get(query_with_id)
                    .header("User-Agent", "Azulbox (https://github.com/musdx/azul-box)")
                    .call();
                match re_for_id {
                    Ok(mut re) => {
                        match re.body_mut().read_json::<IDAPI>() {
                            Ok(data) => {
                                if let Some(artists) = data.artist_credit {
                                    println!("{}", artists[0].name);
                                    tag.set_artist(artists[0].name.clone());
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

                                        println!("{release_id}");
                                        let que = format!(
                                            "https://coverartarchive.org/release/{}",
                                            release_id
                                        );
                                        println!("{que}");
                                        let res = agent
                                            .get(que)
                                            .header(
                                                "User-Agent",
                                                "Azulbox (https://github.com/musdx/azul-box)",
                                            )
                                            .call();
                                        match res {
                                            Ok(mut awnser) => {
                                                let succes_re = awnser
                                                    .body_mut()
                                                    .read_json::<ApiResponseCover>();
                                                match succes_re {
                                                    Ok(callfocover) => {
                                                        println!("Cover??");
                                                        if let Some(images) = callfocover.images {
                                                            println!("{}", images[0].image);
                                                            let img_req = agent
                                                                .get(&images[0].image)
                                                                .header(
                                                                    "User-Agent",
                                                                    "Azulbox (https://github.com/musdx/azul-box)",
                                                                )
                                                                .call()
                                                                .expect("did load pic");
                                                            let data: Vec<u8> = img_req
                                                                .into_body()
                                                                .read_to_vec()?;

                                                            let picture = Picture::new_unchecked(
                                                                PictureType::CoverFront,
                                                                Some(MimeType::Jpeg),
                                                                None,
                                                                data,
                                                            );
                                                            println!("Cover mostly work");
                                                            if tag.picture_count() > 0 {
                                                                tag.remove_picture(0);
                                                            }
                                                            tag.push_picture(picture);
                                                        };
                                                    }
                                                    Err(e) => {
                                                        println!("{e}");
                                                        println!("Cover fail")
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                println!("{e}");
                                                println!("request cover fail");
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                println!("Fail to read data from json with id code: {e}")
                            }
                        };
                    }
                    Err(e) => {
                        println!("Fail to request with ID of record code: {e}");
                    }
                }
            }
        }
        Err(e) => {
            println!("{e:?}");
        }
    };
    println!("Work");
    tag.save_to_path(opt, WriteOptions::default())
        .expect("ERROR: Failed to write the tag!");
    Ok(())
}
use serde::Deserialize;
#[derive(Debug, Deserialize)]
struct IDAPI {
    #[serde(rename = "artist-credit")]
    artist_credit: Option<Vec<ArtistCredit>>,
    releases: Option<Vec<Release>>,
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
