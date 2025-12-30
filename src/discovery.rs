use crate::config::Config;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct GeminiModel {
    name: String,
    #[serde(rename = "displayName")]
    display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GeminiListResponse {
    models: Option<Vec<GeminiModel>>,
}

#[derive(Debug, Deserialize)]
struct GenericModel {
    id: String,
}

#[derive(Debug, Deserialize)]
struct GenericListResponse {
    data: Vec<GenericModel>,
}

pub enum ModelType {
    Text,
    Image,
}

pub struct ModelInfo {
    pub name: String,
    pub model_type: ModelType,
}

pub async fn list_gemini_models(config: Config) -> Result<Vec<ModelInfo>, String> {
    if let Some(api_key) = config.gemini_api_key {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| e.to_string())?;

        let url = format!("https://generativelanguage.googleapis.com/v1beta/models?key={}", api_key);
        
        let resp = client.get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch Gemini models: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Gemini API Error: {}", resp.status()));
        }

        let body: GeminiListResponse = resp.json().await.map_err(|e| format!("Failed to parse Gemini response: {}", e))?;
        
        let models = body.models.unwrap_or_default().into_iter()
            .map(|m| {
                let name = format!("gemini/{}", m.name.replace("models/", ""));
                ModelInfo {
                    name,
                    model_type: ModelType::Text, // Gemini models are primary text/vision completion
                }
            })
            .filter(|m| m.name.contains("gemini"))
            .collect();

        Ok(models)
    } else {
        Err("No Gemini API key configured".to_string())
    }
}

pub async fn list_openai_models(config: Config) -> Result<Vec<ModelInfo>, String> {
    if let Some(api_key) = config.openai_api_key {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| e.to_string())?;

        let resp = client.get("https://api.openai.com/v1/models")
            .bearer_auth(api_key)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch OpenAI models: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("OpenAI API Error: {}", resp.status()));
        }

        let body: GenericListResponse = resp.json().await.map_err(|e| format!("Failed to parse OpenAI response: {}", e))?;
        Ok(body.data.into_iter().map(|m| {
            let model_type = if m.id.contains("dall-e") {
                ModelType::Image
            } else {
                ModelType::Text
            };
            ModelInfo {
                name: format!("openai/{}", m.id),
                model_type,
            }
        }).collect())
    } else {
        Err("No OpenAI API key configured".to_string())
    }
}

pub async fn list_groq_models(config: Config) -> Result<Vec<ModelInfo>, String> {
    if let Some(api_key) = config.groq_api_key {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| e.to_string())?;

        let resp = client.get("https://api.groq.com/openai/v1/models")
            .bearer_auth(api_key)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch Groq models: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Groq API Error: {}", resp.status()));
        }

        let body: GenericListResponse = resp.json().await.map_err(|e| format!("Failed to parse Groq response: {}", e))?;
        Ok(body.data.into_iter().map(|m| {
            ModelInfo {
                name: format!("groq/{}", m.id),
                model_type: ModelType::Text,
            }
        }).collect())
    } else {
        Err("No Groq API key configured".to_string())
    }
}

pub async fn list_huggingface_models(config: Config) -> Result<Vec<ModelInfo>, String> {
    if config.huggingface_api_key.is_some() {
        // Since listing all HF models might be too much, we'll just return a few common image ones for now
        // or we could check if Rig has a way to list them. 
        // For image generation, folks often use specific stable diffusion models.
        Ok(vec![
            ModelInfo {
                name: "hf/stabilityai/stable-diffusion-3-5-large".to_string(),
                model_type: ModelType::Image,
            },
            ModelInfo {
                name: "hf/black-forest-labs/FLUX.1-schnell".to_string(),
                model_type: ModelType::Image,
            },
        ])
    } else {
        Err("No HuggingFace API key configured".to_string())
    }
}

pub async fn list_all_models(config: Config) -> Vec<ModelInfo> {
    let mut all_models = Vec::new();

    if let Ok(models) = list_gemini_models(config.clone()).await {
        all_models.extend(models);
    }

    if let Ok(models) = list_openai_models(config.clone()).await {
        all_models.extend(models);
    }

    if let Ok(models) = list_groq_models(config.clone()).await {
        all_models.extend(models);
    }

    if let Ok(models) = list_huggingface_models(config.clone()).await {
        all_models.extend(models);
    }

    all_models
}
