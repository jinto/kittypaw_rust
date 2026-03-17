from src.core.types.agent_state import AgentState
from src.core.types.event import Event
from src.agent_loop.skill_registry import build_skill_context

SYSTEM_PROMPT = """\
You are Oochy, an AI agent that helps users by writing Python code.

## How you work
1. You receive an event (message, command, etc.)
2. You write Python code to handle it
3. Your code is type-checked with mypy, then executed in a sandbox
4. The result is returned to the user

## Rules
- Write ONLY valid Python code. No markdown fences, no explanations.
- Use the available skill classes to interact with the outside world.
- All async skill methods must be awaited: `await Telegram.send_message(...)`
- Your code runs in an async context — top-level `await` is allowed.
- If you need to store data for later, return a dict with a "state" key.
- Keep your code minimal and focused on the task.
- Handle errors gracefully with try/except.

## Available Skills
{skills}
"""


def build_prompt(state: AgentState, event: Event) -> list[dict[str, str]]:
    """Build the full prompt for the LLM including skills, history, and current event."""
    skills_context = build_skill_context()

    messages: list[dict[str, str]] = [
        {"role": "system", "content": SYSTEM_PROMPT.format(skills=skills_context)},
    ]

    # Add recent conversation history
    for turn in state.recent_turns(20):
        if turn.role == "user":
            content = turn.content
            if turn.result:
                content += f"\n[Previous result: {turn.result}]"
            messages.append({"role": "user", "content": content})
        elif turn.role == "assistant":
            messages.append({"role": "assistant", "content": turn.code or turn.content})

    # Add current event as user message
    event_text = _format_event(event)
    messages.append({"role": "user", "content": event_text})

    return messages


def _format_event(event: Event) -> str:
    """Format an event into a user message."""
    payload = event.payload

    if event.type == "telegram_message":
        user = payload.get("from_name", "User")
        text = payload.get("text", "")
        chat_id = payload.get("chat_id", "")
        return f"[Telegram] {user} (chat_id={chat_id}): {text}"

    if event.type == "web_chat":
        text = payload.get("text", "")
        session = payload.get("session_id", "")
        return f"[WebChat] (session={session}): {text}"

    if event.type == "mqtt_command":
        return f"[Desktop Command] {payload}"

    return f"[{event.type}] {payload}"
