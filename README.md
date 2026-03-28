# KittyPaw

**Program your bot by talking to it.**

KittyPaw is a single-binary AI agent where you teach new skills through conversation. Describe what you want in natural language, KittyPaw writes the code, runs it in a sandbox, and if you approve, it runs forever.

No workflow canvas. No drag-and-drop. Just conversation.

## Quickstart

```bash
# Install
curl -fsSL https://raw.githubusercontent.com/jinto/kittypaw/main/install.sh | bash

# Set up
kittypaw init

# Teach your first skill
kittypaw teach "every morning, summarize the top 5 Hacker News stories and send them to my Telegram"

# Or teach from Telegram
# Send /teach followed by your description to your bot
```

## How It Works

```
You: "send me a daily joke every morning at 9am"

KittyPaw:
  1. LLM generates JavaScript code for the task
  2. Code runs in a dual-layer sandbox (QuickJS VM + OS kernel isolation)
  3. You see the code, the dry-run output, and the permissions it needs
  4. You approve → skill saved to .kittypaw/skills/joke-every-morning.js
  5. Next morning at 9am, the skill runs automatically
```

## Features

- **Conversational programming** — describe what you want, KittyPaw writes the code
- **Teach loop** — `/teach` on Telegram or `kittypaw teach` from CLI
- **Dual-layer sandbox** — QuickJS VM isolation + macOS Seatbelt / Linux Landlock kernel isolation
- **5 primitives** — Telegram, Http, Storage, Llm, Schedule
- **Skill versioning** — skills saved as `.kittypaw/skills/*.skill.toml` + `*.js`, git-friendly
- **Schedule evaluator** — cron-based scheduling with auto-disable on failure
- **Per-skill storage** — each skill gets isolated key-value storage (SQLite)
- **Single binary, zero dependencies** — just `./kittypaw` and an API key

## Demo Examples

### Daily HN Digest
```bash
kittypaw teach "every morning at 9am, fetch the top 5 stories from Hacker News and send a summary to my Telegram"
```

### RSS Watcher
```bash
kittypaw teach "check the Rust blog RSS feed every hour and notify me on Telegram when there's a new post"
```

### GitHub Issue Notifier
```bash
kittypaw teach "when I say /issues, check my GitHub repo for new issues and list them"
```

## CLI Reference

```bash
kittypaw init                          # Set up API key and config
kittypaw teach <description>           # Teach a new skill (interactive)
kittypaw serve                         # Start bot server (Telegram + scheduler)
kittypaw run <name>                    # Run a skill manually
kittypaw run <name> --dry-run          # Dry-run a skill (no side effects)
kittypaw skills list                   # List all taught skills
kittypaw skills explain <name>         # LLM explains what a skill does
kittypaw skills disable <name>         # Disable a skill
kittypaw skills delete <name>          # Delete a skill
kittypaw skills import <path>          # Import skills from a directory
kittypaw config check                  # Validate configuration
```

## Architecture

KittyPaw is organized as a Cargo workspace with focused crates:

| Crate | Purpose |
|---|---|
| `kittypaw-core` | Skill file system, config, types, capability checking |
| `kittypaw-llm` | LLM client (Claude API), prompt construction |
| `kittypaw-sandbox` | QuickJS VM execution + OS-level isolation (Seatbelt/Landlock) |
| `kittypaw-channels` | Telegram channel adapter |
| `kittypaw-cli` | Binary entry point, teach loop, schedule evaluator, skill executor |

### Execution Flow

```
User message or /teach command
  → Channel adapter (Telegram / CLI / stdin)
  → Skill trigger matching (taught skills checked first)
  → If /teach: LLM generates JS → sandbox dry-run → approve → save to .kittypaw/skills/
  → If skill match: load JS from .kittypaw/skills/ → execute in sandbox
  → If no match + freeform enabled: LLM generates one-shot JS → execute
  → OS sandbox enforces filesystem/network policy
  → Result returned to channel
```

## Configuration

Copy `kittypaw.toml.example` to `kittypaw.toml` or run `kittypaw init`:

```toml
[llm]
provider = "claude"
api_key = ""          # or set KITTYPAW_API_KEY env var
model = "claude-sonnet-4-20250514"
max_tokens = 4096

[sandbox]
timeout_secs = 30
memory_limit_mb = 64
allowed_hosts = ["api.telegram.org"]

# Restrict /teach to specific Telegram users (empty = allow all)
admin_chat_ids = []

# Allow freeform LLM responses when no skill matches (default: false)
freeform_fallback = false

# Telegram channel
[[channels]]
channel_type = "telegram"
token = "123456:ABC-your-bot-token"
```

## 5 Primitives

Skills have access to 5 built-in primitives:

| Primitive | Methods | Description |
|-----------|---------|-------------|
| `Telegram` | `sendMessage`, `sendPhoto`, `editMessage` | Send messages via Telegram |
| `Http` | `get`, `post`, `put`, `delete` | HTTP requests (SSRF-protected) |
| `Storage` | `get`, `set`, `delete`, `list` | Per-skill key-value storage (SQLite) |
| `Llm` | `generate` | Call Claude API (max 3 per execution) |
| `Schedule` | (declarative) | Cron-based scheduling via skill TOML |

## Development

```bash
cargo build
cargo test
cargo clippy
```

Run with debug logging:

```bash
RUST_LOG=kittypaw=debug ./kittypaw serve
```

## License

MIT
