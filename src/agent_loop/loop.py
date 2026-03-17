import logging

from src.core.types.agent_state import AgentState
from src.core.types.event import Event
from src.core.utils.s3 import load_state, save_state
from src.agent_loop.executor import execute_code
from src.agent_loop.llm import generate_code
from src.agent_loop.prompt import build_prompt

logger = logging.getLogger(__name__)

MAX_TYPE_CHECK_RETRIES = 3


async def run_agent_loop(event: Event) -> dict[str, object]:
    """Core agent loop: load state → prompt LLM → type check → execute → save state.

    This is the beating heart of Oochy. An event arrives, the LLM writes code
    to handle it, the code is verified and executed, and the world moves forward.
    """
    # 1. Load agent state from S3
    state = load_state(event.agent_id)

    # 2. Record incoming event
    state.add_turn(role="user", content=str(event.payload))

    # 3. Build prompt and call LLM
    messages = build_prompt(state, event)
    code = await generate_code(messages)

    logger.info("LLM generated code:\n%s", code)

    # 4. Execute in sandbox (with type-check retry loop)
    result = await _execute_with_retries(code, messages, event)

    # 5. Record result in state
    state.add_turn(
        role="assistant",
        content=result.get("result", ""),
        code=code,
        result=str(result.get("result", "")),
    )

    # 6. Save state back to S3
    save_state(state)

    return {
        "success": result.get("success", False),
        "code": code,
        "result": result.get("result", ""),
        "agent_id": event.agent_id,
    }


async def _execute_with_retries(
    code: str,
    messages: list[dict[str, str]],
    event: Event,
) -> dict[str, object]:
    """Type-check and execute code, retrying with LLM feedback on type errors."""
    for attempt in range(MAX_TYPE_CHECK_RETRIES):
        result = await execute_code(code, context={"event": event.model_dump()})

        if result.get("success"):
            return result

        type_errors = result.get("type_errors", [])
        if not type_errors:
            # Runtime error, not type error — don't retry
            return result

        # Feed type errors back to LLM for correction
        logger.warning("Type check failed (attempt %d): %s", attempt + 1, type_errors)
        error_feedback = f"Your code had type errors:\n" + "\n".join(str(e) for e in type_errors) + "\nPlease fix the code."
        messages.append({"role": "user", "content": error_feedback})
        code = await generate_code(messages)

    return result
