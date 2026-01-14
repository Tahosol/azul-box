use std::panic::Location;

pub fn get() -> String {
    let location = Location::caller();
    format!("{} : ", location.file())
}
