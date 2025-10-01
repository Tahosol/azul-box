use crate::app::shares::lang::LangThing;
use crate::app::shares::lrclib::lrclib_fetch;
use crate::app::shares::musicbrainz::musicbrain_work;
use crate::app::shares::notify::{
    button_sound, done_sound, fail_sound, notification_done, notification_fail,
};
use eframe::egui::{self, Color32};
use native_dialog::DialogBuilder;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicI8, Ordering};

pub struct MusicDownload {
    pub link: String,
    pub out_directory: String,
    pub status: Arc<AtomicI8>,
    pub format: i8,
    pub lyrics: bool,
    pub frag: i8,
    pub sub_lang: String,
    pub auto_lyric: bool,
    pub sim_rate: i8,
    pub musicbrainz: bool,
    pub lrclib: bool,
    pub config_path: PathBuf,
    pub cookies: Option<String>,
    pub use_cookies: bool,
}

use crate::app::shares::config;

impl Default for MusicDownload {
    fn default() -> Self {
        let default_directory = dirs::audio_dir()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_else(|| String::from(""));
        let path = config::get_config_file_path();
        let configs = match config::load_config(&path) {
            Ok(config) => config,
            Err(e) => {
                println!("music_dl: Fail to read config {e}");
                config::Config::default()
            }
        };
        Self {
            link: String::new(),
            out_directory: default_directory,
            status: Arc::new(AtomicI8::new(0)), // 0 = nothing / 1 = pending / 2 = Done / 3 = Fail
            format: configs.music_dl.format,
            lyrics: configs.music_dl.lyrics,
            frag: configs.music_dl.fragments,
            sub_lang: configs.universal.language,
            auto_lyric: configs.music_dl.auto_gen_sub,
            sim_rate: configs.music_dl.threshold,
            musicbrainz: configs.music_dl.musicbrainz,
            lrclib: configs.music_dl.liblrc,
            cookies: configs.universal.cookies,
            config_path: path,
            use_cookies: configs.universal.use_cookies,
        }
    }
}

