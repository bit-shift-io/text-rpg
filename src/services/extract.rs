use regex::Regex;
use tracing::{error, info};
use std::sync::LazyLock;
pub fn extract_between(opening_tag: &str, closing_tag: &str, hay: &str) -> Result<Vec<String>, ()> {
    let formatted = format!(r"{}((?:.|\n)*?){}", opening_tag, closing_tag);
    let re = Regex::new(&formatted).unwrap();
    let results = re.captures_iter(hay).map(|caps| {
        let (_, [extracted_str]) = caps.extract();
        let str = extracted_str.trim().to_string();
        //info!("{}", str);
        return str;
    }).collect::<Vec<_>>();
    Ok(results)
}