use anyhow::{Context, Result};
use ollama_rs::{
    Ollama,
    generation::chat::{ChatMessage, request::ChatMessageRequest},
    generation::parameters::KeepAlive,
};
use std::time::{Duration, Instant};

const DEFAULT_PROMPT: &str = r#"Improve the following text for clarity, grammar, and style.
Keep the original meaning and tone.
Only output the improved text, nothing else.
Do not add explanations or commentary."#;

pub struct TextImprover {
    ollama: Ollama,
    model: String,
}

impl TextImprover {
    pub fn new(host: &str, port: u16, model: &str) -> Self {
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(120))
            .pool_idle_timeout(Duration::from_secs(60))
            .pool_max_idle_per_host(0) // Disable connection pooling
            .build()
            .expect("Failed to create HTTP client");

        Self {
            ollama: Ollama::new_with_client(host.to_string(), port, client),
            model: model.to_string(),
        }
    }

    pub async fn improve(&self, text: &str) -> Result<String> {
        if text.trim().is_empty() {
            return Ok(String::new());
        }

        let messages = vec![
            ChatMessage::system(DEFAULT_PROMPT.to_string()),
            ChatMessage::user(text.to_string()),
        ];

        let request = ChatMessageRequest::new(self.model.clone(), messages)
            .think(false)
            .keep_alive(KeepAlive::Indefinitely);

        // Retry logic for stale connections
        let mut last_error = None;
        for attempt in 1..=3 {
            let start = Instant::now();
            log::debug!("Ollama request attempt {} for text: {:?}", attempt, text);

            match self.ollama.send_chat_messages(request.clone()).await {
                Ok(response) => {
                    let improved = response.message.content.trim().to_string();
                    log::info!(
                        "Ollama improved text in {:?}: {:?} -> {:?}",
                        start.elapsed(),
                        text,
                        improved
                    );
                    return Ok(improved);
                }
                Err(e) => {
                    log::warn!("Ollama attempt {} failed: {}", attempt, e);
                    last_error = Some(e);
                    if attempt < 3 {
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap()).context("All Ollama retry attempts failed")
    }
}
