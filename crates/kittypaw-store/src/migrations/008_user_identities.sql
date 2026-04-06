-- Cross-channel user identity mapping.
-- Links a channel-specific user (telegram chat_id, web session, etc.)
-- to a global user ID for shared conversation context.
CREATE TABLE IF NOT EXISTS user_identities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    global_user_id TEXT NOT NULL,
    channel TEXT NOT NULL,
    channel_user_id TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(channel, channel_user_id)
);

CREATE INDEX IF NOT EXISTS idx_user_identities_global
    ON user_identities(global_user_id);
