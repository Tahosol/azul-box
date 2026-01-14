use log::info;
use serde_json::Value;
use url::form_urlencoded;

pub fn translate(to: &str, text: &str) -> Result<String, Box<dyn std::error::Error>> {
    let en_text: String = form_urlencoded::byte_serialize(text.as_bytes()).collect();
    let url = format!(
        "https://translate.googleapis.com/translate_a/single?client=gtx&sl=auto&tl={}&dt=t&q={}",
        to, en_text
    );

    let mut translated_text = text.to_string();

    let json_as_string = ureq::get(&url).call()?.body_mut().read_to_string()?;
    let values = serde_json::from_str::<Value>(&json_as_string)?;
    if let Some(value) = values.get(0) {
        info!("{:?}", value.as_str());
        if let Some(list) = value.as_array() {
            let lyrics: Vec<String> = list
                .iter()
                .filter_map(|v| v.get(0).and_then(|v| v.as_str()))
                .map(|s| s.to_string())
                .collect();
            translated_text = lyrics.join("");
            info!("Translate success!");
        }
    }
    Ok(translated_text)
}
