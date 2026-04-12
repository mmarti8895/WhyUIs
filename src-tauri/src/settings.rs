//! Encrypted settings store with .env override support.
//!
//! Keys are stored AES-256-GCM encrypted at rest.  API calls use HTTPS (TLS)
//! so keys are also encrypted in transit.  The frontend never receives raw
//! keys — it only sees the masked sentinel "••••••".

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use aes_gcm::aead::rand_core::RngCore;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use dirs::data_local_dir;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{path::PathBuf, sync::Mutex};

const APP_NAME: &str = "WhyUIs";
const SETTINGS_FILE: &str = "settings.dat";
const SETTINGS_KEY_SEED: &str = "whyuis-roastbot-v1-settings-key";

/// Sentinel value returned to the frontend when a key is stored but must not
/// be transmitted.  If the frontend sends this back on save, the existing key
/// is kept unchanged.
pub const KEY_MASKED: &str = "\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}";

// ── On-disk representation ────────────────────────────────────────────────────

/// Settings persisted to disk inside an AES-256-GCM encryption envelope.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredSettings {
    pub provider: String,
    pub openai_key: String,
    pub openai_model: String,
    pub anthropic_key: String,
    pub anthropic_model: String,
    pub temperature: f32,
}

impl Default for StoredSettings {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            openai_key: String::new(),
            openai_model: "gpt-5.4".to_string(),
            anthropic_key: String::new(),
            anthropic_model: "claude-4-sonnet-20241022".to_string(),
            temperature: 0.9,
        }
    }
}

// ── Frontend-facing types ─────────────────────────────────────────────────────

/// Settings sent to the frontend.  Raw keys are replaced with `KEY_MASKED`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiSettings {
    pub provider: String,
    /// `KEY_MASKED` if a key is stored; `""` if no key is configured.
    pub openai_key: String,
    pub openai_model: String,
    pub anthropic_key: String,
    pub anthropic_model: String,
    pub temperature: f32,
    /// `true` when locked by an `.env` / environment variable — the frontend
    /// should display these fields as read-only.
    pub openai_from_env: bool,
    pub anthropic_from_env: bool,
}

/// Payload the frontend sends when the user clicks "Save".
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSettingsRequest {
    pub provider: String,
    /// New raw key, or `KEY_MASKED` to keep the existing stored key unchanged.
    pub openai_key: String,
    pub openai_model: String,
    pub anthropic_key: String,
    pub anthropic_model: String,
    pub temperature: f32,
}

// ── .env / environment-variable overrides ────────────────────────────────────

/// Ephemeral overrides read from environment variables (never written to disk).
#[derive(Default, Clone)]
struct EnvConfig {
    openai_key: Option<String>,
    openai_model: Option<String>,
    anthropic_key: Option<String>,
    anthropic_model: Option<String>,
}

// ── Store ─────────────────────────────────────────────────────────────────────

pub struct SettingsStore {
    disk: Mutex<StoredSettings>,
    env: EnvConfig,
    cipher_key: Vec<u8>,
}

impl SettingsStore {
    pub fn new() -> Self {
        // Best-effort: load a `.env` file from the CWD or any ancestor.
        let _ = dotenvy::dotenv();

        let cipher_key = derive_key();
        let env = read_env_config();

        let store = SettingsStore {
            disk: Mutex::new(StoredSettings::default()),
            env,
            cipher_key,
        };

        if let Ok(s) = store.load_from_disk() {
            *store.disk.lock().unwrap() = s;
        }

        store
    }

    // ── Read ──────────────────────────────────────────────────────────────────

    /// Effective settings used for every API call.
    /// Stored (UI) values take precedence; .env fills in keys/models that
    /// haven't been set in the app yet.
    pub fn effective(&self) -> StoredSettings {
        let mut s = self.disk.lock().unwrap().clone();
        let e = &self.env;

        // .env is a fallback — only fills gaps where the stored key is empty.
        if s.openai_key.is_empty() {
            if let Some(ref k) = e.openai_key  { s.openai_key   = k.clone(); }
            if let Some(ref m) = e.openai_model { s.openai_model = m.clone(); }
        }
        if s.anthropic_key.is_empty() {
            if let Some(ref k) = e.anthropic_key  { s.anthropic_key   = k.clone(); }
            if let Some(ref m) = e.anthropic_model { s.anthropic_model = m.clone(); }
        }

        // Auto-select provider based on key availability.
        // OpenAI preferred when both keys are present.
        if !s.openai_key.is_empty() {
            s.provider = "openai".to_string();
        } else if !s.anthropic_key.is_empty() {
            s.provider = "anthropic".to_string();
        }

        s
    }

