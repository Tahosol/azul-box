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
                println!("Saved default config")
            }
            Err(e) => {
                eprintln!("Fail to save default config {e}")
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
            return Ok(configs);
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub universal: Universal,
    pub video_dl: VideoDl,
    pub music_dl: MusicDl,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Universal {
    pub language: String,
    pub use_cookies: bool,
    pub cookies: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct VideoDl {
    pub format: i8,
    pub subtitle: bool,
    pub auto_gen_sub: bool,
    pub fragments: i8,
    pub resolution: i32,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct MusicDl {
    pub format: i8,
    pub lyrics: bool,
    pub auto_gen_sub: bool,
    pub liblrc: bool,
    pub kugou_lyrics: bool,
    pub musicbrainz: bool,
    pub threshold: i8,
    pub fragments: i8,
    pub crop_cover: bool,
    pub use_playlist_cover: bool,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            universal: Universal {
                language: "en".to_string(),
                use_cookies: false,
                cookies: None,
            },
            video_dl: VideoDl {
                format: 1,
                subtitle: true,
                auto_gen_sub: false,
                fragments: 1,
                resolution: 1080,
            },
            music_dl: MusicDl {
                format: 1,
                lyrics: true,
                auto_gen_sub: false,
                liblrc: false,
                kugou_lyrics: false,
                musicbrainz: false,
                threshold: 90,
                fragments: 1,
                crop_cover: true,
                use_playlist_cover: true,
            },
        }
    }
}
