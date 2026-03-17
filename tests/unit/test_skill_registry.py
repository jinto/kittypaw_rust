"""Tests for skill registry."""
from src.agent_loop.skill_registry import load_skill_stubs, build_skill_context


def test_load_skill_stubs() -> None:
    skills = load_skill_stubs()
    assert len(skills) >= 4  # telegram, chat, desktop, voice
    names = {s.name for s in skills}
    assert "telegram" in names
    assert "desktop" in names


def test_build_skill_context() -> None:
    context = build_skill_context()
    assert "Telegram" in context
    assert "Desktop" in context
    assert "send_message" in context
