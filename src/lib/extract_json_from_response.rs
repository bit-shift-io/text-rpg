use regex::Regex;

pub fn extract_json_from_response(hay: &str) -> Vec<String> {
    // todo: this could be improved, it doesnt match the ending ``` string properly
    let re = Regex::new(r"```json([\s\S]+)```").unwrap(); // ``json([^(````)]+)```

    let results = re.captures_iter(hay).map(|caps| {
        let (_, [json]) = caps.extract();
        let mut str = json.trim().to_string();
        json_strip_comments::strip(&mut str).unwrap();
        return str;

    }).collect::<Vec<_>>();

    results
}