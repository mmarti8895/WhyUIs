//! Memory system — short-term in-memory + long-term encrypted disk storage.

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use aes_gcm::aead::rand_core::RngCore;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use dirs::data_local_dir;
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::VecDeque,
    io::{Read, Write},
    path::PathBuf,
    sync::Mutex,
};

const MAX_SHORT_TERM: usize = 15;
const APP_NAME: &str = "WhyUIs";
const MEMORY_FILE: &str = "memory.dat";
const MACHINE_KEY_SEED: &str = "whyuis-roastbot-v1-memory-key";

/// A single message in short-term memory.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryMessage {
    pub role: String,
    /// Compact "caveman-style" summary
    pub content: String,
}

/// Long-term persistent memory.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LongTermMemory {
    pub name: Option<String>,
    pub preferences: Vec<String>,
    pub recurring_topics: Vec<String>,
    pub notes: Vec<String>,
}

/// The full memory state.
pub struct MemoryStore {
    pub short_term: Mutex<VecDeque<MemoryMessage>>,
    long_term: Mutex<LongTermMemory>,
    cipher_key: Vec<u8>,
}

impl MemoryStore {
    pub fn new() -> Self {
        let key = derive_machine_key();
        let store = Self {
            short_term: Mutex::new(VecDeque::new()),
            long_term: Mutex::new(LongTermMemory::default()),
            cipher_key: key,
        };
        // Try to load existing long-term memory
        if let Ok(lt) = store.load_long_term() {
            *store.long_term.lock().unwrap() = lt;
        }
        store
    }

    /// Add a message to short-term memory, evicting the oldest if full.
    pub fn push_message(&self, role: &str, content: &str) {
        let compact = compact_content(content);
        let mut st = self.short_term.lock().unwrap();
        if st.len() >= MAX_SHORT_TERM {
            st.pop_front();
        }
        st.push_back(MemoryMessage {
            role: role.to_string(),
            content: compact,
        });
    }

    /// Extract and persist facts from user input.
    pub fn extract_facts(&self, text: &str) {
        let mut lt = self.long_term.lock().unwrap();

        // Name extraction
        if let Some(name) = extract_pattern(text, &[
            r"(?i)my name is\s+([A-Za-z][A-Za-z'-]{1,31})",
            r"(?i)call me\s+([A-Za-z][A-Za-z'-]{1,31})",
            r"(?i)i go by\s+([A-Za-z][A-Za-z'-]{1,31})",
        ]) {
            if let Some(clean_name) = normalize_name(&name) {
                lt.name = Some(clean_name);
            }
        }

        // Preference extraction
        if let Some(pref) = extract_pattern(text, &[
            r"(?i)i (like|love|enjoy|prefer) (.+?)(?:\.|$)",
            r"(?i)i('m| am) (into|a fan of) (.+?)(?:\.|$)",
        ]) {
            if !lt.preferences.contains(&pref) && lt.preferences.len() < 20 {
                lt.preferences.push(pref);
            }
        }

        // Working on extraction
        if let Some(topic) = extract_pattern(text, &[
            r"(?i)i'?m (working on|building|learning) (.+?)(?:\.|$)",
            r"(?i)i (work|worked) (on|at|with) (.+?)(?:\.|$)",
        ]) {
            if !lt.recurring_topics.contains(&topic) && lt.recurring_topics.len() < 20 {
                lt.recurring_topics.push(topic);
            }
        }

        // Save after extraction
        drop(lt);
        let _ = self.save_long_term();
    }

    /// Clear both short-term and long-term memory.
    pub fn clear_all(&self) -> Result<(), String> {
        {
            let mut st = self.short_term.lock().unwrap();
            st.clear();
        }
        {
            let mut lt = self.long_term.lock().unwrap();
            *lt = LongTermMemory::default();
        }
        self.save_long_term()
    }

    /// Build the memory context string injected into every prompt.
    pub fn build_context(&self) -> String {
        let lt = self.long_term.lock().unwrap();
        let st = self.short_term.lock().unwrap();

        let mut parts: Vec<String> = Vec::new();

        if let Some(ref name) = lt.name {
            parts.push(format!("User: {name}"));
        }
        if !lt.preferences.is_empty() {
            parts.push(format!("Likes: {}", lt.preferences.join(", ")));
        }
        if !lt.recurring_topics.is_empty() {
            parts.push(format!("Works on: {}", lt.recurring_topics.join(", ")));
        }
        if !lt.notes.is_empty() {
            parts.push(format!("Notes: {}", lt.notes.join("; ")));
        }

        let recent: Vec<String> = st
            .iter()
            .rev()
            .take(5)
            .rev()
            .map(|m| format!("[{}] {}", m.role, m.content))
            .collect();

        if !recent.is_empty() {
            parts.push(format!("Recent: {}", recent.join(" | ")));
        }

        parts.join("\n")
    }

