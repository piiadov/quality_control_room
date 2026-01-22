//! Ollama API client for LLM inference

use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

const OLLAMA_URL: &str = "http://127.0.0.1:11434";

#[derive(Debug, Serialize)]
pub struct GenerateRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GenerateResponse {
    pub response: String,
    pub done: bool,
}

#[derive(Clone)]
pub struct OllamaClient {
    client: Client,
    model: String,
    system_prompt: String,
}

impl OllamaClient {
    pub fn new(model: &str, documentation: &str) -> Self {
        let system_prompt = format!(
            r#"You are the Virtual Engineer, an AI assistant for the Quality Control Room application.
Your role is to help users understand their quality control analysis results.

You have access to the complete application documentation:

{}

When answering:
- Be concise and practical
- Focus on quality control interpretation
- Explain statistical concepts in simple terms (hypergeometric sampling, CDF fitting, chi-square tests)
- Reference the documentation when relevant
- If given analysis data, explain what the parameter values mean
- Explain the difference between min/max/predicted/sampling estimates
- Interpret chi-square p-values for goodness of fit
- Suggest actions based on results when appropriate

Keep responses focused and under 300 words unless more detail is requested."#,
            documentation
        );

        Self {
            client: Client::new(),
            model: model.to_string(),
            system_prompt,
        }
    }

    /// Check if Ollama is available
    pub async fn health_check(&self) -> bool {
        self.client
            .get(format!("{}/api/tags", OLLAMA_URL))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    /// Stream a response from the LLM
    pub async fn generate_stream(
        &self,
        prompt: &str,
        context: Option<&str>,
    ) -> Result<mpsc::Receiver<String>, String> {
        let full_prompt = match context {
            Some(ctx) => format!(
                "Current analysis context:\n```json\n{}\n```\n\nUser question: {}",
                ctx, prompt
            ),
            None => prompt.to_string(),
        };

        let request = GenerateRequest {
            model: self.model.clone(),
            prompt: full_prompt,
            stream: true,
            system: Some(self.system_prompt.clone()),
        };

        let response = self
            .client
            .post(format!("{}/api/generate", OLLAMA_URL))
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to Ollama: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Ollama error: {}", response.status()));
        }

        let (tx, rx) = mpsc::channel(100);
        let mut stream = response.bytes_stream();

        tokio::spawn(async move {
            while let Some(chunk) = stream.next().await {
                if let Ok(bytes) = chunk {
                    if let Ok(text) = String::from_utf8(bytes.to_vec()) {
                        // Each line is a JSON object
                        for line in text.lines() {
                            if let Ok(resp) = serde_json::from_str::<GenerateResponse>(line) {
                                if !resp.response.is_empty() {
                                    if tx.send(resp.response).await.is_err() {
                                        break;
                                    }
                                }
                                if resp.done {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        });

        Ok(rx)
    }

    /// Non-streaming generate (for simple responses)
    #[allow(dead_code)]
    pub async fn generate(&self, prompt: &str, context: Option<&str>) -> Result<String, String> {
        let full_prompt = match context {
            Some(ctx) => format!(
                "Current analysis context:\n```json\n{}\n```\n\nUser question: {}",
                ctx, prompt
            ),
            None => prompt.to_string(),
        };

        let request = GenerateRequest {
            model: self.model.clone(),
            prompt: full_prompt,
            stream: false,
            system: Some(self.system_prompt.clone()),
        };

        let response = self
            .client
            .post(format!("{}/api/generate", OLLAMA_URL))
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to Ollama: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Ollama error: {}", response.status()));
        }

        let resp: GenerateResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(resp.response)
    }
}
