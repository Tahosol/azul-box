#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod ui;

use crate::ui::shares::config::config_file_default;
use eframe::egui::{self, IconData, RichText, global_theme_preference_buttons};
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
        "azul_box",
        options,
        Box::new(|_cc| {
            // egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::<MainApp>::default())
        }),
    )
}

struct MainApp {
    music_download: ui::music_dl::MusicDownload,
    video_download: ui::video_dl::VideoDownload,
    pinterest_download: ui::pinterest::PinterstDownload,
    image_convert: ui::img_convert::ImgConvert,
    video_convert: ui::video_convert::VideoConvert,
    run_on_start: bool,
    yt: bool,
    ffmpeg: bool,
    pin: bool,
}

impl Default for MainApp {
    fn default() -> Self {
        Self {
            music_download: ui::music_dl::MusicDownload::default(),
            video_download: ui::video_dl::VideoDownload::default(),
            pinterest_download: ui::pinterest::PinterstDownload::default(),
            image_convert: ui::img_convert::ImgConvert::default(),
            video_convert: ui::video_convert::VideoConvert::default(),
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
            // self.check_result = version_check::version_check();
            self.run_on_start = true;
        };

        style
            .text_styles
            .get_mut(&egui::TextStyle::Heading)
            .unwrap()
            .size = 26.0;
        style
            .text_styles
            .get_mut(&egui::TextStyle::Body)
            .unwrap()
            .size = 20.0;
        style
            .text_styles
            .get_mut(&egui::TextStyle::Button)
            .unwrap()
            .size = 20.0;

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
            .width_range(27.0..=90.0)
            .show(ctx, |ui| {
                ui.label(RichText::new("Yt-dlp:").size(17.0));
                ui.add(egui::Checkbox::without_text(&mut self.yt));
                ui.separator();
                ui.label(RichText::new("Pinterest:").size(17.0));
                ui.add(egui::Checkbox::without_text(&mut self.pin));
                ui.separator();
                ui.label(RichText::new("Ffmpeg:").size(17.0));
                ui.add(egui::Checkbox::without_text(&mut self.ffmpeg));
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
