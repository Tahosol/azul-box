use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn config_file_default() {
    let azul_conf = "AzulBox";
    let azul_conf_file = "config.toml";
    let config_dir = dirs::config_dir().unwrap();
    let azul_conf_dir = config_dir.join(azul_conf);
    if !azul_conf_dir.exists() {
        let _ = fs::create_dir(&azul_conf_dir);
    }
    let azul_conf_file_with_dir = azul_conf_dir.join(azul_conf_file);
    if !azul_conf_file_with_dir.exists() {
        let contents: Config = Config::default();
        match save_config(&contents, &azul_conf_file_with_dir) {
            Ok(_) => {
                log::info!("Saved default config")
            }
            Err(e) => {
                log::error!("Fail to save default config: {e}")
            }
        }
    }
}
pub fn get_config_file_path() -> PathBuf {
    let azul_conf = "AzulBox";
    let azul_conf_file = "config.toml";
    let config_dir = dirs::config_dir().expect("Could not find config directory");
    config_dir.join(azul_conf).join(azul_conf_file)
}

fn save_config(config: &Config, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let toml_string = toml::to_string(config)?;
    fs::write(path, toml_string)?;
    Ok(())
}
pub fn load_config(path: &Path) -> Result<Config, Box<dyn std::error::Error>> {
    let toml_string = fs::read_to_string(path)?;
    let config = toml::from_str::<Config>(&toml_string);
    match config {
        Ok(mut configs) => {
            let con = configs.repair();
            match save_config(con, &get_config_file_path()) {
                Ok(_) => log::info!("Saved config repaired"),
                Err(e) => log::error!("{}", e),
            }
            return Ok(con.clone());
        }
        Err(_) => {
            if get_config_file_path().exists() {
                fs::remove_file(get_config_file_path())?;
                config_file_default();
                return Ok(Config::default());
            }
        }
    };
    Err("Fail to fix and read config file".into())
}
pub fn modifier_config<F>(path: &Path, modify_fn: F) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnOnce(&mut Config),
{
    let mut config = load_config(path)?;
    modify_fn(&mut config);
    save_config(&config, path)?;
    Ok(())
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub universal: Universal,
    pub video_dl: VideoDl,
    pub music_dl: MusicDl,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Universal {
    pub language: Option<String>,
    pub use_cookies: Option<bool>,
    pub cookies: Option<String>,
    pub faq: Option<bool>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VideoDl {
    pub format: Option<i8>,
    pub subtitle: Option<bool>,
    pub auto_gen_sub: Option<bool>,
    pub fragments: Option<i8>,
    pub resolution: Option<i32>,
    pub disable_radio: Option<bool>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MusicDl {
    pub format: Option<i8>,
    pub lyrics: Option<bool>,
    pub auto_gen_sub: Option<bool>,
    pub liblrc: Option<bool>,
    pub kugou_lyrics: Option<bool>,
    pub musicbrainz: Option<bool>,
    pub threshold: Option<i8>,
    pub fragments: Option<i8>,
    pub crop_cover: Option<bool>,
    pub use_playlist_cover: Option<bool>,
    pub disable_radio: Option<bool>,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            universal: Universal {
                language: Some("en".to_string()),
                use_cookies: Some(false),
                cookies: None,
                faq: None,
            },
            video_dl: VideoDl {
                format: Some(1),
                subtitle: Some(true),
                auto_gen_sub: Some(false),
                fragments: Some(1),
                resolution: Some(1080),
                disable_radio: Some(true),
            },
            music_dl: MusicDl {
                format: Some(1),
                lyrics: Some(true),
                auto_gen_sub: Some(false),
                liblrc: Some(false),
                kugou_lyrics: Some(false),
                musicbrainz: Some(false),
                threshold: Some(90),
                fragments: Some(1),
                crop_cover: Some(true),
                use_playlist_cover: Some(true),
                disable_radio: Some(true),
            },
        }
    }
}

impl Config {
    fn repair(&mut self) -> &mut Config {
        if self.universal.language.is_none() {
            self.universal.language = Config::default().universal.language;
        }
        if self.universal.use_cookies.is_none() {
            self.universal.use_cookies = Config::default().universal.use_cookies;
        }
        if self.video_dl.format.is_none() {
            self.video_dl.format = Config::default().video_dl.format;
        }
        if self.video_dl.disable_radio.is_none() {
            self.video_dl.disable_radio = Config::default().video_dl.disable_radio;
        }
        if self.video_dl.subtitle.is_none() {
            self.video_dl.subtitle = Config::default().video_dl.subtitle;
        }
        if self.video_dl.auto_gen_sub.is_none() {
            self.video_dl.auto_gen_sub = Config::default().video_dl.auto_gen_sub;
        }
        if self.video_dl.fragments.is_none() {
            self.video_dl.fragments = Config::default().video_dl.fragments;
        }
        if self.video_dl.resolution.is_none() {
            self.video_dl.resolution = Config::default().video_dl.resolution;
        }
        if self.music_dl.format.is_none() {
            self.music_dl.format = Config::default().music_dl.format;
        }
        if self.music_dl.lyrics.is_none() {
            self.music_dl.lyrics = Config::default().music_dl.lyrics;
        }
        if self.music_dl.auto_gen_sub.is_none() {
            self.music_dl.auto_gen_sub = Config::default().music_dl.auto_gen_sub;
        }
        if self.music_dl.liblrc.is_none() {
            self.music_dl.liblrc = Config::default().music_dl.liblrc;
        }
        if self.music_dl.kugou_lyrics.is_none() {
            self.music_dl.kugou_lyrics = Config::default().music_dl.kugou_lyrics;
        }
        if self.music_dl.musicbrainz.is_none() {
            self.music_dl.musicbrainz = Config::default().music_dl.musicbrainz;
        }
        if self.music_dl.threshold.is_none() {
            self.music_dl.threshold = Config::default().music_dl.threshold;
        }
        if self.music_dl.fragments.is_none() {
            self.music_dl.fragments = Config::default().music_dl.fragments;
        }
        if self.music_dl.crop_cover.is_none() {
            self.music_dl.crop_cover = Config::default().music_dl.crop_cover;
        }
        if self.music_dl.use_playlist_cover.is_none() {
            self.music_dl.use_playlist_cover = Config::default().music_dl.use_playlist_cover;
        }
        if self.music_dl.disable_radio.is_none() {
            self.music_dl.disable_radio = Config::default().music_dl.disable_radio;
        }
        self
    }
}

pub fn get_log_path() -> PathBuf {
    let data = dirs::data_local_dir().unwrap().join("azulbox").join("logs");
    if !data.exists() {
        let _ = fs::create_dir_all(&data);
    }
    data
}