    /// Encrypt and save long-term memory to disk.
    fn save_long_term(&self) -> Result<(), String> {
        if cfg!(test) {
            return Ok(());
        }

        let lt = self.long_term.lock().unwrap().clone();
        let json = serde_json::to_vec(&lt).map_err(|e| e.to_string())?;

        // Compress
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&json).map_err(|e| e.to_string())?;
        let compressed = encoder.finish().map_err(|e| e.to_string())?;

        // Encrypt
        let encrypted = self.encrypt(&compressed)?;
        let encoded = BASE64.encode(&encrypted);

        let path = memory_file_path();
        std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        std::fs::write(&path, encoded).map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Load and decrypt long-term memory from disk.
    fn load_long_term(&self) -> Result<LongTermMemory, String> {
        if cfg!(test) {
            return Ok(LongTermMemory::default());
        }

        let path = memory_file_path();
        if !path.exists() {
            return Ok(LongTermMemory::default());
        }

        let encoded = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let encrypted = BASE64.decode(encoded.trim()).map_err(|e| e.to_string())?;

        // Decrypt
        let compressed = self.decrypt(&encrypted)?;

        // Decompress
        let mut decoder = GzDecoder::new(compressed.as_slice());
        let mut json = Vec::new();
        decoder.read_to_end(&mut json).map_err(|e| e.to_string())?;

        let lt: LongTermMemory = serde_json::from_slice(&json).map_err(|e| e.to_string())?;
        Ok(lt)
    }

    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, String> {
        let key = Key::<Aes256Gcm>::from_slice(&self.cipher_key);
        let cipher = Aes256Gcm::new(key);

        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| e.to_string())?;

        // Prepend nonce to ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        if data.len() < 12 {
            return Err("Invalid encrypted data".to_string());
        }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let key = Key::<Aes256Gcm>::from_slice(&self.cipher_key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(nonce_bytes);

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| e.to_string())
    }
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn derive_machine_key() -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(MACHINE_KEY_SEED.as_bytes());
    // Mix in hostname for machine-specific key
    if let Ok(hostname) = std::env::var("HOSTNAME").or_else(|_| std::env::var("COMPUTERNAME")) {
        hasher.update(hostname.as_bytes());
    }
    hasher.finalize().to_vec()
}

fn memory_file_path() -> PathBuf {
    data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(APP_NAME)
        .join(MEMORY_FILE)
}

fn normalize_name(raw: &str) -> Option<String> {
    let cleaned = raw
        .trim()
        .trim_matches(|c: char| !c.is_ascii_alphabetic() && c != '\'' && c != '-')
        .to_string();

    if cleaned.len() < 2 || cleaned.len() > 32 {
        return None;
    }

    let lower = cleaned.to_ascii_lowercase();
    let stop_words = ["not", "no", "nah", "none", "unknown"];
    if stop_words.contains(&lower.as_str()) {
        return None;
    }

    Some(cleaned)
}

/// Reduce content to a compact caveman-style summary.
fn compact_content(text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() <= 12 {
        return text.to_string();
    }
    // Take first 8 + last 4 words
    let head: Vec<&str> = words[..8].to_vec();
    let tail: Vec<&str> = words[words.len() - 4..].to_vec();
    format!("{}...{}", head.join(" "), tail.join(" "))
}

/// Try multiple patterns, return first match group.
fn extract_pattern(text: &str, patterns: &[&str]) -> Option<String> {
    for pattern in patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(caps) = re.captures(text) {
                // Return last capture group
                let idx = caps.len().saturating_sub(1);
                if idx > 0 {
                    if let Some(m) = caps.get(idx) {
                        let s = m.as_str().trim().to_string();
                        if !s.is_empty() {
                            return Some(s);
                        }
                    }
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_context() {
        let store = MemoryStore::new();
        store.push_message("user", "Hello world");
        store.push_message("assistant", "Roasting you now");
        let ctx = store.build_context();
        assert!(ctx.contains("Hello world") || ctx.contains("Roasting"));
    }

    #[test]
    fn test_max_short_term() {
        let store = MemoryStore::new();
        for i in 0..20 {
            store.push_message("user", &format!("message {i}"));
        }
        let st = store.short_term.lock().unwrap();
        assert_eq!(st.len(), MAX_SHORT_TERM);
    }

    #[test]
    fn test_compact_content() {
        let long_text = "one two three four five six seven eight nine ten eleven twelve thirteen";
        let compact = compact_content(long_text);
        assert!(compact.contains("..."));
    }

    #[test]
    fn test_extract_name() {
        let store = MemoryStore::new();
        store.extract_facts("My name is Alice");
        let lt = store.long_term.lock().unwrap();
        assert_eq!(lt.name.as_deref(), Some("Alice"));
    }

    #[test]
    fn test_does_not_extract_name_from_plain_im_statement() {
        let store = MemoryStore::new();
        store.extract_facts("I'm working on a Tauri app");
        let lt = store.long_term.lock().unwrap();
        assert!(lt.name.is_none());
    }

    #[test]
    fn test_encrypt_decrypt() {
        let store = MemoryStore::new();
        let plaintext = b"hello secret memory";
        let encrypted = store.encrypt(plaintext).unwrap();
        let decrypted = store.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }
}
