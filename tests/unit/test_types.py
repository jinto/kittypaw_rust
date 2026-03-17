"""Tests for core type models."""
from src.core.types.agent_state import AgentState
from src.core.types.event import Event, EventType
from src.core.types.message import LLMMessage, Role
from src.core.types.skill import SkillDefinition


def test_event_creation() -> None:
    event = Event(
        type=EventType.TELEGRAM_MESSAGE,
        source="telegram",
        payload={"text": "hello", "chat_id": "123"},
    )
    assert event.type == EventType.TELEGRAM_MESSAGE
    assert event.payload["text"] == "hello"
    assert event.id  # auto-generated


def test_event_custom_id() -> None:
    event = Event(
        id="custom-id",
        type=EventType.SYSTEM,
        source="test",
        payload={},
    )
    assert event.id == "custom-id"


def test_agent_state_add_turn() -> None:
    state = AgentState(agent_id="test")
    state.add_turn(role="user", content="hello")
    state.add_turn(role="assistant", content="hi", code="print('hi')")

    assert len(state.conversation) == 2
    assert state.conversation[0].role == "user"
    assert state.conversation[1].code == "print('hi')"


def test_agent_state_recent_turns() -> None:
    state = AgentState()
    for i in range(30):
        state.add_turn(role="user", content=f"msg {i}")

    recent = state.recent_turns(5)
    assert len(recent) == 5
    assert recent[0].content == "msg 25"


def test_llm_message() -> None:
    msg = LLMMessage(role=Role.USER, content="test")
    assert msg.role == "user"


def test_skill_definition() -> None:
    skill = SkillDefinition(
        name="test",
        description="A test skill",
        stub_code="class Test: ...",
    )
    assert skill.enabled is True
    assert skill.name == "test"
