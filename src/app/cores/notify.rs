use std::{error::Error, process};

use rodio::Decoder;

pub fn done_sound(task: &str, file: String) -> Result<(), Box<dyn Error>> {
    let source = include_bytes!("../../../assets/sounds/completion-success.oga");
    let mut handle = rodio::DeviceSinkBuilder::open_default_sink()?;
    handle.log_on_drop(false);
    let player = rodio::Player::connect_new(&handle.mixer());
    let source = Decoder::new(std::io::Cursor::new(source)).unwrap();
    player.append(source);
    #[cfg(target_os = "linux")]
    {
        let _ = process::Command::new("notify-send")
            .arg("-a")
            .arg("Azul box")
            .arg(format!("Success {task} with file: {file}"))
            .spawn();
    }
    player.sleep_until_end();
    Ok(())
}
pub fn fail_sound(task: &str) -> Result<(), Box<dyn Error>> {
    let source = include_bytes!("../../../assets/sounds/completion-fail.oga");
    let mut handle = rodio::DeviceSinkBuilder::open_default_sink()?;
    handle.log_on_drop(false);
    let player = rodio::Player::connect_new(&handle.mixer());
    let source = Decoder::new(std::io::Cursor::new(source)).unwrap();
    player.append(source);
    #[cfg(target_os = "linux")]
    {
        let _ = process::Command::new("notify-send")
            .arg("-a")
            .arg("Azul box")
            .arg(format!("Fail {task}: Please check log"))
            .spawn();
    }
    player.sleep_until_end();
    Ok(())
}
pub fn button_sound() -> Result<(), Box<dyn Error>> {
    let source = include_bytes!("../../../assets/sounds/button-pressed-modifier.oga");
    let mut handle = rodio::DeviceSinkBuilder::open_default_sink()?;
    handle.log_on_drop(false);
    let player = rodio::Player::connect_new(&handle.mixer());
    let source = Decoder::new(std::io::Cursor::new(source)).unwrap();
    player.append(source);
    player.sleep_until_end();
    Ok(())
}
