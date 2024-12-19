// https://getimg.ai/tools/api

/*

curl \
-X POST https://api.getimg.ai/v1/essential-v2/text-to-image \
-H "Authorization: Bearer $ACCESS_TOKEN" \
-H "Content-Type: application/json" \
-d '{"prompt":"astronaut riding a horse on mars"}'

*/

use std::collections::HashMap;
use tracing::{error, info, warn};

use crate::GLOBAL_CONFIG;

pub async fn get_url_for_prompt(prompt: &str) -> Result<String, ()> {
    let config = GLOBAL_CONFIG.lock().await.clone().unwrap();

    if config.getimgai_api_key.is_none() {
        warn!("No getimgai_api_key found in config");
        return Err(());
    }

    let mut map = HashMap::new();
    map.insert("prompt", prompt);

    let client = reqwest::Client::new();

    let body_str = match serde_json::to_string(&map) {
        Ok(body_str) => body_str,
        Err(err) => {
            error!("Failed to serialize JSON: {}", err);
            return Err(());
        }
    };

    let res = match client.post("http://httpbin.org/post")
        .bearer_auth(config.getimgai_api_key.unwrap())
        .body(body_str)
        .send()
        .await {
            Ok(response) => {
                let res_str = response.text().await.unwrap(); // todo: replace unwrap with ?
                info!("[get_url_from_prompt] response: {:?}", res_str);
                //serde_json::from_str(&res_str); //.unwrap().get("url").cloned().unwrap_or("".to_string())
            }
            Err(err) => {
                error!("Failed to get image URL: {}",err);
                return Err(());
            }
        };

    Ok("".to_string())
}