use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use kittypaw_core::permission::PermissionDecision;
use kittypaw_store::Store;
use kittypaw_workspace::{FileIndexer, FilePermissionChecker, WorkspaceManager};

pub struct AppState {
    pub store: Arc<Mutex<Store>>,
    pub api_key: Arc<Mutex<String>>,
    pub workspace_manager: Arc<Mutex<WorkspaceManager>>,
    pub file_indexer: Arc<Mutex<Option<FileIndexer>>>,
    pub permission_checker: Arc<Mutex<FilePermissionChecker>>,
    pub permission_requests:
        Arc<Mutex<HashMap<String, tokio::sync::oneshot::Sender<PermissionDecision>>>>,
}

impl AppState {
    pub fn new(store: Store) -> Self {
        Self {
            store: Arc::new(Mutex::new(store)),
            api_key: Arc::new(Mutex::new(String::new())),
            workspace_manager: Arc::new(Mutex::new(WorkspaceManager::new())),
            file_indexer: Arc::new(Mutex::new(None)),
            permission_checker: Arc::new(Mutex::new(FilePermissionChecker::permissive())),
            permission_requests: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
