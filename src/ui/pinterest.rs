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
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Status: ");
            if self.status_complete.load(Ordering::Relaxed) {
                ui.colored_label(Color32::GREEN, "Done!");
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
                self.reset_download_status();
                self.start_download_status();

                let link = self.link.clone();
                let directory = self.out_directory.clone();
                let complete = self.status_complete.clone();
                let doing = self.status_pending.clone();
                let videoornot = self.imgoranime.clone();

                tokio::task::spawn(async move {
                    download(link, directory, videoornot).await;
                    complete.store(true, Ordering::Relaxed);
                    doing.store(false, Ordering::Relaxed);
                });
            }
        });
    }
}

async fn download(link: String, directory: String, videoorimg: bool) {
    if videoorimg {
        let output = Command::new("yt-dlp")
            .arg(&link)
            .current_dir(&directory)
            .output()
            .expect("Something");

        let log = String::from_utf8(output.stdout).unwrap_or_else(|_| "Life suck".to_string());
        println!("{log}");
    } else if !videoorimg {
        let _ = pin_pic_dl(&link, &directory);
    }
}

fn pin_pic_dl(link: &String, directory: &String) -> Result<(), Box<dyn Error>> {
    let body = ureq::get(link)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36")
        .call()?
        .body_mut()
        .read_to_string()?;
    let doc = Html::parse_document(&body);
    let selector = Selector::parse("img").unwrap();

    if let Some(first_image) = doc.select(&selector).next() {
        if let Some(src) = first_image.value().attr("src") {
            println!("First image URL: {}", src);
            let filename = src.split("/").last().unwrap();

            let response = get(src).call().expect("Failed to download image");

            let (_, body) = response.into_parts();

            let mut file =
                File::create(Path::new(directory).join(&filename)).expect("Failed to create file");
            copy(&mut body.into_reader(), &mut file).expect("Failed to save image");

            println!("Image downloaded successfully: {}", filename);
        } else {
            println!("The first image does not have a 'src' attribute.");
        }
    } else {
        println!("No images found in the document.");
    }
    Ok(())
}
