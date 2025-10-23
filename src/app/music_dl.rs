use crate::app::shares::lang::LangThing;
use crate::app::shares::{
    notify::{button_sound, done_sound, fail_sound},
    ytdlp,
};
use eframe::egui::{self, Color32};
use native_dialog::DialogBuilder;
use std::path::PathBuf;
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
                        let yt = ytdlp::Music::new(
                            link, directory, format, lyrics, frags, lang_code, auto, sim, brain,
                            lrclib, cook, use_cook,
                        );
                        let status = yt.download();
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
