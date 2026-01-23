use crate::app::cores::config;
use crate::app::cores::config::get_config_file_path;
use eframe::egui::{self, Color32, Ui};

pub struct LangThing {}

impl LangThing {
    pub fn lang_chooser(ui: &mut Ui, mut lang_in: String) -> String {
        let language_codes: Vec<&str> = vec![
            "en",  // English
            "fr",  // French
            "es",  // Spanish
            "zh",  // Chinese
            "de",  // German
            "ja",  // Japanese
            "ar",  // Arabic
            "ru",  // Russian
            "it",  // Italian
            "pt",  // Portuguese
            "nl",  // Dutch
            "sv",  // Swedish
            "no",  // Norwegian
            "fi",  // Finnish
            "da",  // Danish
            "pl",  // Polish
            "cs",  // Czech
            "hu",  // Hungarian
            "ro",  // Romanian
            "tr",  // Turkish
            "vi",  // Vietnamese
            "ko",  // Korean
            "el",  // Greek
            "he",  // Hebrew
            "th",  // Thai
            "id",  // Indonesian
            "ms",  // Malay
            "hi",  // Hindi
            "uk",  // Ukrainian
            "bg",  // Bulgarian
            "hr",  // Croatian
            "sk",  // Slovak
            "sl",  // Slovenian
            "sr",  // Serbian
            "lt",  // Lithuanian
            "lv",  // Latvian
            "et",  // Estonian
            "fa",  // Persian
            "sw",  // Swahili
            "ta",  // Tamil
            "bn",  // Bengali
            "ur",  // Urdu
            "gu",  // Gujarati
            "pa",  // Punjabi
            "te",  // Telugu
            "ml",  // Malayalam
            "kn",  // Kannada
            "mr",  // Marathi
            "eu",  // Basque
            "gl",  // Galician
            "ca",  // Catalan
            "is",  // Icelandic
            "mt",  // Maltese
            "sq",  // Albanian
            "mk",  // Macedonian
            "az",  // Azerbaijani
            "hy",  // Armenian
            "ka",  // Georgian
            "af",  // Afrikaans
            "zu",  // Zulu
            "xh",  // Xhosa
            "yo",  // Yoruba
            "ig",  // Igbo
            "am",  // Amharic
            "my",  // Burmese
            "km",  // Khmer
            "lo",  // Lao
            "mn",  // Mongolian
            "si",  // Sinhala
            "ne",  // Nepali
            "jw",  // Javanese
            "su",  // Sundanese
            "fil", // Filipino
            "tl",  // Tagalog
            "ceb", // Cebuano
            "haw", // Hawaiian
            "sm",  // Samoan
            "fj",  // Fijian
            "to",  // Tongan
            "ht",  // Haitian Creole
            "la",  // Latin
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
            "Persian",
            "Swahili",
            "Tamil",
            "Bengali",
            "Urdu",
            "Gujarati",
            "Punjabi",
            "Telugu",
            "Malayalam",
            "Kannada",
            "Marathi",
            "Basque",
            "Galician",
            "Catalan",
            "Icelandic",
            "Maltese",
            "Albanian",
            "Macedonian",
            "Azerbaijani",
            "Armenian",
            "Georgian",
            "Afrikaans",
            "Zulu",
            "Xhosa",
            "Yoruba",
            "Igbo",
            "Amharic",
            "Burmese",
            "Khmer",
            "Lao",
            "Mongolian",
            "Sinhala",
            "Nepali",
            "Javanese",
            "Sundanese",
            "Filipino",
            "Tagalog",
            "Cebuano",
            "Hawaiian",
            "Samoan",
            "Fijian",
            "Tongan",
            "Haitian Creole",
            "Latin",
        ];
        ui.menu_button("Languages", |ui| {
            egui::ScrollArea::vertical()
                .max_height(350.0)
                .show(ui, |ui| {
                    for (lang, code) in languages.iter().zip(language_codes.iter()) {
                        if lang_in == *code {
                            if ui
                                .add(egui::Button::new(
                                    egui::RichText::new(*lang).color(Color32::LIGHT_BLUE),
                                ))
                                .clicked()
                            {
                                lang_in = code.to_string();
                            };
                        } else {
                            if ui.button(*lang).clicked() {
                                lang_in = code.to_string();
                                let path_config = get_config_file_path();
                                log::info!("{path_config:?}");
                                match config::modifier_config(&path_config, |cfg| {
                                    cfg.universal.language = Some(lang_in.clone())
                                }) {
                                    Ok(_) => {
                                        log::info!("Saved languages")
                                    }
                                    Err(e) => {
                                        log::error!("Fail saved languages {e}")
                                    }
                                }
                            }
                        }
                    }
                });
        });
        lang_in
    }
}
