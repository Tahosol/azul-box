use crate::app::cores::depen_manager::Depen;
use crate::app::cores::lrclib::lrclib_fetch;
use crate::app::cores::{kugou, musicbrainz};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn version_check(depen: &Depen) -> Option<String> {
    match Command::new(&depen.yt_dlp).arg("--version").output() {
        Ok(out) => String::from_utf8(out.stdout).ok(),
        Err(_) => None,
    }
}

pub fn video_download(
    link: String,
    directory: String,
    format: i8,
    frag: i8,
    sub: bool,
    lang: &str,
    auto_gen: bool,
    cookies: Option<String>,
    use_cookies: bool,
    res: i32,
    yt_dlp: PathBuf,
) -> i8 {
    let n = frag.to_string().to_owned();

    let mut yt = Command::new(yt_dlp);
    if let Some(cookie) = cookies
        && use_cookies
    {
        yt.arg("--cookies").arg(cookie);
    }

    yt.arg("--concurrent-fragments")
        .arg(n)
        .arg("--embed-thumbnail")
        .arg("--embed-metadata")
        .arg("--add-metadata")
        .arg("--metadata-from-title")
        .arg("%(title)s")
        .arg("--parse-metadata")
        .arg("title:%(title)s")
        .arg("--parse-metadata")
        .arg("uploader:%(artist)s")
        .arg("--output")
        .arg("%(title)s.%(ext)s")
        .arg("--compat-options")
        .arg("no-live-chat")
        .current_dir(directory);
    if sub && auto_gen {
        yt.arg("--write-auto-subs")
            .arg("--embed-subs")
            .arg("--sub-lang")
            .arg(lang);
    } else if sub {
        yt.arg("--embed-subs").arg("--sub-lang").arg(lang);
    }

    if format == 1 {
        yt.arg("-f")
            .arg(format!("bestvideo[height<={res}]+bestaudio"));
    } else if format == 2 {
        yt.arg("-f")
            .arg("bestvideo[height<={res}][ext=mp4]+bestaudio");
    }
    let output = yt
        .arg(link)
        .output()
        .expect("Failed to execute yt-dlp in Music");

    let log = String::from_utf8_lossy(&output.stdout);
    println!("{log}");

    let status: i8 = if output.status.success() { 2 } else { 3 };

    status
}

use crate::app::cores::cover;
use crate::app::cores::lyrics;

