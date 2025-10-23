use std::process::Command;

pub fn version_check() -> Option<String> {
    match Command::new("yt-dlp").arg("--version").output() {
        Ok(out) => String::from_utf8(out.stdout).ok(),
        Err(_) => None,
    }
}
