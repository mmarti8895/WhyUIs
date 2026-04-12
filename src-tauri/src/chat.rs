//! Chat module — handles LLM API calls for OpenAI and Anthropic.

use crate::persona::{build_system_prompt, sanitize_output};
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Settings passed from the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSettings {
    pub provider: String,
    pub openai_key: String,
    pub openai_model: String,
    pub anthropic_key: String,
    pub anthropic_model: String,
    pub temperature: f32,
}

/// A single message in the conversation history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryMessage {
    pub role: String,
    pub content: String,
}

// ── OpenAI ───────────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize, Deserialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
}

#[derive(Deserialize)]
struct OpenAiErrorWrapper {
    error: OpenAiError,
}

#[derive(Deserialize)]
struct OpenAiError {
    message: String,
}

// ── Anthropic ────────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    system: String,
    messages: Vec<AnthropicMessage>,
    temperature: f32,
}

#[derive(Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
}

#[derive(Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    text: Option<String>,
}

#[derive(Deserialize)]
struct AnthropicErrorWrapper {
    error: AnthropicError,
}

#[derive(Deserialize)]
struct AnthropicError {
    message: String,
}

// ── Main dispatch ─────────────────────────────────────────────────────────────

/// Send a message to the configured LLM and return the roasted response.
pub async fn send_to_llm(
    user_message: &str,
    history: &[HistoryMessage],
    settings: &ChatSettings,
    memory_context: &str,
) -> Result<String, String> {
    let response = match settings.provider.as_str() {
        "anthropic" => {
            send_anthropic(user_message, history, settings, memory_context).await?
        }
        _ => {
            // Default to OpenAI
            send_openai(user_message, history, settings, memory_context).await?
        }
    };

    Ok(sanitize_output(&response))
}

async fn send_openai(
    user_message: &str,
    history: &[HistoryMessage],
    settings: &ChatSettings,
    memory_context: &str,
) -> Result<String, String> {
    if settings.openai_key.is_empty() {
        return Err("OpenAI API key is not configured. Open Settings (⚙️) and add your key.".to_string());
    }

    let system_prompt = build_system_prompt(memory_context);

    let mut messages = vec![OpenAiMessage {
        role: "system".to_string(),
        content: system_prompt,
    }];

    for msg in history {
        messages.push(OpenAiMessage {
            role: msg.role.clone(),
            content: msg.content.clone(),
        });
    }

    messages.push(OpenAiMessage {
        role: "user".to_string(),
        content: user_message.to_string(),
    });

    let request = OpenAiRequest {
        model: settings.openai_model.clone(),
        messages,
        temperature: settings.temperature,
        max_tokens: 1024,
    };

    let client = Client::new();
    let res = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", settings.openai_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        if let Ok(err) = serde_json::from_str::<OpenAiErrorWrapper>(&body) {
            return Err(format!("OpenAI error: {}", err.error.message));
        }
        return Err(format!("OpenAI HTTP {status}: {body}"));
    }

    let data: OpenAiResponse = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse OpenAI response: {e}"))?;

    data.choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .ok_or_else(|| "Empty response from OpenAI".to_string())
}

async fn send_anthropic(
    user_message: &str,
    history: &[HistoryMessage],
    settings: &ChatSettings,
    memory_context: &str,
) -> Result<String, String> {
    if settings.anthropic_key.is_empty() {
        return Err("Anthropic API key is not configured. Open Settings (⚙️) and add your key.".to_string());
    }

    let system_prompt = build_system_prompt(memory_context);

    let mut messages: Vec<AnthropicMessage> = history
        .iter()
        .map(|m| AnthropicMessage {
            role: m.role.clone(),
            content: m.content.clone(),
        })
        .collect();

    messages.push(AnthropicMessage {
        role: "user".to_string(),
        content: user_message.to_string(),
    });

    let request = AnthropicRequest {
        model: settings.anthropic_model.clone(),
        max_tokens: 1024,
        system: system_prompt,
        messages,
        temperature: settings.temperature,
    };

    let client = Client::new();
    let res = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &settings.anthropic_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        if let Ok(err) = serde_json::from_str::<AnthropicErrorWrapper>(&body) {
            return Err(format!("Anthropic error: {}", err.error.message));
        }
        return Err(format!("Anthropic HTTP {status}: {body}"));
    }

    let data: AnthropicResponse = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse Anthropic response: {e}"))?;

    data.content
        .into_iter()
        .find(|c| c.content_type == "text")
        .and_then(|c| c.text)
        .ok_or_else(|| "Empty response from Anthropic".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_defaults() {
        let settings = ChatSettings {
            provider: "openai".to_string(),
            openai_key: String::new(),
            openai_model: "gpt-4o".to_string(),
            anthropic_key: String::new(),
            anthropic_model: "claude-3-5-sonnet-20241022".to_string(),
            temperature: 0.9,
        };
        assert_eq!(settings.provider, "openai");
        assert_eq!(settings.temperature, 0.9);
    }

    #[tokio::test]
    async fn test_no_key_returns_error() {
        let settings = ChatSettings {
            provider: "openai".to_string(),
            openai_key: String::new(),
            openai_model: "gpt-4o".to_string(),
            anthropic_key: String::new(),
            anthropic_model: "claude-3-5-sonnet-20241022".to_string(),
            temperature: 0.9,
        };
        let result = send_to_llm("hello", &[], &settings, "").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("API key"));
    }

    #[tokio::test]
    async fn test_anthropic_no_key_returns_error() {
        let settings = ChatSettings {
            provider: "anthropic".to_string(),
            openai_key: String::new(),
            openai_model: "gpt-4o".to_string(),
            anthropic_key: String::new(),
            anthropic_model: "claude-3-5-sonnet-20241022".to_string(),
            temperature: 0.9,
        };
        let result = send_to_llm("hello", &[], &settings, "").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("API key"));
    }
}
