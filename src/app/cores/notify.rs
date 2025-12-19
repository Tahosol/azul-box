use std::error::Error;

use rodio::Decoder;

pub fn done_sound() -> Result<(), Box<dyn Error>> {
    let source = include_bytes!("../../../assets/sounds/success.wav");
    let stream_handle = rodio::OutputStreamBuilder::open_default_stream()?;
    let sink = rodio::Sink::connect_new(stream_handle.mixer());
    let source = Decoder::new(std::io::Cursor::new(source)).unwrap();
    sink.append(source);
    sink.sleep_until_end();
    Ok(())
}
pub fn fail_sound() -> Result<(), Box<dyn Error>> {
    let source = include_bytes!("../../../assets/sounds/fail.wav");
    let stream_handle = rodio::OutputStreamBuilder::open_default_stream()?;
    let sink = rodio::Sink::connect_new(stream_handle.mixer());
    let source = Decoder::new(std::io::Cursor::new(source)).unwrap();
    sink.append(source);
    sink.sleep_until_end();
    Ok(())
}
pub fn button_sound() -> Result<(), Box<dyn Error>> {
    let source = include_bytes!("../../../assets/sounds/button.wav");
    let stream_handle = rodio::OutputStreamBuilder::open_default_stream()?;
    let sink = rodio::Sink::connect_new(stream_handle.mixer());
    let source = Decoder::new(std::io::Cursor::new(source)).unwrap();
    sink.append(source);
    sink.sleep_until_end();
    Ok(())
}
use notify_rust::Notification;
pub fn notification_done(info: &str) -> Result<(), Box<dyn Error>> {
    Notification::new()
        .summary("Azul Box")
        .body(format!("Your {} is done!", info).as_str())
        .icon("azul_box")
        .show()?;
    Ok(())
}
pub fn notification_fail(info: &str) -> Result<(), Box<dyn Error>> {
    Notification::new()
        .summary("Azul Box")
        .body(format!("Your {} FAIL!!", info).as_str())
        .icon("azul_box")
        .show()?;
    Ok(())
}
