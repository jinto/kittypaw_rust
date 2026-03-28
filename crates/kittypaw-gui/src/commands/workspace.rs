use kittypaw_core::workspace::{FileChange, FileEntry, Workspace};
use kittypaw_workspace::FileIndexer;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub fn open_workspace(path: String, state: State<AppState>) -> Result<Workspace, String> {
    let ws = {
        let mut mgr = state.workspace_manager.lock().unwrap();
        mgr.open(&path).map_err(|e| e.to_string())?
    };

    // Build tantivy index in a background thread
    let ws_root = std::path::PathBuf::from(&ws.root_path);
    let ws_id = ws.id.clone();
    let indexer_arc = state.file_indexer.clone();
    let workspace_manager_arc = state.workspace_manager.clone();

    std::thread::spawn(move || {
        let index_path = dirs_next::data_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("kittypaw")
            .join("index")
            .join(&ws_id);

        let mut indexer = match FileIndexer::new(&index_path) {
            Ok(i) => i,
            Err(e) => {
                tracing::warn!("Failed to create file indexer: {e}");
                return;
            }
        };

        let files = {
            let mgr = workspace_manager_arc.lock().unwrap();
            match mgr.list_files(&ws_id) {
                Ok(f) => f,
                Err(e) => {
                    tracing::warn!("Failed to list files for indexing: {e}");
                    return;
                }
            }
        };

        if let Err(e) = indexer.build_index(&ws_root, &files) {
            tracing::warn!("Failed to build index: {e}");
            return;
        }

        let mut guard = indexer_arc.lock().unwrap();
        *guard = Some(indexer);
        tracing::info!("File index built for workspace {ws_id}");
    });

    Ok(ws)
}

#[tauri::command]
pub fn list_files(workspace_id: String, state: State<AppState>) -> Result<Vec<FileEntry>, String> {
    let mgr = state.workspace_manager.lock().unwrap();
    mgr.list_files(&workspace_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn read_file(
    workspace_id: String,
    path: String,
    state: State<AppState>,
) -> Result<String, String> {
    let mgr = state.workspace_manager.lock().unwrap();
    mgr.read_file(&workspace_id, &path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn preview_change(
    workspace_id: String,
    path: String,
    content: String,
    state: State<AppState>,
) -> Result<FileChange, String> {
    let mut mgr = state.workspace_manager.lock().unwrap();
    mgr.write_file(&workspace_id, &path, &content)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn approve_change(change_id: String, state: State<AppState>) -> Result<(), String> {
    let mut mgr = state.workspace_manager.lock().unwrap();
    let change = mgr
        .get_change(&change_id)
        .map_err(|e| e.to_string())?
        .clone();
    mgr.apply_change(&change).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reject_change(change_id: String, state: State<AppState>) -> Result<(), String> {
    let mut mgr = state.workspace_manager.lock().unwrap();
    mgr.reject_change(&change_id).map_err(|e| e.to_string())
}
