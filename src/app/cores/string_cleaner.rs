pub fn clean_title_before_api_call(title: &str, author_name: &str) -> String {
    let regex_patterns = [r"【[A-Za-z0-9]+】", r"\([^)]*\)"];
    let mut cleaned_content = title.to_string();
    for pattern in regex_patterns.iter() {
        let regex = regex::Regex::new(pattern).unwrap();
        cleaned_content = regex.replace_all(&cleaned_content, "").to_string();
    }
    cleaned_content
        .replace(author_name, "")
        .replace("MV", "")
        .trim()
        .to_string()
}
