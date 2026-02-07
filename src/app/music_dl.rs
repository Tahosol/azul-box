use crate::app::cores::depen_manager::Depen;
use crate::app::cores::url_checker::{UrlStatus, playlist_check, remove_radio};
use crate::app::cores::{
    notify::{button_sound, done_sound, fail_sound},
    ytdlp,
};
use crate::app::share_view::lang_widget::LangThing;
use crate::app::share_view::url_status_view;
use eframe::egui::{self, Color32};
use native_dialog::DialogBuilder;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicI8, Ordering};
use std::sync::{Arc, Mutex};

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
    pub kugou_lyrics: bool,
    pub config_path: PathBuf,
    pub cookies: Option<String>,
    pub use_cookies: bool,
    pub crop_cover: bool,
    pub use_playlist_cover: bool,
    pub sanitize_lyrics: bool,
    pub url_status: UrlStatus,
    pub disable_radio: bool,
    error_message: Arc<Mutex<String>>,
}

use crate::app::cores::{config, files};

impl Default for MusicDownload {
    fn default() -> Self {
        let default_directory = dirs::audio_dir()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_else(|| String::from(""));
        let path = config::get_config_file_path();
        let configs = match config::load_config(&path) {
            Ok(config) => config,
            Err(e) => {
                log::error!("Fail to read config {e}");
                config::Config::default()
            }
        };
        Self {
            link: String::new(),
            out_directory: default_directory,
            status: Arc::new(AtomicI8::new(0)), // 0 = nothing / 1 = pending / 2 = Done / 3 = Fail
            // The unwrap here is good me in the futur pls DONT "FIX" this. the value is guarantee to be Some
            format: configs.music_dl.format.unwrap(),
            lyrics: configs.music_dl.lyrics.unwrap(),
            kugou_lyrics: configs.music_dl.kugou_lyrics.unwrap(),
            frag: configs.music_dl.fragments.unwrap(),
            sub_lang: configs.universal.language.unwrap(),
            auto_lyric: configs.music_dl.auto_gen_sub.unwrap(),
            sim_rate: configs.music_dl.threshold.unwrap(),
            musicbrainz: configs.music_dl.musicbrainz.unwrap(),
            lrclib: configs.music_dl.liblrc.unwrap(),
            cookies: configs.universal.cookies,
            config_path: path,
            use_cookies: configs.universal.use_cookies.unwrap(),
            crop_cover: configs.music_dl.crop_cover.unwrap(),
            use_playlist_cover: configs.music_dl.use_playlist_cover.unwrap(),
            sanitize_lyrics: false,
            url_status: UrlStatus::None,
            disable_radio: configs.music_dl.disable_radio.unwrap(),
            error_message: Arc::new(Mutex::new(String::new())),
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
                        cfg.music_dl.musicbrainz = Some(self.musicbrainz)
                    }) {
                        Ok(_) => {
                            log::info!("musicbrainz changed");
                        }
                        Err(e) => {
                            log::error!("Fail musicbrainz {e}");
                        }
                    }
                }
            });
            let slider = egui::widgets::Slider::new(&mut self.sim_rate, 0..=100)
                .text("Similarity threshold");
            let response = ui.add(slider);
            if response.changed() {
                match config::modifier_config(&self.config_path, |cfg| {
                    cfg.music_dl.threshold = Some(self.sim_rate)
                }) {
                    Ok(_) => {
                        log::info!("Changed threshold");
                    }
                    Err(e) => {
                        log::error!("Fail change threshold {e}");
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
            };
        } else {
            if ui.button(name).clicked() {
                self.format = numbername;
                match config::modifier_config(&self.config_path, |cfg| {
                    cfg.music_dl.format = Some(self.format)
                }) {
                    Ok(_) => {
                        log::info!("Changed format");
                    }
                    Err(e) => {
                        log::error!("Fail change format {e}");
                    }
                }
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
                    cfg.music_dl.auto_gen_sub = Some(self.auto_lyric)
                }) {
                    Ok(_) => {
                        log::info!("Changed auto lyric");
                    }
                    Err(e) => {
                        log::error!("Fail change auto lyric {e}");
                    }
                }
            }
        } else {
            if ui.button("Auto generated").clicked() {
                self.auto_lyric = true;
                match config::modifier_config(&self.config_path, |cfg| {
                    cfg.music_dl.auto_gen_sub = Some(self.auto_lyric)
                }) {
                    Ok(_) => {
                        log::info!("Changed auto lyric");
                    }
                    Err(e) => {
                        log::error!("Fail change auto lyric {e}");
                    }
                }
            }
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, depen: &Depen) {
        if self.format == 5 {
            self.lyrics = false;
        }
        ui.horizontal(|ui| {
            ui.menu_button("Setting", |ui| {
                ui.menu_button("cookies", |ui| {
                    let check = ui.checkbox(&mut self.use_cookies, "Use cookies");
                    if check.changed() {
                        match config::modifier_config(&self.config_path, |cfg| {
                            cfg.universal.use_cookies = Some(self.use_cookies)
                        }) {
                            Ok(_) => {
                                log::info!("Cookie usage successfully changed");
                            }
                            Err(e) => {
                                log::error!("Fail change use_cookies {e}");
                            }
                        }
                    }
                    if ui.button("Cookie directory").clicked() {
                        let path = DialogBuilder::file()
                            .set_location(&self.out_directory)
                            .add_filter("cookies.txt", ["txt"])
                            .open_single_file()
                            .show()
                            .unwrap();

                        if let Some(p) = path {
                            self.cookies = Some(p.to_string_lossy().into_owned());
                        } else {
                            log::info!("No file was selected.");
                        }
                    }
                });
                let radio_toggle = ui.toggle_value(&mut self.disable_radio, "Disable radio");
                if radio_toggle.changed() {
                    match config::modifier_config(&self.config_path, |cfg| {
                        cfg.music_dl.disable_radio = Some(self.disable_radio)
                    }) {
                        Ok(_) => {
                            log::info!("Changed disable_radio");
                        }
                        Err(e) => {
                            log::error!("Fail change disable_radio {e}");
                        }
                    }
                }
                ui.menu_button("Cover", |ui| {
                    let check_1 = ui.checkbox(&mut self.use_playlist_cover, "Use playlist cover");
                    if check_1.changed() {
                        match config::modifier_config(&self.config_path, |cfg| {
                            cfg.music_dl.use_playlist_cover = Some(self.use_playlist_cover)
                        }) {
                            Ok(_) => {
                                log::info!("Changed use_playlist_cover");
                            }
                            Err(e) => {
                                log::error!("Fail change use_playlist_cover {e}");
                            }
                        }
                    }
                    let check_2 = ui.checkbox(&mut self.crop_cover, "Crop cover to 1:1");
                    if check_2.changed() {
                        match config::modifier_config(&self.config_path, |cfg| {
                            cfg.music_dl.crop_cover = Some(self.crop_cover)
                        }) {
                            Ok(_) => {
                                log::info!("Changed crop_cover");
                            }
                            Err(e) => {
                                log::error!("Fail change Crop cover {e}");
                            }
                        }
                    }
                });
                ui.menu_button("Format", |ui| {
                    self.format_button(ui, "OPUS", 1);
                    self.format_button(ui, "FLAC", 2);
                    self.format_button(ui, "MP3", 3);
                    self.format_button(ui, "M4A", 4);
                    self.format_button(ui, "WAV", 5);
                });
                ui.menu_button("Lyrics", |ui| {
                    let lang_in = self.sub_lang.clone();
                    self.sub_lang = LangThing::lang_chooser(ui, lang_in);

                    ui.separator();

                    let youtube_lyrics = ui.checkbox(&mut self.lyrics, "youtube lyrics");
                    if youtube_lyrics.changed() {
                        match config::modifier_config(&self.config_path, |cfg| {
                            cfg.music_dl.lyrics = Some(self.lyrics)
                        }) {
                            Ok(_) => {
                                log::info!("Changed yt lyrics");
                            }
                            Err(e) => {
                                log::error!("Fail change yt lyrics {e}");
                            }
                        }
                    }
                    if self.lyrics {
                        ui.toggle_value(&mut self.sanitize_lyrics, "Sanitization");
                        self.auto_on(ui);
                    }

                    ui.separator();

                    let check = ui.checkbox(&mut self.lrclib, "Lrclib lyrics");
                    if check.changed() {
                        match config::modifier_config(&self.config_path, |cfg| {
                            cfg.music_dl.liblrc = Some(self.lrclib)
                        }) {
                            Ok(_) => {
                                log::info!("Changed Liblrc");
                            }
                            Err(e) => {
                                log::error!("Fail change Liblrc {e}");
                            }
                        }
                    }
                    ui.separator();
                    let kugou = ui.checkbox(&mut self.kugou_lyrics, "kugou lyrics");
                    if kugou.changed() {
                        match config::modifier_config(&self.config_path, |cfg| {
                            cfg.music_dl.kugou_lyrics = Some(self.kugou_lyrics)
                        }) {
                            Ok(_) => {
                                log::info!("Changed kugou");
                            }
                            Err(e) => {
                                log::error!("Fail change kugou {e}");
                            }
                        }
                    }
                });
                self.music_brainz_button(ui);

                let check =
                    ui.add(egui::widgets::Slider::new(&mut self.frag, 1..=10).text("Fragments"));
                if check.changed() {
                    match config::modifier_config(&self.config_path, |cfg| {
                        cfg.music_dl.fragments = Some(self.frag)
                    }) {
                        Ok(_) => {
                            log::info!("Changed frag");
                        }
                        Err(e) => {
                            log::error!("Fail frag {e}");
                        }
                    }
                }
                if ui.button("Close").clicked() {}
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
            if self.status.load(Ordering::Relaxed) == 1 {
                url_status_view::show(ui, &self.url_status);
            }
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
                    log::info!("No file selected.");
                }
            };

            if self.status.load(Ordering::Relaxed) != 1 {
                if ui.button("Download").clicked() {
                    if self.disable_radio {
                        self.link = remove_radio(&self.link)
                    }
                    self.url_status = playlist_check(&self.link);
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
                    let crop = self.crop_cover;
                    let playlist_cover = self.use_playlist_cover;
                    let sanitization = self.sanitize_lyrics;
                    let yt_dlp_path = depen.yt_dlp.clone();
                    let kugou = self.kugou_lyrics;
                    let error_message_clone = Arc::clone(&self.error_message);

                    tokio::task::spawn(async move {
                        let yt = ytdlp::Music {
                            link,
                            directory,
                            format: format,
                            lyrics: lyrics,
                            frags: frags,
                            lang_code: lang_code,
                            lyric_auto: auto,
                            sim_rate: sim,
                            musicbrainz: brain,
                            lrclib: lrclib,
                            kugou_lyrics: kugou,
                            cookies: cook,
                            use_cookies: use_cook,
                            crop_cover: crop,
                            use_playlist_cover: playlist_cover,
                            sanitize_lyrics: sanitization,
                            yt_dlp: yt_dlp_path,
                        };
                        match yt.download() {
                            Ok(_) => {
                                progress.store(2, Ordering::Relaxed);
                                let _ = done_sound();
                            }
                            Err(e) => {
                                progress.store(3, Ordering::Relaxed);
                                *error_message_clone.lock().unwrap() = e.to_string();
                                let _ = fail_sound();
                            }
                        }
                    });
                }
            } else if self.status.load(Ordering::Relaxed) == 1 {
                if ui.button("Cancel").clicked() {
                    #[cfg(target_os = "windows")]
                    {
                        let _ = button_sound();
                        let _ = Command::new("taskkill")
                            .args(&["/IM", "yt-dlp.exe", "/F"])
                            .output();
                    }
                    #[cfg(target_os = "linux")]
                    {
                        let _ = button_sound();
                        let _ = Command::new("pkill").arg("yt-dlp").output();
                    }
                }
            }
            if self.status.load(Ordering::Relaxed) == 1 {
                if let Some(file) = files::file_finder_no_name(&self.out_directory, &["part"]) {
                    match fs::metadata(&file) {
                        Ok(value) => {
                            if let Some(f) = file.file_name().and_then(|f| f.to_str()) {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{f}: {} Mib",
                                        (value.len() / 1024 / 1024)
                                    ))
                                    .color(Color32::LIGHT_BLUE),
                                );
                            }
                        }

                        Err(e) => {
                            if let Some(f) = file.file_name().and_then(|f| f.to_str()) {
                                ui.label(
                                    egui::RichText::new(format!("{f}: {}", e))
                                        .color(Color32::LIGHT_BLUE),
                                );
                            }
                        }
                    }
                }
            } else if self.status.load(Ordering::Relaxed) == 3 {
                ui.spacing();
                ui.separator();
                egui::ScrollArea::vertical()
                    .max_height(100.0)
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(format!("{}", self.error_message.lock().unwrap()))
                                .color(Color32::LIGHT_RED)
                                .size(16.0),
                        );
                    });
            }
        });
    }
}
