use eframe::egui::{Color32, Ui};

use crate::app::cores::url_checker::UrlStatus;

pub fn show(ui: &mut Ui, status: &UrlStatus) {
    match status {
        UrlStatus::Playlist => {
            ui_part(ui, "playlist");
        }
        UrlStatus::Radio => {
            ui_part(ui, "radio");
        }
        UrlStatus::Single => {
            ui_part(ui, "single");
        }
        UrlStatus::None => {
            ui_part(ui, "none");
        }
    }
}

fn ui_part(ui: &mut Ui, name: &str) {
    ui.horizontal(|ui| {
        ui.label("Type: ");
        ui.spacing();
        ui.colored_label(Color32::LIGHT_BLUE, name);
    });
}
