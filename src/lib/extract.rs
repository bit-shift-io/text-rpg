use regex::Regex;
use tracing::{error, info};
use std::sync::LazyLock;

pub fn extract_json(hay: &str) -> Result<Vec<String>, ()> {
    let re = match Regex::new(r"```json((?:.|\n)*?)```") {
        Ok(re) => re,
        Err(e) => return Err(()),
    };

    let results = re.captures_iter(hay).map(|caps| {
        let (_, [json]) = caps.extract();
        let mut str = json.trim().to_string();

        info!("{}", str);

        let str = match json_strip_comments::strip(&mut str) {
            Ok(stripped) => str,
            Err(_) => {
                error!("Failed to strip comments from JSON");
                return String::new(); // or return Err(e) if you want to propagate the error
            }
        };
        return str;

    }).collect::<Vec<_>>();

    Ok(results)
}


pub static RE_EXTRACT_STORY_BLOCK: LazyLock<Regex> = LazyLock::new(|| { Regex::new(r"```story((?:.|\n)*?)```").unwrap() });

pub fn extract_blocks(re: &Regex, hay: &str) -> Result<Vec<String>, ()> {
    let results = re.captures_iter(hay).map(|caps| {
        let (_, [extracted_str]) = caps.extract();
        let str = extracted_str.trim().to_string();
        //info!("{}", str);
        return str;
    }).collect::<Vec<_>>();
    Ok(results)
}