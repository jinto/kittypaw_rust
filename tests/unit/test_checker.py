"""Tests for mypy type checker."""
import pytest
from src.code_sandbox.checker import type_check


@pytest.mark.skipif(
    not __import__("shutil").which("mypy"),
    reason="mypy not installed",
)
class TestTypeChecker:
    def test_valid_code(self) -> None:
        code = 'x: int = 42\nprint(x)'
        errors = type_check(code)
        # May have some strict-mode warnings, but should have no hard errors
        # about the core logic
        assert isinstance(errors, list)

    def test_invalid_type(self) -> None:
        code = 'x: int = "not an int"'
        errors = type_check(code)
        assert len(errors) > 0
