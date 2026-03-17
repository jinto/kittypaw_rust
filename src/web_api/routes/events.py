"""Telegram webhook and external event routes."""
import json
import os

import boto3
from fastapi import APIRouter, Request

from src.core.types.event import Event, EventType

router = APIRouter()

SQS_QUEUE_URL = os.environ.get("SQS_QUEUE_URL", "")


@router.post("/telegram/webhook")
async def telegram_webhook(request: Request) -> dict[str, str]:
    """Receive Telegram webhook updates and queue them for processing."""
    body = await request.json()

    message = body.get("message", {})
    if not message:
        return {"status": "ignored"}

    text = message.get("text", "")
    chat_id = str(message.get("chat", {}).get("id", ""))
    from_user = message.get("from", {})
    from_name = from_user.get("first_name", "Unknown")

    event = Event(
        type=EventType.TELEGRAM_MESSAGE,
        source="telegram",
        payload={
            "text": text,
            "chat_id": chat_id,
            "from_name": from_name,
        },
    )

    # Queue the event for the agent loop
    _enqueue_event(event)

    return {"status": "queued"}


def _enqueue_event(event: Event) -> None:
    """Send an event to the SQS queue."""
    if not SQS_QUEUE_URL:
        return

    sqs = boto3.client("sqs")
    sqs.send_message(
        QueueUrl=SQS_QUEUE_URL,
        MessageBody=event.model_dump_json(),
        MessageGroupId=event.agent_id,
        MessageDeduplicationId=event.id,
    )
