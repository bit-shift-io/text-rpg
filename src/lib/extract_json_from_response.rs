use regex::Regex;

pub fn extract_json_from_response(hay: &str) -> Vec<&str> {
    // todo: this could be improved, it doesnt match the ending ``` string properly
    let re = Regex::new(r"```json([\s\S]+)```").unwrap(); // ``json([^(````)]+)```

    let results: Vec<&str> = re.captures_iter(hay).map(|caps| {
        let (_, [json]) = caps.extract();
        json.trim()
    }).collect();

    results
}