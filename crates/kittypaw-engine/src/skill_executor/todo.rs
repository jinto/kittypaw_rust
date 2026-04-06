use kittypaw_core::error::{KittypawError, Result};
use kittypaw_core::types::SkillCall;

/// Todo tool: in-memory task decomposition that survives context compaction.
/// Tasks are persisted in Storage so they outlive individual turns.
pub(super) fn execute_todo(
    call: &SkillCall,
    store: &kittypaw_store::Store,
) -> Result<serde_json::Value> {
    let todo_key = "todo:current";

    match call.method.as_str() {
        "add" => {
            let task = call.args.first().and_then(|v| v.as_str()).ok_or_else(|| {
                KittypawError::Skill("Todo.add: task description required".into())
            })?;

            let mut todos = load_todos(store);
            todos.push(TodoItem {
                text: task.to_string(),
                done: false,
            });
            save_todos(store, &todos)?;
            Ok(serde_json::json!({"added": task, "total": todos.len()}))
        }
        "done" => {
            let index = call
                .args
                .first()
                .and_then(|v| v.as_u64())
                .ok_or_else(|| KittypawError::Skill("Todo.done: index required".into()))?
                as usize;

            let mut todos = load_todos(store);
            if index >= todos.len() {
                return Err(KittypawError::Skill(format!(
                    "Todo.done: index {index} out of range (total: {})",
                    todos.len()
                )));
            }
            todos[index].done = true;
            save_todos(store, &todos)?;
            Ok(serde_json::json!({"completed": todos[index].text, "index": index}))
        }
        "list" => {
            let todos = load_todos(store);
            let items: Vec<serde_json::Value> = todos
                .iter()
                .enumerate()
                .map(|(i, t)| {
                    serde_json::json!({
                        "index": i,
                        "text": t.text,
                        "done": t.done,
                    })
                })
                .collect();
            Ok(serde_json::json!({"todos": items, "total": todos.len()}))
        }
        "clear" => {
            store.set_user_context(todo_key, "[]", "todo")?;
            Ok(serde_json::json!({"cleared": true}))
        }
        _ => Err(KittypawError::CapabilityDenied(format!(
            "Unknown Todo method: {}",
            call.method
        ))),
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct TodoItem {
    text: String,
    done: bool,
}

fn load_todos(store: &kittypaw_store::Store) -> Vec<TodoItem> {
    store
        .get_user_context("todo:current")
        .ok()
        .flatten()
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
}

fn save_todos(store: &kittypaw_store::Store, todos: &[TodoItem]) -> Result<()> {
    let json = serde_json::to_string(todos)
        .map_err(|e| KittypawError::Skill(format!("Todo serialize error: {e}")))?;
    store.set_user_context("todo:current", &json, "todo")
}
