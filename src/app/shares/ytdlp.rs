use crate::app::shares::lrclib::lrclib_fetch;
use crate::app::shares::musicbrainz;
use crate::app::shares::notify::{notification_done, notification_fail};
use std::path::Path;
use std::process::Command;

pub fn version_check() -> Option<String> {
    match Command::new("yt-dlp").arg("--version").output() {
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
) -> i8 {
    let n = frag.to_string().to_owned();

    let mut yt = Command::new("yt-dlp");
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
        yt.arg("-f").arg("bestvideo+bestaudio");
    } else if format == 2 {
        yt.arg("-f")
            .arg("bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best");
    }
    let output = yt.arg(link).output().expect("Fail To Run Yt-dlp");

    let log = String::from_utf8_lossy(&output.stdout);
    println!("{log}");

    let status: i8 = if output.status.success() { 2 } else { 3 };

    if status == 2 {
        let _ = notification_done("video downloader");
    } else {
        let _ = notification_fail("video downloader");
    }
    status
}

use crate::app::shares::cover;
use crate::app::shares::lyrics;

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
    pub cookies: Option<String>,
    pub use_cookies: bool,
    pub crop_cover: bool,
    pub use_playlist_cover: bool,
    pub sanatize_lyrics: bool,
}

impl Music {
    pub fn new(
        link: String,
        directory: String,
        format: i8,
        lyrics: bool,
        frags: i8,
        lang_code: String,
        lyric_auto: bool,
        sim_rate: i8,
        musicbrainz: bool,
        lrclib: bool,
        cookies: Option<String>,
        use_cookies: bool,
        crop_cover: bool,
        use_playlist_cover: bool,
        sanatize_lyrics: bool,
    ) -> Self {
        Self {
            link,
            directory,
            format,
            lyrics,
            frags,
            lang_code,
            lyric_auto,
            sim_rate,
            musicbrainz,
            lrclib,
            cookies,
            use_cookies,
            crop_cover,
            use_playlist_cover,
            sanatize_lyrics,
        }
    }
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

        let mut yt = Command::new("yt-dlp");

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
        let output = yt.output().expect("Failed to execute command");

        let log = String::from_utf8(output.stdout).unwrap_or_default();
        println!("{log}");

        let playlist_name: Vec<&str> = log
            .lines()
            .filter_map(|line| {
                const PREFIX: &str = "[download] Finished downloading playlist:";
                line.strip_prefix(PREFIX).map(str::trim)
            })
            .collect();
        let play: Option<_> = playlist_name.get(0).cloned();

        let files: Vec<&str> = log
            .lines()
            .filter(|line| line.starts_with("[Metadata]"))
            .collect();
        for i in files.into_iter() {
            println!("i: {i}");
            let item = i.split("Adding metadata to \"").last().unwrap();
            let item = item[0..item.len() - 1].to_string();
            println!("item: {item}");

            let extension = format!(".{}", format_name);
            let filename = &item.split(&extension).next().unwrap();
            println!("filename: {filename}");

            let music_file = Path::new(&self.directory).join(&item);
            println!("music dir: {music_file:?}");

            println!("Playlist name: {play:?}");

            match cover::embed(
                self.crop_cover,
                self.use_playlist_cover,
                &music_file,
                &self.directory,
                filename,
                play,
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
                    self.sanatize_lyrics,
                ) {
                    Ok(_) => println!("Lyrics from youtube embeded"),
                    Err(e) => println!("Fail to use lyrics from youtube: {e}"),
                }
                if self.lrclib {
                    let _ = lrclib_fetch(&music_file, &self.lang_code);
                }
            }
        }
        if let Some(trash_cover) = play {
            let _ =
                std::fs::remove_file(Path::new(&self.directory).join(format!("{trash_cover}.png")));
            let _ =
                std::fs::remove_file(Path::new(&self.directory).join(format!("{trash_cover}.jpg")));
        }

        let status = if output.status.success() { 2 } else { 3 };

        if status == 2 {
            let _ = notification_done("music downloader");
        } else {
            let _ = notification_fail("music downloader");
        }
        status
    }
}
