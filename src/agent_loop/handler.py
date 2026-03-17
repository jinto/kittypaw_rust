"""AWS Lambda handler for the Agent Loop.

Triggered by SQS events. Each message contains an Event to process.
"""
import asyncio
import json
import logging

from src.core.types.event import Event
from src.agent_loop.loop import run_agent_loop

logger = logging.getLogger(__name__)
logger.setLevel(logging.INFO)


def handler(event: dict, context: object) -> dict[str, object]:
    """Lambda entry point. Processes SQS messages through the agent loop."""
    results: list[dict[str, object]] = []

    for record in event.get("Records", []):
        body = json.loads(record["body"])
        agent_event = Event.model_validate(body)

        logger.info("Processing event: %s (type=%s)", agent_event.id, agent_event.type)

        result = asyncio.get_event_loop().run_until_complete(
            run_agent_loop(agent_event)
        )
        results.append(result)

    return {
        "statusCode": 200,
        "body": json.dumps({"processed": len(results), "results": results}, default=str),
    }