impl MusicDownload {
    fn start_download_status(&mut self) {
        self.status.store(1, Ordering::Relaxed);
    }
    fn music_brainz_button(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Musicbrainz", |ui| {
            ui.horizontal(|ui| {
                ui.label("On/Off: ");
                let check = ui.checkbox(&mut self.musicbrainz, "");
                if check.changed() {
                    match config::modifier_config(&self.config_path, |cfg| {
                        cfg.music_dl.musicbrainz = self.musicbrainz
                    }) {
                        Ok(_) => {
                            println!("music_dl: musicbrainz changed")
                        }
                        Err(e) => {
                            println!("music_dl: Fail musicbrainz {e}")
                        }
                    }
                }
            });
            let slider = egui::widgets::Slider::new(&mut self.sim_rate, 0..=100)
                .text("Similarity threshold");
            let response = ui.add(slider);
            if response.changed() {
                match config::modifier_config(&self.config_path, |cfg| {
                    cfg.music_dl.threshold = self.sim_rate
                }) {
                    Ok(_) => {
                        println!("music_dl: Changed threshold")
                    }
                    Err(e) => {
                        println!("music_dl: Fail change threshold {e}")
                    }
                }
            }
        });
    }
    fn format_button(&mut self, ui: &mut egui::Ui, name: &str, numbername: i8) {
        if self.format == numbername {
            if ui
                .add(egui::Button::new(
                    egui::RichText::new(name).color(Color32::LIGHT_BLUE),
                ))
                .clicked()
            {
                self.format = numbername;
                ui.close();
            };
        } else {
            if ui.button(name).clicked() {
                self.format = numbername;
                match config::modifier_config(&self.config_path, |cfg| {
                    cfg.music_dl.format = self.format
                }) {
                    Ok(_) => {
                        println!("music_dl: Changed format")
                    }
                    Err(e) => {
                        println!("music_dl: Fail change format {e}")
                    }
                }
                ui.close();
            };
        }
    }
    fn auto_on(&mut self, ui: &mut egui::Ui) {
        if self.auto_lyric {
            if ui
                .add(egui::Button::new(
                    egui::RichText::new("Auto generated").color(Color32::LIGHT_BLUE),
                ))
                .clicked()
            {
                self.auto_lyric = false;
                match config::modifier_config(&self.config_path, |cfg| {
                    cfg.music_dl.auto_gen_sub = self.auto_lyric
                }) {
                    Ok(_) => {
                        println!("music_dl: Changed auto lyric")
                    }
                    Err(e) => {
                        println!("music_dl: Fail change auto lyric {e}")
                    }
                }
            }
        } else {
            if ui.button("Auto generated").clicked() {
                self.auto_lyric = true;
                match config::modifier_config(&self.config_path, |cfg| {
                    cfg.music_dl.auto_gen_sub = self.auto_lyric
                }) {
                    Ok(_) => {
                        println!("music_dl: Changed auto lyric")
                    }
                    Err(e) => {
                        println!("music_dl: Fail change auto lyric {e}")
                    }
                }
            }
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        if self.format == 5 {
            self.lyrics = false;
        }
        ui.horizontal(|ui| {
            ui.menu_button("Setting", |ui| {
                let check = ui.checkbox(&mut self.use_cookies, "Use cookies");
                if check.changed() {
                    match config::modifier_config(&self.config_path, |cfg| {
                        cfg.universal.use_cookies = self.use_cookies
                    }) {
                        Ok(_) => {
                            println!("music_dl: Changed use_cookies")
                        }
                        Err(e) => {
                            println!("music_dl: Fail change use_cookies {e}")
                        }
                    }
                }
                ui.menu_button("Format", |ui| {
                    self.format_button(ui, "OPUS", 1);
                    self.format_button(ui, "FLAC", 2);
                    self.format_button(ui, "MP3", 3);
                    self.format_button(ui, "M4A", 4);
                    self.format_button(ui, "WAV", 5);
                });
                ui.menu_button("Lyrics", |ui| {
                    if self.lyrics && self.format != 5 {
                        ui.horizontal(|ui| {
                            ui.label("On/Off: ");
                            let check = ui.checkbox(&mut self.lyrics, "");
                            if check.changed() {
                                match config::modifier_config(&self.config_path, |cfg| {
                                    cfg.music_dl.lyrics = self.lyrics
                                }) {
                                    Ok(_) => {
                                        println!("music_dl: Changed lyric")
                                    }
                                    Err(e) => {
                                        println!("music_dl: Fail change lyric {e}")
                                    }
                                }
                            }
                        });
                        let lang_in = self.sub_lang.clone();
                        self.sub_lang = LangThing::lang_chooser(ui, lang_in);
                        self.auto_on(ui);
                        ui.separator();
                        let check = ui.checkbox(&mut self.lrclib, "Liblrc lyrics");
                        if check.changed() {
                            match config::modifier_config(&self.config_path, |cfg| {
                                cfg.music_dl.liblrc = self.lrclib
                            }) {
                                Ok(_) => {
                                    println!("music_dl: Changed lrclib")
                                }
                                Err(e) => {
                                    println!("music_dl: Fail change lrclib {e}")
                                }
                            }
                        }
                    } else if self.format != 5 {
                        ui.horizontal(|ui| {
                            ui.label("On/Off: ");
                            let check = ui.checkbox(&mut self.lyrics, "");
                            if check.changed() {
                                match config::modifier_config(&self.config_path, |cfg| {
                                    cfg.music_dl.lyrics = self.lyrics
                                }) {
                                    Ok(_) => {
                                        println!("music_dl: Changed lyric")
                                    }
                                    Err(e) => {
                                        println!("music_dl: Fail change lyric {e}")
                                    }
                                }
                            }
                        });
                    }
                });
                self.music_brainz_button(ui);

                let check =
                    ui.add(egui::widgets::Slider::new(&mut self.frag, 1..=10).text("Fragments"));
                if check.changed() {
                    match config::modifier_config(&self.config_path, |cfg| {
                        cfg.music_dl.fragments = self.frag
                    }) {
                        Ok(_) => {
                            println!("music_dl: Changed fragments")
                        }
                        Err(e) => {
                            println!("music_dl: Fail change fragments {e}")
                        }
                    }
                }
                if ui.button("Close").clicked() {
                    ui.close();
                }
            });
            ui.label("Status: ");
            if self.status.load(Ordering::Relaxed) == 1 {
                ui.spinner();
            } else if self.status.load(Ordering::Relaxed) == 2 {
                ui.colored_label(Color32::LIGHT_GREEN, "Done!");
            } else if self.status.load(Ordering::Relaxed) == 3 {
                ui.colored_label(Color32::LIGHT_RED, "Fail!");
            }
        });
        ui.separator();
        ui.vertical_centered(|ui| {
            let link_label = ui.label("Link: ");
            ui.text_edit_singleline(&mut self.link)
                .labelled_by(link_label.id);

            let dir_label = ui.label("Directory: ");
            if ui
                .text_edit_singleline(&mut self.out_directory)
                .labelled_by(dir_label.id)
                .clicked()
            {
                let path = DialogBuilder::file()
                    .set_location(&self.out_directory)
                    .open_single_dir()
                    .show()
                    .unwrap();

                if let Some(p) = path {
                    self.out_directory = p.to_string_lossy().into_owned();
                } else {
                    println!("No file selected.");
                }
            };

            if self.status.load(Ordering::Relaxed) != 1 {
                if ui.button("Download").clicked() {
                    let _ = button_sound();

                    self.start_download_status();

                    let link = self.link.clone();
                    let directory = self.out_directory.clone();
                    let format = self.format;
                    let progress = self.status.clone();
                    let lyrics = self.lyrics;
                    let frags = self.frag;
                    let lang_code = self.sub_lang.clone();
                    let auto = self.auto_lyric;
                    let brain = self.musicbrainz;
                    let sim = self.sim_rate;
                    let lrclib = self.lrclib;
                    let cook = self.cookies.clone();
                    let use_cook = self.use_cookies;

                    tokio::task::spawn(async move {
                        let status = download(
                            link, directory, format, lyrics, frags, lang_code, auto, sim, brain,
                            lrclib, cook, use_cook,
                        );
                        progress.store(status, Ordering::Relaxed);
                        if status == 2 {
                            let _ = done_sound();
                        } else {
                            let _ = fail_sound();
                        }
                    });
                }
            } else if self.status.load(Ordering::Relaxed) == 1 {
                if ui.button("Cancel").clicked() {
                    let _ = button_sound();
                    let _ = Command::new("pkill").arg("yt-dlp").output();
                }
            }
        });
    }
}

