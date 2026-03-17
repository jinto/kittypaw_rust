"""Tests for code sandbox runner."""
import pytest
from src.code_sandbox.runner import run_code


@pytest.mark.asyncio
async def test_run_simple_code() -> None:
    result = await run_code('print("hello world")')
    assert result["success"] is True
    assert "hello world" in result["result"]


@pytest.mark.asyncio
async def test_run_code_with_error() -> None:
    result = await run_code('raise ValueError("test error")')
    assert result["success"] is False


@pytest.mark.asyncio
async def test_run_code_with_context() -> None:
    result = await run_code(
        'print(_context["name"])',
        context={"name": "oochy"},
    )
    assert result["success"] is True
    assert "oochy" in result["result"]
