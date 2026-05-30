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
        Ok(configs) => {
            let con = configs.repair();
            match save_config(&con, &get_config_file_path()) {
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
    pub keep_lrc: Option<bool>,
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
                keep_lrc: Some(false),
            },
        }
    }
}

impl Config {
    fn repair(mut self) -> Self {
        let default = Config::default();

        self.universal.language = self.universal.language.or(default.universal.language);

        self.universal.use_cookies = self.universal.use_cookies.or(default.universal.use_cookies);

        self.video_dl.format = self.video_dl.format.or(default.video_dl.format);

        self.video_dl.disable_radio = self
            .video_dl
            .disable_radio
            .or(default.video_dl.disable_radio);

        self.video_dl.subtitle = self.video_dl.subtitle.or(default.video_dl.subtitle);

        self.video_dl.auto_gen_sub = self.video_dl.auto_gen_sub.or(default.video_dl.auto_gen_sub);

        self.video_dl.fragments = self.video_dl.fragments.or(default.video_dl.fragments);

        self.video_dl.resolution = self.video_dl.resolution.or(default.video_dl.resolution);

        self.music_dl.format = self.music_dl.format.or(default.music_dl.format);

        self.music_dl.lyrics = self.music_dl.lyrics.or(default.music_dl.lyrics);

        self.music_dl.auto_gen_sub = self.music_dl.auto_gen_sub.or(default.music_dl.auto_gen_sub);

        self.music_dl.liblrc = self.music_dl.liblrc.or(default.music_dl.liblrc);

        self.music_dl.kugou_lyrics = self.music_dl.kugou_lyrics.or(default.music_dl.kugou_lyrics);

        self.music_dl.musicbrainz = self.music_dl.musicbrainz.or(default.music_dl.musicbrainz);

        self.music_dl.threshold = self.music_dl.threshold.or(default.music_dl.threshold);

        self.music_dl.fragments = self.music_dl.fragments.or(default.music_dl.fragments);

        self.music_dl.crop_cover = self.music_dl.crop_cover.or(default.music_dl.crop_cover);

        self.music_dl.use_playlist_cover = self
            .music_dl
            .use_playlist_cover
            .or(default.music_dl.use_playlist_cover);

        self.music_dl.disable_radio = self
            .music_dl
            .disable_radio
            .or(default.music_dl.disable_radio);

        self.music_dl.keep_lrc = self.music_dl.disable_radio.or(default.music_dl.keep_lrc);

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
