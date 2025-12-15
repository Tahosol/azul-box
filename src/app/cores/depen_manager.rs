use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::copy;
use std::path::PathBuf;
use std::process::Command;
use std::{error::Error, path::Path};

pub const OS: &str = std::env::consts::OS;
pub struct Depen {
    pub app_data: PathBuf,
    pub yt_dlp: PathBuf,
    pub deno: PathBuf,
    pub version: PathBuf,
    pub ffmpeg: Option<PathBuf>,
}

pub fn get_path() -> Depen {
    let data = dirs::data_local_dir().unwrap().join("azulbox");
    if OS == "linux" {
        Depen {
            app_data: data.clone(),
            yt_dlp: data.join("yt-dlp"),
            deno: data.join("deno"),
            version: data.join("version.json"),
            ffmpeg: None,
        }
    } else {
        Depen {
            app_data: data.clone(),
            yt_dlp: data.join("yt-dlp.exe"),
            deno: data.join("deno.exe"),
            version: data.join("version.json"),
            ffmpeg: Some(
                data.join("ffmpeg-master-latest-win64-gpl")
                    .join("bin")
                    .join("ffmpeg.exe"),
            ),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
struct GithubRelease {
    name: String,
    tag_name: String,
    assets: Vec<Asset>,
}
#[derive(Debug, Deserialize, Clone)]
struct Asset {
    name: String,
    browser_download_url: String,
    digest: String,
}

fn unzip(file: &Path) -> Result<(), Box<dyn Error>> {
    let file = fs::File::open(file)?;

    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => path,
            None => continue,
        };

        if file.is_dir() {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }
            let mut outfile = fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))?;
            }
        }
    }
    Ok(())
}

fn download_file(
    filename: &str,
    dir: &Path,
    url: &str,
    digest: &str,
) -> Result<(), Box<dyn Error>> {
    let response = ureq::get(url).call()?;

    let (_, body) = response.into_parts();

    let mut file = File::create(dir.join(filename))?;
    copy(&mut body.into_reader(), &mut file)?;
    if OS == "linux" {
        let sum = String::from_utf8(
            Command::new("sha256sum")
                .arg(filename)
                .current_dir(dir)
                .output()?
                .stdout,
        )?;
        if sum.split(" ").nth(0) != digest.split(":").last() {
            fs::remove_file(dir.join(filename))?;
            return Err(format!("Sha256 fail for {filename}").into());
        }
    }
    Ok(())
}

fn get_github_release(url: &str) -> Result<GithubRelease, Box<dyn Error>> {
    Ok(ureq::get(url)
        .call()?
        .body_mut()
        .read_json::<GithubRelease>()?)
}

fn yt_dlp_install(dir: &Path, github: &GithubRelease) -> Result<GithubRelease, Box<dyn Error>> {
    let file = match OS {
        "linux" => "yt-dlp",
        "windows" => "yt-dlp.exe",
        _ => return Err("Wrong OS".into()),
    };
    for asset in &github.assets {
        if asset.name == file {
            download_file(&file, dir, &asset.browser_download_url, &asset.digest)?
        }
    }
    Ok(github.clone())
}

fn ffmpeg_install(dir: &Path, github: &GithubRelease) -> Result<GithubRelease, Box<dyn Error>> {
    let (file, ext) = match OS {
        "linux" => {
            return Ok(GithubRelease {
                name: "linux".to_string(),
                tag_name: "linux".to_string(),
                assets: vec![Asset {
                    name: "linux".to_string(),
                    browser_download_url: "linux".to_string(),
                    digest: "linux".to_string(),
                }],
            });
        }
        "windows" => ("ffmpeg-master-latest-win64-gpl.zip", "exe"),
        _ => return Err("Wrong OS".into()),
    };
    for asset in &github.assets {
        if asset.name == file {
            download_file(&file, dir, &asset.browser_download_url, &asset.digest)?
        }
    }
    let zipfile = dir.join(&file);
    unzip(&zipfile)?;
    fs::remove_file(zipfile)?;
    Ok(github.clone())
}

fn deno_install(dir: &Path, github: &GithubRelease) -> Result<GithubRelease, Box<dyn Error>> {
    let file = match OS {
        "linux" => "deno-x86_64-unknown-linux-gnu.zip",
        "windows" => "deno-x86_64-pc-windows-msvc.zip",
        _ => return Err("Wrong OS".into()),
    };
    for asset in &github.assets {
        if asset.name == file {
            download_file(&file, dir, &asset.browser_download_url, &asset.digest)?
        }
    }
    let zipfile = dir.join(&file);
    unzip(&zipfile)?;
    fs::remove_file(zipfile)?;
    Ok(github.clone())
}

pub fn install(dir: &Path) -> Result<(), Box<dyn Error>> {
    let saved_data_location = dir.join("version.json");

    let yt_github =
        get_github_release("https://api.github.com/repos/yt-dlp/yt-dlp/releases/latest")?;
    let ffmpeg_github =
        get_github_release("https://api.github.com/repos/yt-dlp/FFmpeg-Builds/releases/latest")?;
    let deno_github =
        get_github_release("https://api.github.com/repos/denoland/deno/releases/latest")?;

    match fs::read_to_string(&saved_data_location) {
        Ok(data) => {
            let mut data_struct: VersionJson = serde_json::from_str(&data)?;
            if data_struct.deno != deno_github.name {
                data_struct.yt_dlp = yt_dlp_install(dir, &yt_github)?.name;
            }
            if data_struct.ffmpeg != ffmpeg_github.name {
                data_struct.ffmpeg = ffmpeg_install(dir, &ffmpeg_github)?.name;
            }
            if data_struct.yt_dlp != yt_github.name {
                data_struct.deno = deno_install(dir, &deno_github)?.name;
            }
            let datas = serde_json::to_string(&data_struct)?;
            fs::write(&saved_data_location, datas)?;
        }
        Err(_) => {
            let data = serde_json::to_string(&VersionJson {
                yt_dlp: yt_dlp_install(dir, &yt_github)?.name,
                ffmpeg: ffmpeg_install(dir, &ffmpeg_github)?.name,
                deno: deno_install(dir, &deno_github)?.name,
            })?;
            fs::write(&saved_data_location, data)?;
        }
    }
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
struct VersionJson {
    yt_dlp: String,
    ffmpeg: String,
    deno: String,
}
