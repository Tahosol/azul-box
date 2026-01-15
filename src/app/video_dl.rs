use crate::app::cores::depen_manager::Depen;
use crate::app::cores::url_checker::{UrlStatus, playlist_check};
use crate::app::share_view::lang_widget::LangThing;
use crate::app::share_view::url_status_view;
use eframe::egui::{self, Color32};
use native_dialog::DialogBuilder;
use std::fs;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicI8, Ordering};

use crate::app::cores::{
    notify::{button_sound, done_sound, fail_sound},
    ytdlp,
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
    pub res: i32,
    url_status: UrlStatus,
}

use crate::app::cores::{config, files};

impl Default for VideoDownload {
    fn default() -> Self {
        let default_directory = dirs::video_dir()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_else(|| String::from(""));
        let path = config::get_config_file_path();
        let configs = match config::load_config(&path) {
            Ok(config) => config,
            Err(e) => {
                log::error!("Failed to read config: {e}");
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
            res: configs.video_dl.resolution,
            url_status: UrlStatus::None,
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
                        log::info!("Format successfully changed");
                    }
                    Err(e) => {
                        log::error!("Fail change format {e}");
                    }
                }
            };
        }
    }
    fn res_button(&mut self, ui: &mut egui::Ui, name: &str, res: i32) {
        if self.res == res {
            if ui
                .add(egui::Button::new(
                    egui::RichText::new(name).color(Color32::LIGHT_BLUE),
                ))
                .clicked()
            {
                self.res = res;
            };
        } else {
            if ui.button(name).clicked() {
                self.res = res;
                match config::modifier_config(&self.config_path, |cfg| {
                    cfg.video_dl.resolution = self.res
                }) {
                    Ok(_) => {
                        log::info!("Resolution successfully changed");
                    }
                    Err(e) => {
                        log::error!("Fail change Resolution {e}");
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
                        log::info!("Auto subtitle generation successfully changed");
                    }
                    Err(e) => {
                        log::error!("Fail change auto_sub {e}");
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
                        log::info!("Changed auto_sub");
                    }
                    Err(e) => {
                        log::error!("Fail change auto_sub {e}");
                    }
                }
            }
        }
    }
    pub fn ui(&mut self, ui: &mut egui::Ui, depen: &Depen) {
        ui.horizontal(|ui| {
            ui.menu_button("Setting", |ui| {
                let check = ui.checkbox(&mut self.use_cookies, "Use cookies");
                if check.changed() {
                    match config::modifier_config(&self.config_path, |cfg| {
                        cfg.universal.use_cookies = self.use_cookies
                    }) {
                        Ok(_) => {
                            log::info!("Cookie usage successfully changed");
                        }
                        Err(e) => {
                            log::error!("Fail change use_cookies {e}");
                        }
                    }
                }
                ui.menu_button("Resolution", |ui| {
                    self.res_button(ui, "144p", 144);
                    self.res_button(ui, "240p", 240);
                    self.res_button(ui, "360p", 360);
                    self.res_button(ui, "480p", 480);
                    self.res_button(ui, "720p", 720);
                    self.res_button(ui, "1080p", 1080);
                    self.res_button(ui, "1440p", 1440);
                    self.res_button(ui, "2160p", 2160);
                });
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
                                        log::info!("Subtitle settings successfully changed");
                                    }
                                    Err(e) => {
                                        log::error!("Fail change subtitle {e}");
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
                                        log::info!("Changed subtitle");
                                    }
                                    Err(e) => {
                                        log::error!("Fail change subtitle {e}");
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
                            log::info!("Fragment settings successfully changed");
                        }
                        Err(e) => {
                            log::error!("Fail change fragment {e}");
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
                    log::info!("No file was selected.");
                }
            };

            if self.status.load(Ordering::Relaxed) != 1 {
                if ui.button("Download").clicked() {
                    self.url_status = playlist_check(&self.link);
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
                    let res = self.res;
                    let yt_dlp = depen.yt_dlp.clone();

                    tokio::task::spawn(async move {
                        let status = ytdlp::video_download(
                            link, directory, format, frags, subtile, &lang, auto_gen, cook,
                            use_cook, res, yt_dlp,
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
            }
        });
    }
}
