use crate::app::cores::depen_manager::Depen;

use crate::app::cores::notify::{button_sound, done_sound, fail_sound};
use eframe::egui::{self, Color32};
use native_dialog::DialogBuilder;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicI8, Ordering};
use std::sync::{Arc, Mutex};

static IMAGE_FORMAT: &[&str; 8] = &["png", "bmp", "tif", "gif", "webp", "heic", "jpg", "avif"];
static AUDIO_FORMAT: &[&str; 9] = &[
    "mp3", "wav", "aac", "flac", "ogg", "m4a", "wma", "opus", "aiff",
];
static VIDEO_FORMAT: &[&str; 17] = &[
    "mp4", "avi", "mkv", "mov", "wmv", "flv", "webm", "mpg", "3gp", "ogv", "m4v", "asf", "vob",
    "ts", "ogg", "webp", "gif",
];
pub struct Ffmpeg {
    pub out_directory: String,
    pub status: Arc<AtomicI8>,
    pub format_in: String,
    pub format_out: String,
    pub input_file: String,
    error_message: Arc<Mutex<String>>,
}

impl Default for Ffmpeg {
    fn default() -> Self {
        let default_directory = dirs::picture_dir()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_else(|| String::from(""));
        Self {
            input_file: String::new(),
            out_directory: default_directory,
            status: Arc::new(AtomicI8::new(0)), // 0 = nothing / 1 = pending / 2 = Done / 3 = Fail
            format_in: String::new(),
            format_out: String::from("None"),
            error_message: Arc::new(Mutex::new(String::new())),
        }
    }
}

impl Ffmpeg {
    fn start_download_status(&mut self) {
        self.status.store(1, Ordering::Relaxed);
    }
    fn format_out_button(&mut self, ui: &mut egui::Ui, name: &str) {
        if self.format_out == name {
            if ui
                .add(egui::Button::new(
                    egui::RichText::new(name).color(Color32::LIGHT_BLUE),
                ))
                .clicked()
            {
                self.format_out = name.to_string();
            };
        } else {
            if ui.button(name).clicked() {
                self.format_out = name.to_string();
            };
        }
    }
    pub fn ui(&mut self, ui: &mut egui::Ui, depen: &Depen) {
        ui.horizontal(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("Output: ");
                ui.menu_button(self.format_out.clone(), |ui| {
                    if AUDIO_FORMAT.contains(&self.format_in.as_str()) {
                        for format in AUDIO_FORMAT {
                            self.format_out_button(ui, format);
                        }
                    } else if VIDEO_FORMAT.contains(&self.format_in.as_str()) {
                        for format in VIDEO_FORMAT {
                            self.format_out_button(ui, format);
                        }
                    } else if IMAGE_FORMAT.contains(&self.format_in.as_str()) {
                        for format in IMAGE_FORMAT {
                            self.format_out_button(ui, format);
                        }
                    } else {
                        self.format_out_button(ui, "Nothing");
                    }
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
            let link_label = ui.label("Media: ");
            if ui
                .text_edit_singleline(&mut self.input_file)
                .labelled_by(link_label.id)
                .clicked()
            {
                let mut filter = Vec::with_capacity(
                    AUDIO_FORMAT.len() + VIDEO_FORMAT.len() + IMAGE_FORMAT.len(),
                );
                filter.extend_from_slice(AUDIO_FORMAT);
                filter.extend_from_slice(VIDEO_FORMAT);
                filter.extend_from_slice(IMAGE_FORMAT);
                let path = DialogBuilder::file()
                    .set_location(&self.out_directory)
                    .add_filter("Media", filter)
                    .open_single_file()
                    .show()
                    .unwrap();

                if let Some(p) = path {
                    self.input_file = p.to_string_lossy().into_owned();
                    if let Some(out_path) = p.parent().and_then(|x| x.to_str()) {
                        self.out_directory = out_path.to_string();
                    }
                    let input = self.input_file.split(".").last().unwrap();
                    self.format_in = input.to_string();
                } else {
                    log::info!("No file selected.");
                }
                self.format_out = "None".to_string();
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
                    log::info!("No file selected.");
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
                    let ffmpeg = depen.ffmpeg.clone();
                    let error_message_clone = Arc::clone(&self.error_message);

                    tokio::task::spawn(async move {
                        match ffmpeg_cli(input, directory, format_out, ffmpeg) {
                            Ok(_) => {
                                progress.store(2, Ordering::Relaxed);
                                let _ = done_sound();
                            }
                            Err(e) => {
                                *error_message_clone.lock().unwrap() = e.to_string();
                                progress.store(3, Ordering::Relaxed);
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
                            .args(&["/IM", "ffmpeg.exe", "/F"])
                            .output();
                    }
                    #[cfg(target_os = "linux")]
                    {
                        let _ = button_sound();
                        let _ = Command::new("pkill").arg("ffmpeg").output();
                    }
                }
            }
            if self.status.load(Ordering::Relaxed) == 3 {
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

fn ffmpeg_cli(
    input: String,
    directory: String,
    format_out: String,
    ffmpeg: Option<PathBuf>,
) -> Result<(), Box<dyn Error>> {
    if input.is_empty() {
        return Err("No input".into());
    }
    let filename = Path::new(&input)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or("Failed to extract filename stem")?;
    let filename = format!("{}-azul-ffmpeg", filename);

    let output = match ffmpeg {
        Some(f) => Command::new(f)
            .arg("-i")
            .arg(&input)
            .arg("-q:v")
            .arg("100")
            .arg(format!("{}.{}", filename, format_out))
            .current_dir(directory)
            .output()?,
        None => Command::new("ffmpeg")
            .arg("-i")
            .arg(&input)
            .arg("-q:v")
            .arg("100")
            .arg(format!("{}.{}", filename, format_out))
            .current_dir(directory)
            .output()?,
    };

    let status = output.status;

    if status.success() {
        Ok(())
    } else {
        log::error!("{}", String::from_utf8_lossy(&output.stderr));
        Err(String::from_utf8_lossy(&output.stderr).into())
    }
}