pub struct Music {
    pub link: String,
    pub directory: String,
    pub format: i8,
    pub lyrics: bool,
    pub frags: i8,
    pub lang_code: String,
    pub lyric_auto: bool,
    pub sim_rate: i8,
    pub musicbrainz: bool,
    pub lrclib: bool,
    pub kugou_lyrics: bool,
    pub cookies: Option<String>,
    pub use_cookies: bool,
    pub crop_cover: bool,
    pub use_playlist_cover: bool,
    pub sanitize_lyrics: bool,
    pub yt_dlp: PathBuf,
}
use regex::Regex;
use scraper::{Html, Selector};
use std::error::Error;
#[allow(dead_code)]
fn get_name_from_title(html: &str) -> String {
    let doc = Html::parse_document(&html);
    let sel = Selector::parse("title").unwrap();
    for i in doc.select(&sel) {
        return i.inner_html().replace(" - YouTube", "");
    }
    String::new()
}
#[allow(dead_code)]
fn get_all_songs_name_from_regex_playlist(html: &str) -> Vec<String> {
    let regex = Regex::new(r##""title":\{"runs":\[\{[^}]*\}\],"accessibility""##).unwrap();
    let mut list_of_song = vec![];
    for i in regex.find_iter(html) {
        let a = i
            .as_str()
            .replace(r##""}],"accessibility""##, "")
            .replace(r##""title":{"runs":[{"text":""##, "");
        dbg!(&a);
        list_of_song.push(a);
    }

    list_of_song
}
#[allow(dead_code)]
fn playlist_fix_url(url: &str) -> String {
    if url.contains("&list") {
        match url.split("&index=").nth(0) {
            Some(removed_index) => {
                let playlist_id = removed_index.split("&list=").last().unwrap();
                return format!("https://www.youtube.com/playlist?list={}", playlist_id);
            }
            None => {
                let playlist_id = url.split("&list=").last().unwrap();
                return format!("https://www.youtube.com/playlist?list={}", playlist_id);
            }
        }
    } else if url.contains("?list=") && url.contains("youtu.be") {
        let playlist_id = url.split("?list=").last().unwrap();
        return format!("https://www.youtube.com/playlist?list={}", playlist_id);
    } else {
        url.to_string()
    }
}

impl Music {
    pub fn download(self) -> i8 {
        let format_name = match self.format {
            1 => "opus",
            2 => "flac",
            3 => "mp3",
            4 => "m4a",
            5 => "wav",
            _ => return 3,
        };
        let n = self.frags.to_string();
        println!("{n}");

        let mut yt = Command::new(self.yt_dlp);

        if let Some(cookie) = self.cookies
            && self.use_cookies
        {
            yt.arg("--cookies").arg(cookie);
        }

        yt.arg("--concurrent-fragments")
            .arg(&n)
            .arg("-x")
            .arg("--audio-quality")
            .arg("0")
            .arg("--audio-format")
            .arg(format_name)
            .arg("--write-thumbnail")
            .arg("--add-metadata")
            .arg("--metadata-from-title")
            .arg("%(title)s")
            .arg("--parse-metadata")
            .arg("title:%(title)s")
            .arg("--parse-metadata")
            .arg("uploader:%(artist)s")
            .arg("--output")
            .arg("%(title)s.%(ext)s")
            .arg("--compat-options")
            .arg("no-live-chat")
            .current_dir(&self.directory);

        if self.lyrics {
            if self.lyric_auto {
                yt.arg("--write-auto-subs");
            }
            yt.arg("--write-subs").arg("--convert-subs").arg("lrc");

            if self.lang_code != "en" {
                yt.arg("--sub-langs").arg(&self.lang_code);
            }
        }
        yt.arg(&self.link);
        let output = yt.output().expect("Failed to execute yt-dlp in Music");

        let log = String::from_utf8(output.stdout).unwrap_or_default();
        println!("{log}");

        let play: Option<String>;
        let files: Vec<String>;
        #[cfg(target_os = "windows")]
        {
            if self.link.contains("list=") {
                match get_html(&self.link) {
                    Ok(html) => {
                        println!("case 1 worked");
                        play = Some(get_name_from_title(&html));
                        files = get_all_songs_name_from_regex_playlist(&html);
                    }
                    Err(e) => {
                        println!("Error case 1 : {e}");
                        play = None;
                        files = vec![];
                    }
                }
            } else {
                match get_html(&self.link) {
                    Ok(html) => {
                        println!("case 2 worked");
                        play = None;
                        files = vec![get_name_from_title(&html)];
                    }
                    Err(e) => {
                        println!("Error case 2: {e}");
                        play = None;
                        files = vec![];
                    }
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            const PREFIX: &str = "[download] Finished downloading playlist:";
            let playlist_name: Vec<String> = log
                .lines()
                .filter_map(|line| line.strip_prefix(PREFIX).map(str::trim).to_owned())
                .map(|l| l.to_string())
                .collect();

            play = playlist_name.get(0).cloned();

            files = log
                .lines()
                .filter(|line| line.starts_with("[Metadata]"))
                .map(|l| l.to_string())
                .collect();
        }

        for i in files.into_iter() {
            let extension = format!(".{}", format_name);
            println!("i: {i}");

            #[cfg(target_os = "linux")]
            let item = i.split("Adding metadata to \"").last().unwrap();
            #[cfg(target_os = "linux")]
            let item = item[0..item.len() - 1].to_string();
            #[cfg(target_os = "linux")]
            let filename = item.split(&extension).next().unwrap();
            #[cfg(target_os = "linux")]
            println!("item: {item}");

            #[cfg(target_os = "windows")]
            let filename = i;

            println!("filename: {filename}");

            let music_file = Path::new(&self.directory).join(format!("{}{}", filename, extension));
            println!("music dir: {music_file:?}");

            println!("Playlist name: {play:?}");

            match cover::embed(
                self.crop_cover,
                self.use_playlist_cover,
                &music_file,
                &self.directory,
                &filename,
                &play,
            ) {
                Ok(_) => println!("embeded cover"),
                Err(e) => println!("embed cover fail: {e}"),
            }

            if self.musicbrainz {
                let _ = musicbrainz::work(&music_file, self.sim_rate);
            }
            if self.lyrics {
                match lyrics::work(
                    &filename,
                    &music_file,
                    format_name,
                    &self.directory,
                    self.sanitize_lyrics,
                ) {
                    Ok(_) => println!("Lyrics from youtube embeded"),
                    Err(e) => println!("Fail to use lyrics from youtube: {e}"),
                }
            }
            if self.lrclib {
                let _ = lrclib_fetch(&music_file, &self.lang_code);
            }
            if self.kugou_lyrics {
                let _ = kugou::get(&music_file, &self.lang_code);
            }
        }
        if let Some(trash_cover) = play {
            let _ =
                std::fs::remove_file(Path::new(&self.directory).join(format!("{trash_cover}.png")));
            let _ =
                std::fs::remove_file(Path::new(&self.directory).join(format!("{trash_cover}.jpg")));
        }

        let status = if output.status.success() { 2 } else { 3 };

        status
    }
}
#[allow(dead_code)]
fn get_html(url: &str) -> Result<String, Box<dyn Error>> {
    Ok(ureq::get(playlist_fix_url(url))
        .call()?
        .body_mut()
        .read_to_string()?)
}
