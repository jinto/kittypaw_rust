"""Tests for prompt builder."""
from src.core.types.agent_state import AgentState
from src.core.types.event import Event, EventType
from src.agent_loop.prompt import build_prompt


def test_build_prompt_basic() -> None:
    state = AgentState()
    event = Event(
        type=EventType.TELEGRAM_MESSAGE,
        source="telegram",
        payload={"text": "hello", "chat_id": "123", "from_name": "Jinto"},
    )

    messages = build_prompt(state, event)

    assert messages[0]["role"] == "system"
    assert "Oochy" in messages[0]["content"]
    assert messages[-1]["role"] == "user"
    assert "Jinto" in messages[-1]["content"]
    assert "hello" in messages[-1]["content"]


def test_build_prompt_with_history() -> None:
    state = AgentState()
    state.add_turn(role="user", content="previous message")
    state.add_turn(role="assistant", content="response", code="print('hi')")

    event = Event(
        type=EventType.WEB_CHAT,
        source="web",
        payload={"text": "new message", "session_id": "s1"},
    )

    messages = build_prompt(state, event)

    # system + 2 history + current event = 4
    assert len(messages) == 4
    assert messages[1]["content"] == "previous message"
    assert messages[2]["content"] == "print('hi')"
