mod app;
pub const USERAGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36";
pub const OS: &str = std::env::consts::OS;

use std::{
    fs,
    sync::{Arc, atomic::AtomicBool},
};

use crate::app::cores::{
    config::config_file_default,
    depen_manager::{Depen, get_path, install},
    ytdlp,
};
use eframe::egui::{self, IconData, global_theme_preference_buttons};
#[tokio::main]
async fn main() -> eframe::Result {
    let icon = include_bytes!("../assets/logo.png").to_vec();
    let icon = IconData {
        rgba: icon,
        width: 32,
        height: 32,
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_icon(icon),
        ..Default::default()
    };
    eframe::run_native(
        "Azul box",
        options,
        Box::new(|_cc| Ok(Box::<MainApp>::default())),
    )
}

struct MainApp {
    music_download: app::music_dl::MusicDownload,
    video_download: app::video_dl::VideoDownload,
    pinterest_download: app::pinterest::PinterstDownload,
    image_convert: app::img_convert::ImgConvert,
    video_convert: app::video_convert::VideoConvert,
    run_on_start: bool,
    yt: bool,
    yt_version: String,
    ffmpeg: bool,
    pin: bool,
    app_data: Depen,
    is_install_depen: Arc<AtomicBool>,
}

impl Default for MainApp {
    fn default() -> Self {
        let app_data = get_path();
        let yt_version = match ytdlp::version_check(&app_data) {
            Some(version) => version,
            None => "Missing".to_string(),
        };
        Self {
            music_download: app::music_dl::MusicDownload::default(),
            video_download: app::video_dl::VideoDownload::default(),
            pinterest_download: app::pinterest::PinterstDownload::default(),
            image_convert: app::img_convert::ImgConvert::default(),
            video_convert: app::video_convert::VideoConvert::default(),
            run_on_start: false,
            yt_version: yt_version,
            yt: true,
            ffmpeg: false,
            pin: false,
            app_data: app_data,
            is_install_depen: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl eframe::App for MainApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut style = (*ctx.style()).clone();

        if let Ok(false) = self.app_data.version.try_exists()
            && !self
                .is_install_depen
                .load(std::sync::atomic::Ordering::Relaxed)
        {
            self.is_install_depen
                .store(true, std::sync::atomic::Ordering::Relaxed);
            let _ = fs::create_dir(&self.app_data.app_data);

            let progress = self.is_install_depen.clone();
            let dir = self.app_data.app_data.clone();

            tokio::task::spawn(async move {
                match install(&dir) {
                    Ok(_) => {
                        progress.store(false, std::sync::atomic::Ordering::Relaxed);
                        println!("Updated dependencies");
                    }
                    Err(e) => {
                        println!("dependencies install error {e}");
                    }
                }
            });
        }
        // println!("{:?}", self.is_install_depen);
        if !self.run_on_start {
            config_file_default();
            self.run_on_start = true;
        };

        style
            .text_styles
            .get_mut(&egui::TextStyle::Heading)
            .unwrap()
            .size = 30.0;
        style
            .text_styles
            .get_mut(&egui::TextStyle::Body)
            .unwrap()
            .size = 24.0;
        style
            .text_styles
            .get_mut(&egui::TextStyle::Button)
            .unwrap()
            .size = 24.0;

        ctx.set_style(style);
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.heading("Azul Box");
                ui.horizontal_wrapped(|ui| {
                    global_theme_preference_buttons(ui);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.menu_button("About", |ui| {
                            ui.add(egui::Label::new(
                                egui::RichText::new(format!(
                                    "azul box: {}",
                                    env!("CARGO_PKG_VERSION")
                                ))
                                .size(20.0),
                            ));
                            ui.add(egui::Label::new(
                                egui::RichText::new(format!("yt-dlp: {}", self.yt_version))
                                    .size(20.0),
                            ));
                        });

                        if ui.button("Update").clicked() {
                            if !self
                                .is_install_depen
                                .load(std::sync::atomic::Ordering::Relaxed)
                            {
                                self.is_install_depen
                                    .store(true, std::sync::atomic::Ordering::Relaxed);

                                let progress = self.is_install_depen.clone();
                                let dir = self.app_data.app_data.clone();

                                tokio::task::spawn(async move {
                                    match install(&dir) {
                                        Ok(_) => {
                                            progress
                                                .store(false, std::sync::atomic::Ordering::Relaxed);
                                            println!("Updated dependencies");
                                        }
                                        Err(e) => {
                                            println!("dependencies install error {e}");
                                        }
                                    }
                                });
                            }
                        };
                    });
                });
            });
        });
        if !self
            .is_install_depen
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            egui::CentralPanel::default().show(ctx, |ui| ui.label(""));
            egui::SidePanel::left("Panel")
                .resizable(true)
                .width_range(50.0..=100.0)
                .show(ctx, |ui| {
                    ui.add(egui::Checkbox::new(
                        &mut self.yt,
                        egui::RichText::new("yt-dl").size(17.0),
                    ));
                    ui.separator();
                    ui.add(egui::Checkbox::new(
                        &mut self.pin,
                        egui::RichText::new("pin-dl").size(17.0),
                    ));
                    ui.separator();
                    ui.add(egui::Checkbox::new(
                        &mut self.ffmpeg,
                        egui::RichText::new("ffmpeg").size(17.0),
                    ));
                    ui.separator();
                });
            if self.yt {
                //music
                egui::Window::new("Music-dl")
                    .default_open(false)
                    .resizable(false)
                    .show(ctx, |ui| self.music_download.ui(ui, &self.app_data));
                //Video
                egui::Window::new("Video-dl")
                    .default_open(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        self.video_download.ui(ui, &self.app_data);
                    });
            }
            if self.pin {
                //Pinterest
                egui::Window::new("Pinterest-dl")
                    .default_open(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        self.pinterest_download.ui(ui, &self.app_data);
                    });
            }
            if self.ffmpeg {
                //Img convert
                egui::Window::new("Image converter")
                    .default_open(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        self.image_convert.ui(ui);
                    });
                //Video convert
                egui::Window::new("Video converter")
                    .default_open(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        self.video_convert.ui(ui);
                    });
            }
        } else {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.spinner();
                ui.label("Downloading dependencies");
            });
        }
    }
}
