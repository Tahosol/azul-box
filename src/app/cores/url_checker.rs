pub enum UrlStatus {
    Single,
    Playlist,
    Radio,
    None,
}

pub fn playlist_check(url: &str) -> UrlStatus {
    if url.contains("start_radio") {
        UrlStatus::Radio
    } else if url.contains("list") {
        UrlStatus::Playlist
    } else if url.contains("http") {
        UrlStatus::Single
    } else {
        UrlStatus::None
    }
}

pub fn remove_radio(url: &str) -> String {
    if url.contains("&start_radio=1") && url.contains("&list=") {
        return url.split("&list=").nth(0).unwrap_or_default().to_string();
    } else {
        return url.to_string();
    }
}
