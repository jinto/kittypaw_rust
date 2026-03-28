use kittypaw_core::types::{LlmMessage, Role};
use kittypaw_llm::claude::ClaudeProvider;
use kittypaw_llm::provider::LlmProvider;
use kittypaw_workspace::{SearchResult, SemanticResult};
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub fn search_files(
    query: String,
    workspace_id: String,
    state: State<AppState>,
) -> Result<Vec<SearchResult>, String> {
    let _ = workspace_id; // index is global to last opened workspace
    let indexer_guard = state.file_indexer.lock().unwrap();
    match indexer_guard.as_ref() {
        Some(indexer) => indexer.search(&query, 20).map_err(|e| e.to_string()),
        None => Err("No index available. Open a workspace first.".to_string()),
    }
}

#[tauri::command]
pub async fn semantic_search(
    query: String,
    workspace_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<SemanticResult>, String> {
    let api_key = {
        let key = state.api_key.lock().unwrap();
        key.clone()
    };

    if api_key.is_empty() {
        return Err("API key not configured. Please set your API key in Settings.".to_string());
    }

    let (files, workspace_root) = {
        let mgr = state.workspace_manager.lock().unwrap();
        let ws = mgr
            .workspaces_iter()
            .find(|(id, _)| *id == workspace_id.as_str())
            .map(|(_, ws)| ws.clone())
            .ok_or_else(|| format!("Workspace not found: {workspace_id}"))?;

        let root = std::path::PathBuf::from(&ws.root_path);
        let files = mgr.list_files(&workspace_id).map_err(|e| e.to_string())?;
        (files, root)
    };

    let provider = ClaudeProvider::new(api_key, "claude-haiku-4-20250514".to_string(), 1024);

    kittypaw_workspace::semantic::semantic_search(
        &query,
        &files,
        &workspace_root,
        |prompt| async move {
            let messages = vec![LlmMessage {
                role: Role::User,
                content: prompt,
            }];
            provider
                .generate(&messages)
                .await
                .map_err(|e| kittypaw_core::error::KittypawError::Sandbox(e.to_string()))
        },
    )
    .await
    .map_err(|e| e.to_string())
}
