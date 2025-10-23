use crate::app::shares::notify::{notification_done, notification_fail};

use std::process::Command;
pub fn version_check() -> Option<String> {
    match Command::new("yt-dlp").arg("--version").output() {
        Ok(out) => String::from_utf8(out.stdout).ok(),
        Err(_) => None,
    }
}

pub fn video_download(
    link: String,
    directory: String,
    format: i8,
    frag: i8,
    sub: bool,
    lang: &str,
    auto_gen: bool,
    cookies: Option<String>,
    use_cookies: bool,
) -> i8 {
    let n = frag.to_string().to_owned();

    let mut yt = Command::new("yt-dlp");
    if let Some(cookie) = cookies
        && use_cookies
    {
        yt.arg("--cookies").arg(cookie);
    }

    yt.arg("--concurrent-fragments")
        .arg(n)
        .arg("--embed-thumbnail")
        .arg("--embed-metadata")
        .current_dir(directory);
    if sub && auto_gen {
        yt.arg("--write-auto-subs")
            .arg("--embed-subs")
            .arg("--sub-lang")
            .arg(lang);
    } else if sub {
        yt.arg("--embed-subs").arg("--sub-lang").arg(lang);
    }

    if format == 1 {
        yt.arg("-f").arg("bestvideo+bestaudio");
    } else if format == 2 {
        yt.arg("-f")
            .arg("bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best");
    }
    let output = yt.arg(link).output().expect("Fail To Run Yt-dlp");

    let log = String::from_utf8_lossy(&output.stdout);
    println!("{log}");

    let status: i8 = if output.status.success() { 2 } else { 3 };

    if status == 2 {
        let _ = notification_done("video downloader");
    } else {
        let _ = notification_fail("video downloader");
    }
    status
}