fn download(
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
) -> i8 {
    if format == 1 {
        format_dl(
            link,
            directory,
            "opus",
            lyrics,
            frags,
            lang_code,
            lyric_auto,
            sim_rate,
            musicbrainz,
            lrclib,
            cookies,
            use_cookies,
        )
    } else if format == 2 {
        format_dl(
            link,
            directory,
            "flac",
            lyrics,
            frags,
            lang_code,
            lyric_auto,
            sim_rate,
            musicbrainz,
            lrclib,
            cookies,
            use_cookies,
        )
    } else if format == 3 {
        format_dl(
            link,
            directory,
            "mp3",
            lyrics,
            frags,
            lang_code,
            lyric_auto,
            sim_rate,
            musicbrainz,
            lrclib,
            cookies,
            use_cookies,
        )
    } else if format == 4 {
        format_dl(
            link,
            directory,
            "m4a",
            lyrics,
            frags,
            lang_code,
            lyric_auto,
            sim_rate,
            musicbrainz,
            lrclib,
            cookies,
            use_cookies,
        )
    } else if format == 5 {
        format_dl(
            link,
            directory,
            "wav",
            lyrics,
            frags,
            lang_code,
            lyric_auto,
            sim_rate,
            musicbrainz,
            lrclib,
            cookies,
            use_cookies,
        )
    } else {
        3
    }
}

