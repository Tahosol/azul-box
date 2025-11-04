use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

static VALID_FORMAT: &[&str] = &["flac", "opus", "mp3", "m4a"];

pub fn work(
    filename: &str,
    music_file: &str,
    format_name: &str,
    directory: &str,
) -> Result<(), Box<dyn Error>> {
    let lyrics_file = match finder_lyrics(&directory, &filename) {
        Some(path) => path,
        None => {
            return Err("Lyrics file not found.".into());
        }
    };
    let music_file = Path::new(&music_file);
    let lyrics = fs::read_to_string(&lyrics_file)?;
    if !lyrics.is_empty() && VALID_FORMAT.contains(&format_name) {
        use lofty::config::WriteOptions;
        use lofty::prelude::*;
        use lofty::probe::Probe;
        use lofty::tag::Tag;

        let mut tagged_file = Probe::open(&music_file)?.read()?;

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
        tag.save_to_path(&music_file, WriteOptions::default())?;

        fs::remove_file(&lyrics_file)?;
    }
    Ok(())
}

fn finder_lyrics(directory: &str, filename: &str) -> Option<PathBuf> {
    let elements = fs::read_dir(&directory).ok()?;
    let mut thing = Some(PathBuf::new());

    for item in elements {
        let path = item.ok()?.path();
        if path.is_file() {
            if path.extension().and_then(|ext| ext.to_str()) == Some("lrc") {
                if let Some(file) = path.file_name().and_then(|name| name.to_str()) {
                    if file.contains(filename) {
                        thing = Some(path);
                    }
                }
            }
        }
    }
    thing
}