    /// Masked settings safe to send to the frontend.
    /// Keys are replaced with KEY_MASKED when non-empty. `*_from_env` is true
    /// when no key is stored in the app and the active key comes from .env.
    /// The frontend always shows editable inputs — from_env is just a hint.
    pub fn ui_settings(&self) -> UiSettings {
        let disk = self.disk.lock().unwrap().clone();
        let e = &self.env;

        // from_env = no stored key, but env provides one
        let openai_from_env    = disk.openai_key.is_empty()    && e.openai_key.is_some();
        let anthropic_from_env = disk.anthropic_key.is_empty() && e.anthropic_key.is_some();

        let eff_openai_key      = if openai_from_env    { e.openai_key.as_deref().unwrap_or("")    } else { &disk.openai_key };
        let eff_anthropic_key   = if anthropic_from_env { e.anthropic_key.as_deref().unwrap_or("") } else { &disk.anthropic_key };
        let eff_openai_model    = if openai_from_env    { e.openai_model.clone().unwrap_or_else(|| disk.openai_model.clone())       } else { disk.openai_model.clone() };
        let eff_anthropic_model = if anthropic_from_env { e.anthropic_model.clone().unwrap_or_else(|| disk.anthropic_model.clone()) } else { disk.anthropic_model.clone() };

        let provider = if !eff_openai_key.is_empty() {
            "openai".to_string()
        } else if !eff_anthropic_key.is_empty() {
            "anthropic".to_string()
        } else {
            disk.provider.clone()
        };

        UiSettings {
            provider,
            openai_key: mask_key(eff_openai_key),
            openai_model: eff_openai_model,
            anthropic_key: mask_key(eff_anthropic_key),
            anthropic_model: eff_anthropic_model,
            temperature: disk.temperature,
            openai_from_env,
            anthropic_from_env,
        }
    }

    // ── Write ─────────────────────────────────────────────────────────────────

    /// Merge new settings from the frontend and persist to disk.
    /// Keys equal to `KEY_MASKED` or empty are not overwritten.
    pub fn update(&self, req: SaveSettingsRequest) -> Result<(), String> {
        let mut disk = self.disk.lock().unwrap();

        disk.provider       = req.provider;
        disk.openai_model   = req.openai_model;
        disk.anthropic_model = req.anthropic_model;
        disk.temperature    = req.temperature;

        if req.openai_key != KEY_MASKED && !req.openai_key.is_empty() {
            disk.openai_key = req.openai_key;
        }
        if req.anthropic_key != KEY_MASKED && !req.anthropic_key.is_empty() {
            disk.anthropic_key = req.anthropic_key;
        }

        let to_save = disk.clone();
        drop(disk);
        self.save_to_disk(&to_save)
    }

    // ── Persistence ───────────────────────────────────────────────────────────

    fn save_to_disk(&self, settings: &StoredSettings) -> Result<(), String> {
        let json      = serde_json::to_vec(settings).map_err(|e| e.to_string())?;
        let encrypted = self.encrypt(&json)?;
        let encoded   = BASE64.encode(&encrypted);

        let path = settings_file_path();
        std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        std::fs::write(&path, encoded).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn load_from_disk(&self) -> Result<StoredSettings, String> {
        let path = settings_file_path();
        if !path.exists() {
            return Ok(StoredSettings::default());
        }
        let encoded   = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let encrypted = BASE64.decode(encoded.trim()).map_err(|e| e.to_string())?;
        let json      = self.decrypt(&encrypted)?;
        serde_json::from_slice(&json).map_err(|e| e.to_string())
    }

    // ── Crypto ────────────────────────────────────────────────────────────────

    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, String> {
        let key    = Key::<Aes256Gcm>::from_slice(&self.cipher_key);
        let cipher = Aes256Gcm::new(key);

        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| e.to_string())?;

        // Prepend nonce so it can be recovered during decryption.
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        if data.len() < 12 {
            return Err("Encrypted settings data is too short".to_string());
        }
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let key    = Key::<Aes256Gcm>::from_slice(&self.cipher_key);
        let cipher = Aes256Gcm::new(key);
        let nonce  = Nonce::from_slice(nonce_bytes);
        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| "Failed to decrypt settings — data may be corrupt".to_string())
    }
}

