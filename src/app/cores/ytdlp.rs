use crate::app::cores::depen_manager::{Depen, get_path};
use crate::app::cores::lrclib::lrclib_fetch;
use crate::app::cores::{kugou, musicbrainz};
use std::error::Error;
use std::fs;
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
) -> Result<(), Box<dyn Error>> {
    let n = frag.to_string().to_owned();

    let mut yt = Command::new(yt_dlp);
    if let Some(cookie) = cookies
        && use_cookies
    {
        yt.arg("--cookies").arg(cookie);
    }

    let deno_path = get_path().deno;
    if let Some(deno) = deno_path.to_str() {
        yt.arg("--js-runtimes").arg(format!("deno:{}", deno));
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
            .arg(format!("bv*[ext=mp4][height<={res}]+ba[ext=m4a]"));
    }
    let output = yt.arg(link).output()?;

    let log = String::from_utf8_lossy(&output.stdout);
    log::info!("{log}");

    if output.status.success() {
        log::warn!("{}", String::from_utf8_lossy(&output.stderr));
        Ok(())
    } else {
        log::error!("{}", String::from_utf8_lossy(&output.stderr));
        Err(String::from_utf8_lossy(&output.stderr).into())
    }
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

use serde::Deserialize;
#[derive(Debug, Deserialize)]
struct InfoJson {
    #[serde(rename = "_type")]
    filetype: String,
    title: String,
}

pub fn get_all_music_title_and_playlist(
    path: &Path,
) -> Result<(Vec<String>, Option<String>), Box<dyn Error>> {
    let mut titles: Vec<String> = vec![];
    let mut playlist: Option<String> = None;
    let reader = fs::read_dir(path)?;

    for i in reader {
        let item = i?.path();
        if let Some(file) = item.to_str()
            && file.contains(".info.json")
        {
            if let Ok(infojson) = serde_json::from_str::<InfoJson>(&fs::read_to_string(&item)?) {
                if infojson.filetype == "playlist" {
                    playlist = Some(infojson.title);
                } else {
                    titles.push(infojson.title);
                }
                fs::remove_file(item)?;
            } else {
                log::warn!("Failed to parse JSON for item: {:?}", item);
                continue;
            }
        }
    }

    return Ok((titles, playlist));
}

impl Music {
    pub fn download(self) -> Result<(), Box<dyn Error>> {
        let format_name = match self.format {
            1 => "opus",
            2 => "flac",
            3 => "mp3",
            4 => "m4a",
            5 => "wav",
            _ => return Err("Invalided format".into()),
        };
        let n = self.frags.to_string();
        log::info!("{}", n);

        let mut yt = Command::new(self.yt_dlp);

        if let Some(cookie) = self.cookies
            && self.use_cookies
        {
            yt.arg("--cookies").arg(cookie);
        }
        let deno_path = get_path().deno;
        if let Some(deno) = deno_path.to_str() {
            yt.arg("--js-runtimes").arg(format!("deno:{}", deno));
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
            .arg("--write-info-json")
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
        let output = yt.output()?;

        let log = String::from_utf8(output.stdout)?;
        log::info!("{}", log);

        let (files, play) = get_all_music_title_and_playlist(Path::new(&self.directory))?;

        for i in files {
            let extension = format!(".{}", format_name);
            let filename = i;

            log::info!("filename: {filename}");

            let music_file = Path::new(&self.directory).join(format!("{}{}", filename, extension));
            log::info!("music dir: {music_file:?}");

            log::info!("Playlist name: {play:?}");

            match cover::embed(
                self.crop_cover,
                self.use_playlist_cover,
                &music_file,
                &self.directory,
                &filename,
                &play,
            ) {
                Ok(_) => log::info!("embedded cover"),
                Err(e) => log::error!("embed cover fail: {e}"),
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
                    Ok(_) => log::info!("Lyrics from youtube embedded"),
                    Err(e) => log::error!("Fail to use lyrics from youtube: {e}"),
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
        if output.status.success() {
            log::warn!("{}", String::from_utf8_lossy(&output.stderr));
            Ok(())
        } else {
            log::error!("{}", String::from_utf8_lossy(&output.stderr));
            Err(String::from_utf8_lossy(&output.stderr).into())
        }
    }
}
