import json
import os

import boto3

SANDBOX_FUNCTION = os.environ.get("SANDBOX_FUNCTION", "oochy-code-sandbox")


async def execute_code(code: str, context: dict[str, object] | None = None) -> dict[str, object]:
    """Send code to the sandbox Lambda for type-checking and execution.

    Returns a dict with:
        - success: bool
        - result: str (execution output or error)
        - type_errors: list[str] (mypy errors if any)
    """
    client = boto3.client("lambda")

    payload = {
        "code": code,
        "context": context or {},
    }

    response = client.invoke(
        FunctionName=SANDBOX_FUNCTION,
        InvocationType="RequestResponse",
        Payload=json.dumps(payload),
    )

    result = json.loads(response["Payload"].read())

    if "errorMessage" in result:
        return {
            "success": False,
            "result": result["errorMessage"],
            "type_errors": [],
        }

    return result
