use kittypaw_store::Store;

use super::helpers::db_path;

pub(crate) fn run_suggestions_list() {
    let db_path = db_path();
    let store = match Store::open(&db_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Database error: {e}");
            return;
        }
    };

    match store.pending_suggestions() {
        Ok(suggestions) if suggestions.is_empty() => {
            println!("No pending suggestions.");
        }
        Ok(suggestions) => {
            println!("=== Pending Suggestions ===\n");
            for sg in &suggestions {
                println!(
                    "  {} ({}) — {} cron: {}",
                    sg.skill_name, sg.skill_id, sg.suggestion_type, sg.suggested_cron
                );
            }
            println!(
                "\nAccept: kittypaw suggestions accept <skill_id>\nDismiss: kittypaw suggestions dismiss <skill_id>"
            );
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}

pub(crate) fn run_suggestions_accept(skill_id: &str) {
    let db_path = db_path();
    let store = match Store::open(&db_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Database error: {e}");
            return;
        }
    };

    match store.accept_suggestion(skill_id) {
        Ok(Some(cron)) => {
            println!("Accepted! Skill '{}' now scheduled: {}", skill_id, cron);
        }
        Ok(None) => {
            eprintln!("No time pattern detected for '{}'.", skill_id);
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}

pub(crate) fn run_suggestions_dismiss(skill_id: &str) {
    let db_path = db_path();
    let store = match Store::open(&db_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Database error: {e}");
            return;
        }
    };

    let key = format!("suggest_dismissed:{}", skill_id);
    match store.set_user_context(&key, "1", "user") {
        Ok(()) => println!("Dismissed suggestion for '{}'.", skill_id),
        Err(e) => eprintln!("Error: {e}"),
    }
}
