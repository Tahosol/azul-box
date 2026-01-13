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
    } else {
        UrlStatus::Single
    }
}
