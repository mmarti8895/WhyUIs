//! WhyUIs — AI Roast Chatbot
//! Tauri application library entry point.

mod chat;
mod memory;
mod persona;

use chat::{ChatSettings, HistoryMessage};
use memory::MemoryStore;
use std::sync::Arc;
use tauri::State;

/// Global app state.
pub struct AppState {
    pub memory: Arc<MemoryStore>,
}

/// Tauri command: send a message and get a roasted response.
#[tauri::command]
async fn send_message(
    message: String,
    session_id: String,
    settings: ChatSettings,
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

    // Call LLM
    let response = chat::send_to_llm(&message, &history, &settings, &memory_context).await?;

    // Store assistant response in memory
    state.memory.push_message("assistant", &response);

    Ok(response)
}

/// Tauri command: clear memory for the current user.
#[tauri::command]
fn clear_memory(state: State<'_, AppState>) -> Result<(), String> {
    let mut st = state.memory.short_term.lock().unwrap();
    st.clear();
    Ok(())
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
        })
        .invoke_handler(tauri::generate_handler![
            send_message,
            clear_memory,
            get_memory_context,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
