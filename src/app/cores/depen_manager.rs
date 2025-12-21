use serde::{Deserialize, Serialize};
use std::env::consts::OS;
use std::fs::{self, File};
use std::io::copy;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::{error::Error, path::Path};

use crate::USERAGENT;

pub struct Depen {
    pub app_data: PathBuf,
    pub yt_dlp: PathBuf,
    #[allow(dead_code)]
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
            ffmpeg: Some(data.join("ffmpeg.exe")),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
struct GithubRelease {
    name: String,
    assets: Vec<Asset>,
}
#[derive(Debug, Deserialize, Clone)]
struct Asset {
    name: String,
    browser_download_url: String,
    digest: String,
}

fn unzip(file_in: &Path) -> Result<(), Box<dyn Error>> {
    let file = fs::File::open(file_in)?;
    let dir_out = file_in.ancestors().nth(1).unwrap();
    dbg!(&dir_out);

    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => dir_out.join(path),
            None => continue,
        };
        dbg!(&outpath);

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
    thread::sleep(std::time::Duration::from_secs(1));
    let response = ureq::get(url).header("User-Agent", USERAGENT).call()?;

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
        if digest != "linux" && sum.split(" ").nth(0) != digest.split(":").last() {
            fs::remove_file(dir.join(filename))?;
            println!("{filename} fail: {digest:?} is not same as {sum}");
            return Err(format!("Sha256 fail for {filename}").into());
        }
    }
    Ok(())
}

fn get_github_release(url: &str) -> Result<GithubRelease, Box<dyn Error>> {
    thread::sleep(std::time::Duration::from_secs(1));
    Ok(ureq::get(url)
        .header("User-Agent", USERAGENT)
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
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(&dir.join("yt-dlp"), fs::Permissions::from_mode(0o755))?;
    }
    Ok(github.clone())
}

fn ffmpeg_install(dir: &Path, github: &GithubRelease) -> Result<GithubRelease, Box<dyn Error>> {
    let (file, ext) = match OS {
        "linux" => {
            return Ok(GithubRelease {
                name: "linux".to_string(),
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
    fs::remove_file(&zipfile)?;
    fs::copy(
        &zipfile
            .with_extension("")
            .join("bin")
            .join(format!("ffmpeg.{ext}")),
        dir.join(format!("ffmpeg.{ext}")),
    )?;
    Ok(github.clone())
}

fn deno_install(dir: &Path, github: &GithubRelease) -> Result<GithubRelease, Box<dyn Error>> {
    let file = match OS {
        "linux" => {
            #[cfg(target_arch = "aarch64")]
            {
                " deno-aarch64-unknown-linux-gnu.zip "
            }
            #[cfg(target_arch = "x86_64")]
            {
                "deno-x86_64-unknown-linux-gnu.zip"
            }
        }
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
    println!("dependi_install: yt_github {yt_github}");

    let ffmpeg_github =
        get_github_release("https://api.github.com/repos/yt-dlp/FFmpeg-Builds/releases/latest")?;
    println!("dependi_install: ffmpeg_github {ffmpeg_github}");

    let deno_github =
        get_github_release("https://api.github.com/repos/denoland/deno/releases/latest")?;
    println!("dependi_install: deno_github {deno_github}");

    match fs::read_to_string(&saved_data_location) {
        Ok(data) => {
            let mut data_struct: VersionJson = serde_json::from_str(&data)?;
            if data_struct.deno != deno_github.name {
                println!(
                    "Update avaiable for Deno from {} to {}",
                    data_struct.deno, deno_github.name
                );
                data_struct.deno = deno_install(dir, &deno_github)?.name;
            }
            if data_struct.ffmpeg != ffmpeg_github.name {
                println!(
                    "Update avaiable for ffmpeg from {} to {}",
                    data_struct.ffmpeg, ffmpeg_github.name
                );
                data_struct.ffmpeg = ffmpeg_install(dir, &ffmpeg_github)?.name;
            }
            if data_struct.yt_dlp != yt_github.name {
                println!(
                    "Update avaiable for yt_dlp from {} to {}",
                    data_struct.yt_dlp, yt_github.name
                );
                data_struct.yt_dlp = yt_dlp_install(dir, &yt_github)?.name;
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
impl std::fmt::Display for GithubRelease {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "release name is {}, it have {} assets",
            self.name,
            self.assets.len()
        )
    }
}
