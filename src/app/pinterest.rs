use eframe::egui::{self, Color32};
use native_dialog::DialogBuilder;
use scraper::{Html, Selector};
use std::error::Error;
use std::fs::File;
use std::io::copy;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio;
use ureq::get;

use crate::USERAGENT;
use crate::app::cores::depen_manager::Depen;
use crate::app::cores::notify::{button_sound, done_sound, fail_sound, notification_done};

pub struct PinterstDownload {
    pub link: String,
    pub out_directory: String,
    pub status_complete: Arc<AtomicBool>,
    pub status_pending: Arc<AtomicBool>,
    pub imgoranime: bool,
}

impl Default for PinterstDownload {
    fn default() -> Self {
        let default_directory = dirs::picture_dir()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_else(|| String::from(""));
        Self {
            link: String::new(),
            out_directory: default_directory,
            status_complete: Arc::new(AtomicBool::new(false)),
            status_pending: Arc::new(AtomicBool::new(false)),
            imgoranime: false,
        }
    }
}

impl PinterstDownload {
    fn reset_download_status(&mut self) {
        self.status_complete.store(false, Ordering::Relaxed);
        self.status_pending.store(false, Ordering::Relaxed);
    }
    fn start_download_status(&mut self) {
        self.status_pending.store(true, Ordering::Relaxed);
    }
    pub fn ui(&mut self, ui: &mut egui::Ui, depen: &Depen) {
        ui.horizontal(|ui| {
            ui.label("Status: ");
            if self.status_complete.load(Ordering::Relaxed) {
                ui.colored_label(Color32::LIGHT_GREEN, "Done!");
            } else if self.status_pending.load(Ordering::Relaxed) {
                ui.spinner();
            }
            ui.add_space(20.0);
            ui.separator();
            ui.add_space(20.0);
            ui.checkbox(&mut self.imgoranime, "Video");
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

            if ui.button("Download").clicked() {
                let _ = button_sound();
                if !self.status_pending.load(Ordering::Relaxed) {
                    self.reset_download_status();
                    self.start_download_status();

                    let link = self.link.clone();
                    let directory = self.out_directory.clone();
                    let complete = self.status_complete.clone();
                    let doing = self.status_pending.clone();
                    let videoornot = self.imgoranime;
                    let yt_dlp_path = depen.yt_dlp.clone();

                    tokio::task::spawn(async move {
                        match download(link, directory, videoornot, &yt_dlp_path) {
                            Ok(_) => {
                                complete.store(true, Ordering::Relaxed);
                                doing.store(false, Ordering::Relaxed);
                                let _ = done_sound();
                            }
                            Err(e) => {
                                println!("Fail pinterest dl: {e}");
                                complete.store(false, Ordering::Relaxed);
                                doing.store(false, Ordering::Relaxed);
                                let _ = fail_sound();
                            }
                        }
                    });
                }
            }
        });
    }
}

fn download(
    link: String,
    directory: String,
    videoorimg: bool,
    yt_dlp_path: &Path,
) -> Result<(), Box<dyn Error>> {
    if videoorimg {
        let output = Command::new(yt_dlp_path)
            .arg(&link)
            .current_dir(&directory)
            .output()?;

        let log = String::from_utf8(output.stdout).unwrap_or_else(|_| "Life suck".to_string());
        println!("{log}");
        let _ = notification_done("pinterest downloader");
    } else if !videoorimg {
        let _ = pin_pic_dl(&link, &directory);
    }
    Ok(())
}

fn pin_pic_dl(link: &String, directory: &String) -> Result<(), Box<dyn Error>> {
    let body = ureq::get(link)
        .header("User-Agent", USERAGENT)
        .call()?
        .body_mut()
        .read_to_string()?;
    let doc = Html::parse_document(&body);
    let selector = Selector::parse("img").unwrap();

    if let Some(first_image) = doc.select(&selector).next() {
        if let Some(src) = first_image.value().attr("src") {
            println!("First image URL: {}", src);
            let filename = src.split("/").last().unwrap();

            let response = get(src).call()?;

            let (_, body) = response.into_parts();

            let mut file = File::create(Path::new(directory).join(filename))?;
            copy(&mut body.into_reader(), &mut file)?;

            println!("Image downloaded successfully: {}", filename);
        } else {
            println!("The first image does not have a 'src' attribute.");
        }
    } else {
        println!("No images found in the document.");
    }
    let _ = notification_done("pinterest downloader");
    Ok(())
}
