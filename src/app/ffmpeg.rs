use crate::app::cores::depen_manager::Depen;

use crate::app::cores::notify::{button_sound, done_sound, fail_sound};
use eframe::egui::{self, Color32};
use rfd::FileDialog;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicI8, Ordering};
use std::sync::{Arc, Mutex};

static IMAGE_FORMAT: &[&str; 9] = &[
    "png", "bmp", "tif", "gif", "webp", "heic", "jpg", "avif", "jpeg",
];
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
    compression: FFmpegCompression,
    error_message: Arc<Mutex<String>>,
}

impl Default for Ffmpeg {
    fn default() -> Self {
        Self {
            input_file: String::new(),
            out_directory: "".to_string(),
            status: Arc::new(AtomicI8::new(0)), // 0 = nothing / 1 = pending / 2 = Done / 3 = Fail
            format_in: String::new(),
            format_out: String::from("None"),
            compression: FFmpegCompression::None,
            error_message: Arc::new(Mutex::new(String::new())),
        }
    }
}

impl Ffmpeg {
    fn start_download_status(&mut self) {
        self.status.store(1, Ordering::Relaxed);
    }
    fn toggle_compression(
        &mut self,
        ui: &mut egui::Ui,
        atoms: &str,
        compression: FFmpegCompression,
    ) {
        match self.compression {
            FFmpegCompression::PngCompression(_) => {
                if ui
                    .add(egui::Button::new(
                        egui::RichText::new(atoms).color(Color32::LIGHT_BLUE),
                    ))
                    .clicked()
                {
                    self.compression = FFmpegCompression::None;
                };
            }
            _ => {
                if ui.button(atoms).clicked() {
                    self.compression = compression;
                }
            }
        }
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
            ui.menu_button("Compression", |ui| {
                if self.format_in == "png" && self.format_out == "png" {
                    self.toggle_compression(
                        ui,
                        "Png compression",
                        FFmpegCompression::PngCompression(100),
                    );
                    if let FFmpegCompression::PngCompression(value) = &mut self.compression {
                        ui.add(
                            egui::widgets::Slider::new(value, 0..=100).text("Percentage quality: "),
                        )
                        .on_hover_text(
                            "percentage of compression from 0% to 100%, more is less file size",
                        );
                    }
                } else if self.format_out == "jpg" || self.format_out == "jpeg" {
                    self.toggle_compression(
                        ui,
                        "Jpg compression",
                        FFmpegCompression::JpgCompression(3),
                    );
                    if let FFmpegCompression::JpgCompression(value) = &mut self.compression {
                        ui.add(
                            egui::widgets::Slider::new(value, 1..=100).text("Compression level: "),
                        )
                        .on_hover_text("Bigger is more compression");
                    }
                } else if VIDEO_FORMAT.contains(&self.format_out.as_ref()) {
                    self.toggle_compression(
                        ui,
                        "libx264 compression",
                        FFmpegCompression::Libx264(23),
                    );
                    if let FFmpegCompression::Libx264(value) = &mut self.compression {
                        ui.add(
                            egui::widgets::Slider::new(value, 1..=100).text("Compression level: "),
                        )
                        .on_hover_text("Bigger is more compression");
                    }
                } else if AUDIO_FORMAT.contains(&&self.format_out.as_ref()) {
                    self.toggle_compression(
                        ui,
                        "bitrate compression",
                        FFmpegCompression::AudioBitrate(128),
                    );
                    if let FFmpegCompression::AudioBitrate(value) = &mut self.compression {
                        ui.add(
                            egui::widgets::Slider::new(value, 16..=128)
                                .text("bitrate: ")
                                .step_by(16.0),
                        )
                        .on_hover_text("smaller is lower quality and more compression");
                    }
                }
            });
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
                let path = FileDialog::new()
                    .set_directory(&self.out_directory)
                    .add_filter("Media", &filter[..])
                    .pick_file();

                if let Some(p) = path {
                    self.input_file = p.to_string_lossy().into_owned();
                    if let Some(out_path) = p.parent().and_then(|x| x.to_str()) {
                        self.out_directory = out_path.to_string();
                    }
                    let input = self.input_file.split(".").last().unwrap();
                    self.format_in = input.to_string();
                    self.format_out = input.to_string();
                } else {
                    log::info!("No file selected.");
                }
            }

            let dir_label = ui.label("Output Directory: ");
            if ui
                .text_edit_singleline(&mut self.out_directory)
                .labelled_by(dir_label.id)
                .clicked()
            {
                let path = FileDialog::new()
                    .set_directory(&self.out_directory)
                    .pick_folder();

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
                    let compression = self.compression.clone();
                    let error_message_clone = Arc::clone(&self.error_message);

                    tokio::task::spawn(async move {
                        match ffmpeg_cli(input, directory, format_out, ffmpeg, compression) {
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
            if self.status.load(Ordering::Relaxed) == 1 {
                ui.spacing();
                ui.separator();
                ui.horizontal_wrapped(|ui| {
                    ui.spinner();
                    ui.label(
                        egui::RichText::new("This may take awhile").color(Color32::LIGHT_GRAY),
                    );
                });
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
            } else if self.status.load(Ordering::Relaxed) == 2 {
                ui.colored_label(Color32::LIGHT_GREEN, "Done!");
            };
        });
    }
}

fn ffmpeg_cli(
    input: String,
    directory: String,
    format_out: String,
    ffmpeg: Option<PathBuf>,
    compression: FFmpegCompression,
) -> Result<(), Box<dyn Error>> {
    if input.is_empty() {
        return Err("No input".into());
    }
    let filename = Path::new(&input)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or("Failed to extract filename stem")?;
    let filename = format!("{}-azul-ffmpeg", filename);

    let ffmpeg_bin = match ffmpeg {
        Some(bin) => bin,
        None => "ffmpeg".into(),
    };

    let mut cli_build = Command::new(ffmpeg_bin);
    cli_build.arg("-i").arg(&input).arg("-q:v").arg("100");

    match compression {
        FFmpegCompression::Libx264(cfg) => {
            cli_build
                .arg("-vcodec")
                .arg("libx264")
                .arg("-crf")
                .arg(format!("{}", cfg));
        }
        FFmpegCompression::AudioBitrate(cfg) => {
            cli_build.arg("-b:a").arg(format!("{}k", cfg));
        }
        FFmpegCompression::JpgCompression(cfg) => {
            cli_build.arg("-q:v").arg(format!("{}", cfg));
        }
        FFmpegCompression::PngCompression(cfg) => {
            cli_build.arg("-compression_level").arg(format!("{}", cfg));
        }
        FFmpegCompression::None => {}
    }

    cli_build
        .arg(format!("{}.{}", filename, format_out))
        .current_dir(directory);
    let output = cli_build.output()?;

    if output.status.success() {
        Ok(())
    } else {
        log::error!("{}", String::from_utf8_lossy(&output.stderr));
        Err(String::from_utf8_lossy(&output.stderr).into())
    }
}

#[derive(Debug, Clone)]
enum FFmpegCompression {
    Libx264(u8),
    JpgCompression(u8),
    PngCompression(u8),
    AudioBitrate(u8),
    None,
}
