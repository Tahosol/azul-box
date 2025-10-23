use crate::app::shares::lang::LangThing;
use eframe::egui::{self, Color32};
use native_dialog::DialogBuilder;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicI8, Ordering};

use crate::app::shares::notify::{
    button_sound, done_sound, fail_sound, notification_done, notification_fail,
};

use std::path::PathBuf;

pub struct VideoDownload {
    pub link: String,
    pub out_directory: String,
    pub status: Arc<AtomicI8>,
    pub format: i8,
    pub frag: i8,
    pub subtitle: bool,
    pub sub_lang: String,
    pub auto_sub: bool,
    pub config_path: PathBuf,
    pub cookies: Option<String>,
    pub use_cookies: bool,
}

use crate::app::shares::config;

impl Default for VideoDownload {
    fn default() -> Self {
        let default_directory = dirs::video_dir()
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
            format: configs.video_dl.format,
            frag: configs.video_dl.fragments,
            subtitle: configs.video_dl.subtitle,
            sub_lang: configs.universal.language,
            auto_sub: configs.video_dl.auto_gen_sub,
            config_path: path,
            cookies: configs.universal.cookies,
            use_cookies: configs.universal.use_cookies,
        }
    }
}

impl VideoDownload {
    fn start_download_status(&mut self) {
        self.status.store(1, Ordering::Relaxed);
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
                    cfg.video_dl.format = self.format
                }) {
                    Ok(_) => {
                        println!("video_dl: Changed format")
                    }
                    Err(e) => {
                        println!("video_dl: Fail change format {e}")
                    }
                }
            };
        }
    }
    fn auto_on(&mut self, ui: &mut egui::Ui) {
        if self.auto_sub {
            if ui
                .add(egui::Button::new(
                    egui::RichText::new("Auto generated").color(Color32::LIGHT_BLUE),
                ))
                .clicked()
            {
                self.auto_sub = false;
                match config::modifier_config(&self.config_path, |cfg| {
                    cfg.video_dl.auto_gen_sub = self.auto_sub
                }) {
                    Ok(_) => {
                        println!("video_dl: Changed auto_sub")
                    }
                    Err(e) => {
                        println!("video_dl: Fail change auto_sub {e}")
                    }
                }
            }
        } else {
            if ui.button("Auto generated").clicked() {
                self.auto_sub = true;
                match config::modifier_config(&self.config_path, |cfg| {
                    cfg.video_dl.auto_gen_sub = self.auto_sub
                }) {
                    Ok(_) => {
                        println!("video_dl: Changed auto_sub")
                    }
                    Err(e) => {
                        println!("video_dl: Fail change auto_sub {e}")
                    }
                }
            }
        }
    }
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.menu_button("Setting", |ui| {
                let check = ui.checkbox(&mut self.use_cookies, "Use cookies");
                if check.changed() {
                    match config::modifier_config(&self.config_path, |cfg| {
                        cfg.universal.use_cookies = self.use_cookies
                    }) {
                        Ok(_) => {
                            println!("video_dl: Changed use_cookies")
                        }
                        Err(e) => {
                            println!("video_dl: Fail change use_cookies {e}")
                        }
                    }
                }
                ui.menu_button("Format", |ui| {
                    self.format_button(ui, "MKV", 1);
                    self.format_button(ui, "MP4", 2);
                });
                ui.menu_button("Subtitles", |ui| {
                    if self.subtitle {
                        ui.horizontal(|ui| {
                            ui.label("On/Off: ");
                            let check = ui.checkbox(&mut self.subtitle, "");
                            if check.changed() {
                                match config::modifier_config(&self.config_path, |cfg| {
                                    cfg.video_dl.subtitle = self.subtitle
                                }) {
                                    Ok(_) => {
                                        println!("video_dl: Changed subtitle")
                                    }
                                    Err(e) => {
                                        println!("video_dl: Fail change subtitle {e}")
                                    }
                                }
                            }
                        });
                        let lang_in = self.sub_lang.clone();
                        self.sub_lang = LangThing::lang_chooser(ui, lang_in);
                        self.auto_on(ui);
                    } else {
                        ui.horizontal(|ui| {
                            ui.label("On/Off: ");
                            let check = ui.checkbox(&mut self.subtitle, "");
                            if check.changed() {
                                match config::modifier_config(&self.config_path, |cfg| {
                                    cfg.video_dl.subtitle = self.subtitle
                                }) {
                                    Ok(_) => {
                                        println!("video_dl: Changed subtitle")
                                    }
                                    Err(e) => {
                                        println!("video_dl: Fail change subtitle {e}")
                                    }
                                }
                            }
                        });
                    }
                });

                let c =
                    ui.add(egui::widgets::Slider::new(&mut self.frag, 1..=10).text("Fragments"));
                if c.changed() {
                    match config::modifier_config(&self.config_path, |cfg| {
                        cfg.video_dl.fragments = self.frag
                    }) {
                        Ok(_) => {
                            println!("video_dl: Changed fragments")
                        }
                        Err(e) => {
                            println!("video_dl: Fail change fragments {e}")
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
                    let frags = self.frag;
                    let progress = self.status.clone();
                    let subtile = self.subtitle;
                    let lang = self.sub_lang.clone();
                    let auto_gen = self.auto_sub;
                    let cook = self.cookies.clone();
                    let use_cook = self.use_cookies;

                    tokio::task::spawn(async move {
                        let status = download(
                            link, directory, format, frags, subtile, &lang, auto_gen, cook,
                            use_cook,
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
