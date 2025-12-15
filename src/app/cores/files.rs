use std::fs;
use std::path::PathBuf;

pub fn file_finder(directory: &str, filename: &str, matchs: &[&str]) -> Option<PathBuf> {
    let elements = fs::read_dir(&directory).ok()?;

    for item in elements {
        let path = item.ok()?.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
                if matchs.contains(&ext) {
                    let file = path.file_name().and_then(|name| name.to_str())?;
                    if file.contains(filename) {
                        let good_file = Some(path);
                        return good_file;
                    }
                }
            }
        }
    }
    None
}
