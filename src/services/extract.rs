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

pub fn extract_markdown_block(info_string: &str, hay: &str) -> Result<Vec<String>, ()> {
    let formatted = format!(r"```{}\n((?:.|\n)*?)```", info_string);
    let re = Regex::new(&formatted).unwrap();
    let results = re.captures_iter(hay).map(|caps| {
        let (_, [extracted_str]) = caps.extract();
        let str = extracted_str.trim().to_string();
        return str;
    }).collect::<Vec<_>>();
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_markdown_block_simple() {
        let hay = "Here is some code:\n```json\n{\"foo\": \"bar\"}\n```\nEnd of code.";
        let results = extract_markdown_block("json", hay).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], "{\"foo\": \"bar\"}");
    }

    #[test]
    fn test_extract_markdown_block_with_newlines() {
        let hay = "```json\n\n{\n  \"foo\": \"bar\"\n}\n\n```";
        let results = extract_markdown_block("json", hay).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], "{\n  \"foo\": \"bar\"\n}");
    }

    #[test]
    fn test_extract_markdown_block_multiple() {
        let hay = "```json\n{\"a\": 1}\n```\ntext\n```json\n{\"b\": 2}\n```";
        let results = extract_markdown_block("json", hay).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], "{\"a\": 1}");
        assert_eq!(results[1], "{\"b\": 2}");
    }

    #[test]
    fn test_extract_markdown_block_wrong_lang() {
        let hay = "```rust\nfn main() {}\n```";
        let results = extract_markdown_block("json", hay).unwrap();
        assert_eq!(results.len(), 0);
    }
}