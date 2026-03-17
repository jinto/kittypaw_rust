"""Web chat routes with SSE streaming."""
import json
import uuid

from fastapi import APIRouter
from fastapi.responses import StreamingResponse
from pydantic import BaseModel

from src.core.types.event import Event, EventType
from src.agent_loop.loop import run_agent_loop

router = APIRouter()


class ChatRequest(BaseModel):
    message: str
    session_id: str | None = None


@router.post("/send")
async def send_message(request: ChatRequest) -> dict[str, object]:
    """Send a chat message and get an agent response."""
    session_id = request.session_id or str(uuid.uuid4())

    event = Event(
        type=EventType.WEB_CHAT,
        source="web",
        payload={
            "text": request.message,
            "session_id": session_id,
        },
    )

    result = await run_agent_loop(event)

    return {
        "session_id": session_id,
        "response": result.get("result", ""),
        "code": result.get("code", ""),
        "success": result.get("success", False),
    }
