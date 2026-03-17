"""AWS Lambda handler for the Code Sandbox.

Receives code, runs mypy type-check, then executes in a subprocess.
"""
import asyncio
import json
import logging

from src.code_sandbox.checker import type_check
from src.code_sandbox.runner import run_code

logger = logging.getLogger(__name__)
logger.setLevel(logging.INFO)


def handler(event: dict, context: object) -> dict[str, object]:
    """Lambda entry point for code sandbox."""
    code = event.get("code", "")
    exec_context = event.get("context", {})

    if not code.strip():
        return {
            "success": False,
            "result": "No code provided",
            "type_errors": [],
        }

    # Step 1: Type check with mypy
    logger.info("Type-checking code (%d chars)", len(code))
    type_errors = type_check(code)

    if type_errors:
        logger.warning("Type check failed: %s", type_errors)
        return {
            "success": False,
            "result": "Type check failed",
            "type_errors": type_errors,
        }

    # Step 2: Execute in sandbox
    logger.info("Executing code in sandbox")
    result = asyncio.get_event_loop().run_until_complete(
        run_code(code, context=exec_context)
    )

    return {
        "success": result.get("success", False),
        "result": result.get("result", ""),
        "type_errors": [],
    }
