use std::fs;
use std::path::{Path, PathBuf};

pub fn work(filename: &str, music_file: &str, format_name: &str, directory: &str) {
    let lyrics_file = finder_lyrics(&directory, &filename).unwrap();
    let music_file = Path::new(&music_file);
    let lyrics = match fs::read_to_string(&lyrics_file) {
        Ok(file) => file,
        Err(error) => {
            println!("{:?}", error);
            String::new()
        }
    };
    let valid_format = &["flac", "opus", "mp3", "m4a"];
    if !lyrics.is_empty() && valid_format.contains(&format_name) {
        use lofty::config::WriteOptions;
        use lofty::prelude::*;
        use lofty::probe::Probe;
        use lofty::tag::Tag;

        let mut tagged_file = Probe::open(&music_file)
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
        tag.insert_text(ItemKey::Lyrics, lyrics);
        tag.save_to_path(&music_file, WriteOptions::default())
            .expect("ERROR: Failed to write the tag!");

        println!("Lyrics from  Youtube Inserted Sucessfully");
        let _ = fs::remove_file(&lyrics_file);
    }
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
