import os

import anthropic


async def generate_code(messages: list[dict[str, str]], max_retries: int = 1) -> str:
    """Call Claude API to generate Python code from the prompt."""
    client = anthropic.AsyncAnthropic(api_key=os.environ.get("ANTHROPIC_API_KEY", ""))

    system_message = ""
    api_messages: list[dict[str, str]] = []

    for msg in messages:
        if msg["role"] == "system":
            system_message = msg["content"]
        else:
            api_messages.append(msg)

    response = await client.messages.create(
        model=os.environ.get("LLM_MODEL", "claude-sonnet-4-20250514"),
        max_tokens=4096,
        system=system_message,
        messages=api_messages,
    )

    # Extract text content
    code = ""
    for block in response.content:
        if block.type == "text":
            code += block.text

    # Strip markdown fences if LLM wraps code
    code = _strip_code_fences(code)

    return code


def _strip_code_fences(text: str) -> str:
    """Remove markdown code fences if present."""
    text = text.strip()
    if text.startswith("```python"):
        text = text[len("```python"):]
    elif text.startswith("```"):
        text = text[3:]
    if text.endswith("```"):
        text = text[:-3]
    return text.strip()
