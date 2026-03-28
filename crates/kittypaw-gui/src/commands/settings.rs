use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> String {
    let api_key = state.api_key.lock().unwrap();
    if api_key.is_empty() {
        return String::new();
    }
    // Return masked key — never expose full key to frontend
    let len = api_key.len();
    let suffix = &api_key[len.saturating_sub(4)..];
    format!("sk-...{suffix}")
}

#[tauri::command]
pub fn save_api_key(api_key: String, state: State<AppState>) {
    let mut key = state.api_key.lock().unwrap();
    *key = api_key;
}

/// Internal-only: get the real API key for backend use. Not a Tauri command.
pub fn get_api_key_internal(state: &State<AppState>) -> String {
    state.api_key.lock().unwrap().clone()
}
