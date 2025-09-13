use crate::app::shares::config;
use crate::app::shares::config::get_config_file_path;
use eframe::egui::{self, Color32, Ui};

pub struct LangThing {}

impl LangThing {
    pub fn lang_chooser(ui: &mut Ui, mut lang_in: String) -> String {
        let language_codes: Vec<&str> = vec![
            "en", // English
            "fr", // French
            "es", // Spanish
            "zh", // Chinese
            "de", // German
            "ja", // Japanese
            "ar", // Arabic
            "ru", // Russian
            "it", // Italian
            "pt", // Portuguese
            "nl", // Dutch
            "sv", // Swedish
            "no", // Norwegian
            "fi", // Finnish
            "da", // Danish
            "pl", // Polish
            "cs", // Czech
            "hu", // Hungarian
            "ro", // Romanian
            "tr", // Turkish
            "vi", // Vietnamese
            "ko", // Korean
            "el", // Greek
            "he", // Hebrew
            "th", // Thai
            "id", // Indonesian
            "ms", // Malay
            "hi", // Hindi
            "uk", // Ukrainian
            "bg", // Bulgarian
            "hr", // Croatian
            "sk", // Slovak
            "sl", // Slovenian
            "sr", // Serbian
            "lt", // Lithuanian
            "lv", // Latvian
            "et", // Estonian
        ];
        let languages: Vec<&str> = vec![
            "English",
            "French",
            "Spanish",
            "Chinese",
            "German",
            "Japanese",
            "Arabic",
            "Russian",
            "Italian",
            "Portuguese",
            "Dutch",
            "Swedish",
            "Norwegian",
            "Finnish",
            "Danish",
            "Polish",
            "Czech",
            "Hungarian",
            "Romanian",
            "Turkish",
            "Vietnamese",
            "Korean",
            "Greek",
            "Hebrew",
            "Thai",
            "Indonesian",
            "Malay",
            "Hindi",
            "Ukrainian",
            "Bulgarian",
            "Croatian",
            "Slovak",
            "Slovenian",
            "Serbian",
            "Lithuanian",
            "Latvian",
            "Estonian",
        ];
        ui.menu_button("Languages", |ui| {
            egui::ScrollArea::vertical()
                .max_height(350.0)
                .show(ui, |ui| {
                    for (lang, code) in languages.iter().zip(language_codes.iter()) {
                        if lang_in == code.to_string() {
                            if ui
                                .add(egui::Button::new(
                                    egui::RichText::new(*lang).color(Color32::LIGHT_BLUE),
                                ))
                                .clicked()
                            {
                                lang_in = code.to_string();
                                ui.close();
                            };
                        } else {
                            if ui.button(*lang).clicked() {
                                lang_in = code.to_string();
                                let path_config = get_config_file_path();
                                println!("{path_config:?}");
                                match config::modifier_config(&path_config, |cfg| {
                                    cfg.universal.language = lang_in.clone()
                                }) {
                                    Ok(_) => {
                                        println!("Saved languages")
                                    }
                                    Err(e) => {
                                        eprintln!("Fail saved languages {e}")
                                    }
                                }
                                ui.close();
                            }
                        }
                    }
                });
        });
        lang_in
    }
}
