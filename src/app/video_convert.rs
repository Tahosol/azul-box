use eframe::egui::{self, Color32};
use native_dialog::DialogBuilder;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicI8, Ordering};

use crate::app::shares::notify::{
    button_sound, done_sound, fail_sound, notification_done, notification_fail,
};

pub struct VideoConvert {
    pub out_directory: String,
    pub status: Arc<AtomicI8>,
    pub format_in: String,
    pub format_out: String,
    pub input_file: String,
}

impl Default for VideoConvert {
    fn default() -> Self {
        let default_directory = dirs::video_dir()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_else(|| String::from(""));
        Self {
            input_file: String::new(),
            out_directory: default_directory,
            status: Arc::new(AtomicI8::new(0)), // 0 = nothing / 1 = pending / 2 = Done / 3 = Fail
            format_in: String::new(),
            format_out: String::from("None"),
        }
    }
}

impl VideoConvert {
    fn start_download_status(&mut self) {
        self.status.store(1, Ordering::Relaxed);
    }
    fn format_out_button(&mut self, ui: &mut egui::Ui, name: &str) {
        if self.format_out == name && self.format_in != name {
            if ui
                .add(egui::Button::new(
                    egui::RichText::new(name).color(Color32::LIGHT_BLUE),
                ))
                .clicked()
            {
                self.format_out = name.to_string();
                ui.close();
            };
        } else if self.format_in != name {
            if ui.button(name).clicked() {
                self.format_out = name.to_string();
                ui.close();
            };
        }
    }
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("Output: ");
                ui.menu_button(self.format_out.clone(), |ui| {
                    self.format_out_button(ui, "mp4");
                    self.format_out_button(ui, "avi");
                    self.format_out_button(ui, "mkv");
                    self.format_out_button(ui, "mov");
                    self.format_out_button(ui, "wmv");
                    self.format_out_button(ui, "flv");
                    self.format_out_button(ui, "webm");
                    self.format_out_button(ui, "mpg");
                    self.format_out_button(ui, "3gp");
                    self.format_out_button(ui, "ogv");
                    self.format_out_button(ui, "m4v");
                    self.format_out_button(ui, "asf");
                    self.format_out_button(ui, "vob");
                    self.format_out_button(ui, "ts");
                    self.format_out_button(ui, "f4v");
                    self.format_out_button(ui, "dv");
                    self.format_out_button(ui, "gif");
                });
            });
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);
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
            let link_label = ui.label("Video: ");
            if ui
                .text_edit_singleline(&mut self.input_file)
                .labelled_by(link_label.id)
                .clicked()
            {
                let path = DialogBuilder::file()
                    .set_location(&self.out_directory)
                    .add_filter(
                        "Video",
                        [
                            "mp4", "avi", "mkv", "mov", "wmv", "flv", "webm", "mpeg", "mpg", "3gp",
                            "ogv", "m4v", "asf", "vob", "ts", "f4v", "dv",
                        ],
                    )
                    .open_single_file()
                    .show()
                    .unwrap();

                if let Some(p) = path {
                    self.input_file = p.to_string_lossy().into_owned();
                    let input = self.input_file.split(".").last().unwrap();
                    self.format_in = input.to_string();
                } else {
                    println!("No file selected.");
                }
            }

            let dir_label = ui.label("Output Directory: ");
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
                if ui.button("Convert").clicked() {
                    let _ = button_sound();
                    self.start_download_status();

                    let input = self.input_file.clone();
                    let directory = self.out_directory.clone();
                    let format_out = self.format_out.clone();
                    let progress = self.status.clone();

                    tokio::task::spawn(async move {
                        let status = download(input, directory, format_out);
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
                    let _ = Command::new("pkill").arg("ffmpeg").output();
                }
            }
        });
    }
}

fn download(input: String, directory: String, format_out: String) -> i8 {
    let filename = input.split("/").last().unwrap().split(".").next().unwrap();

    let output = Command::new("ffmpeg")
        .arg("-i")
        .arg(&input)
        .arg(format!("{}.{}", filename, format_out))
        .current_dir(directory)
        .output()
        .expect("Failed to execute command");

    let status = output.status;

    println!("{status}");
    let status: i8 = if status.success() { 2 } else { 3 };
    if status == 2 {
        let _ = notification_done("video converter");
    } else {
        let _ = notification_fail("video converter");
    }
    status
}
