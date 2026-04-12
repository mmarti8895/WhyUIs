//! WhyUIs — AI Roast Chatbot
//! Tauri application library entry point.

mod chat;
mod memory;
mod persona;
mod settings;

use chat::HistoryMessage;
use memory::MemoryStore;
use settings::{SaveSettingsRequest, SettingsStore, UiSettings};
use std::sync::Arc;
use tauri::State;

/// Global app state.
pub struct AppState {
    pub memory: Arc<MemoryStore>,
    pub settings: Arc<SettingsStore>,
}

/// Tauri command: send a message and get a roasted response.
/// API keys are read from the backend settings store — they are never sent
/// over the frontend↔backend bridge.
#[tauri::command]
async fn send_message(
    message: String,
    session_id: String,
    history: Vec<HistoryMessage>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Extract facts from user message
    state.memory.extract_facts(&message);

    // Push to short-term memory
    state.memory.push_message("user", &message);

    // Build memory context
    let memory_context = state.memory.build_context();

    // Drop session_id usage to avoid lint warning (used for routing in future)
    let _ = session_id;

    // Resolve effective settings (disk + .env overrides)
    let effective = state.settings.effective();

    // Call LLM
    let response = chat::send_to_llm(&message, &history, &effective, &memory_context).await?;

    // Store assistant response in memory
    state.memory.push_message("assistant", &response);

    Ok(response)
}

/// Tauri command: load settings for the UI (keys are masked).
#[tauri::command]
fn load_settings(state: State<'_, AppState>) -> UiSettings {
    state.settings.ui_settings()
}

/// Tauri command: save settings from the UI (merges with stored keys).
#[tauri::command]
fn save_settings(
    request: SaveSettingsRequest,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.settings.update(request)
}

/// Tauri command: clear memory for the current user.
#[tauri::command]
fn clear_memory(state: State<'_, AppState>) -> Result<(), String> {
    state.memory.clear_all()
}

/// Tauri command: get the current memory context (for debugging).
#[tauri::command]
fn get_memory_context(state: State<'_, AppState>) -> String {
    state.memory.build_context()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            memory: Arc::new(MemoryStore::new()),
            settings: Arc::new(SettingsStore::new()),
        })
        .invoke_handler(tauri::generate_handler![
            send_message,
            load_settings,
            save_settings,
            clear_memory,
            get_memory_context,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