impl Default for SettingsStore {
    fn default() -> Self {
        Self::new()
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Derive a machine-specific 256-bit key from a fixed seed + hostname.
fn derive_key() -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(SETTINGS_KEY_SEED.as_bytes());
    if let Ok(host) = std::env::var("HOSTNAME").or_else(|_| std::env::var("COMPUTERNAME")) {
        hasher.update(host.as_bytes());
    }
    hasher.finalize().to_vec()
}

fn settings_file_path() -> PathBuf {
    data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(APP_NAME)
        .join(SETTINGS_FILE)
}

/// Read the four recognised env vars into an `EnvConfig`.
fn read_env_config() -> EnvConfig {
    EnvConfig {
        openai_key:      std::env::var("OPENAI_API_KEY")   .ok().and_then(|s| normalize_api_key(&s)),
        openai_model:    std::env::var("OPENAI_MODEL")     .ok().filter(|s| !s.is_empty()),
        anthropic_key:   std::env::var("ANTHROPIC_API_KEY").ok().and_then(|s| normalize_api_key(&s)),
        anthropic_model: std::env::var("ANTHROPIC_MODEL")  .ok().filter(|s| !s.is_empty()),
    }
}

fn normalize_api_key(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Treat placeholder keys as not configured so users can set real keys in-app.
    let placeholders = [
        "sk-your-openai-key-here",
        "sk-ant-your-anthropic-key-here",
    ];
    if placeholders.iter().any(|p| trimmed.eq_ignore_ascii_case(p)) {
        return None;
    }

    Some(trimmed.to_string())
}

/// Return `KEY_MASKED` for a non-empty key; return `""` for an empty one.
fn mask_key(key: &str) -> String {
    if key.is_empty() { String::new() } else { KEY_MASKED.to_string() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_store(disk: StoredSettings, env: EnvConfig) -> SettingsStore {
        SettingsStore {
            disk: Mutex::new(disk),
            env,
            cipher_key: vec![7u8; 32],
        }
    }

    #[test]
    fn normalize_api_key_treats_placeholders_as_unset() {
        assert_eq!(normalize_api_key(""), None);
        assert_eq!(normalize_api_key("   "), None);
        assert_eq!(normalize_api_key("sk-your-openai-key-here"), None);
        assert_eq!(normalize_api_key("sk-ant-your-anthropic-key-here"), None);
    }

    #[test]
    fn normalize_api_key_keeps_real_value() {
        let key = "sk-real-key-123";
        assert_eq!(normalize_api_key(key), Some(key.to_string()));
        assert_eq!(normalize_api_key("  sk-real-key-123  "), Some(key.to_string()));
    }

    #[test]
    fn mask_key_masks_only_non_empty_values() {
        assert_eq!(mask_key(""), "");
        assert_eq!(mask_key("abc"), KEY_MASKED.to_string());
    }

    #[test]
    fn effective_prefers_openai_when_both_keys_present() {
        let disk = StoredSettings {
            provider: "anthropic".to_string(),
            openai_key: "sk-openai".to_string(),
            openai_model: "gpt-5.4".to_string(),
            anthropic_key: "sk-ant".to_string(),
            anthropic_model: "claude-4-sonnet-20241022".to_string(),
            temperature: 0.9,
        };
        let env = EnvConfig::default();
        let store = make_store(disk, env);

        let eff = store.effective();
        assert_eq!(eff.provider, "openai");
    }

    #[test]
    fn effective_uses_env_fallback_when_disk_key_missing() {
        let disk = StoredSettings {
            provider: "openai".to_string(),
            openai_key: String::new(),
            openai_model: "gpt-5.4".to_string(),
            anthropic_key: String::new(),
            anthropic_model: "claude-4-sonnet-20241022".to_string(),
            temperature: 0.9,
        };
        let env = EnvConfig {
            openai_key: Some("sk-env-openai".to_string()),
            openai_model: Some("gpt-5.4".to_string()),
            anthropic_key: None,
            anthropic_model: None,
        };
        let store = make_store(disk, env);

        let eff = store.effective();
        assert_eq!(eff.openai_key, "sk-env-openai");
        assert_eq!(eff.provider, "openai");
    }

    #[test]
    fn ui_settings_marks_env_source_when_disk_empty() {
        let disk = StoredSettings::default();
        let env = EnvConfig {
            openai_key: Some("sk-env-openai".to_string()),
            openai_model: Some("gpt-5.4".to_string()),
            anthropic_key: None,
            anthropic_model: None,
        };
        let store = make_store(disk, env);

        let ui = store.ui_settings();
        assert!(ui.openai_from_env);
        assert_eq!(ui.openai_key, KEY_MASKED);
        assert_eq!(ui.provider, "openai");
    }
}
