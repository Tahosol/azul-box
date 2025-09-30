#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;

use crate::app::shares::config::config_file_default;
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
    ffmpeg: bool,
    pin: bool,
}

impl Default for MainApp {
    fn default() -> Self {
        Self {
            music_download: app::music_dl::MusicDownload::default(),
            video_download: app::video_dl::VideoDownload::default(),
            pinterest_download: app::pinterest::PinterstDownload::default(),
            image_convert: app::img_convert::ImgConvert::default(),
            video_convert: app::video_convert::VideoConvert::default(),
            run_on_start: false,
            yt: true,
            ffmpeg: false,
            pin: false,
        }
    }
}

impl eframe::App for MainApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut style = (*ctx.style()).clone();
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
                });
            });
        });

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
                .show(ctx, |ui| self.music_download.ui(ui));
            //Video
            egui::Window::new("Video-dl")
                .default_open(false)
                .resizable(false)
                .show(ctx, |ui| {
                    self.video_download.ui(ui);
                });
        }
        if self.pin {
            //Pinterest
            egui::Window::new("Pinterest-dl")
                .default_open(false)
                .resizable(false)
                .show(ctx, |ui| {
                    self.pinterest_download.ui(ui);
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
    }
}