fn format_dl(
    link: String,
    directory: String,
    format_name: &str,
    lyrics: bool,
    frags: i8,
    lang_code: String,
    auto_lyric: bool,
    sim_rate: i8,
    musicbrainz: bool,
    lrclib: bool,
    cookies: Option<String>,
    use_cookies: bool,
) -> i8 {
    let n = frags.to_string();
    println!("{n}");

    let files: Vec<&str>;

    let mut yt = Command::new("yt-dlp");

    if let Some(cookie) = cookies
        && use_cookies
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
        .arg("--embed-thumbnail")
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
        .current_dir(&directory);

    let status: i8;
    if lyrics {
        if auto_lyric {
            yt.arg("--write-auto-subs");
        }
        yt.arg("--write-subs").arg("--convert-subs").arg("lrc");

        if lang_code != "en" {
            yt.arg("--sub-langs").arg(&lang_code);
        }

        yt.arg(&link);
        let output = yt.output().expect("Failed to execute command");

        let log = String::from_utf8(output.stdout).unwrap_or_else(|_| "Life suck".to_string());
        println!("{log}");

        files = log
            .lines()
            .filter(|line| line.starts_with("[EmbedThumbnail]"))
            .collect();
        for i in files.into_iter() {
            println!("i: {i}");
            let item = i.split("Adding thumbnail to \"").last().unwrap();
            println!("item: {item}");
            let extension = format!(".{}\"", format_name);
            let filename = &item.split(&extension).next().unwrap();
            println!("filename: {filename}");
            let music_file = format!("{}/{}", &directory, &item[0..item.len() - 1].to_string());
            println!("music dir:{music_file}");
            lyrics_work(&filename, &music_file, format_name, &directory);
            let music_file = Path::new(&music_file);
            if musicbrainz {
                let _ = musicbrain_work(&music_file, sim_rate);
            }
            if lrclib {
                let _ = lrclib_fetch(&music_file, &lang_code);
            }
        }
        status = if output.status.success() { 2 } else { 3 };
    } else {
        yt.arg(&link);
        let output = yt.output().expect("Failed to execute command");
        let log = String::from_utf8(output.stdout).unwrap_or_else(|_| "Fail".to_string());
        println!("{log}");

        if musicbrainz {
            files = log
                .lines()
                .filter(|line| line.starts_with("[EmbedThumbnail]"))
                .collect();
            for i in files.into_iter() {
                println!("i: {i}");
                let item = i.split("Adding thumbnail to \"").last().unwrap();
                println!("item: {item}");

                let music_file = format!("{}/{}", &directory, &item[0..item.len() - 1].to_string());
                println!("music dir:{music_file}");
                let music_file = Path::new(&music_file);
                let _ = musicbrain_work(&music_file, sim_rate);
            }
        }

        status = if output.status.success() { 2 } else { 3 };
    }
    if status == 2 {
        let _ = notification_done("music downloader");
    } else {
        let _ = notification_fail("music downloader");
    }

    status
}

fn lyrics_work(filename: &str, music_file: &str, format_name: &str, directory: &str) {
    let lyrics_file = finder_lyrics(&directory, &filename).unwrap();
    let music_file = Path::new(&music_file);
    let lyrics = match fs::read_to_string(&lyrics_file) {
        Ok(file) => file,
        Err(error) => {
            println!("{:?}", error);
            "No-1-1!!!F".to_string()
        }
    };
    if (!(lyrics == "No-1-1!!!F") && format_name == "flac")
        || (!(lyrics == "No-1-1!!!F") && format_name == "opus")
        || (!(lyrics == "No-1-1!!!F") && format_name == "mp3")
        || (!(lyrics == "No-1-1!!!F") && format_name == "m4a")
    {
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
