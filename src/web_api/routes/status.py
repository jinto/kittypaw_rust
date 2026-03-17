"""Agent status and monitoring routes."""
from fastapi import APIRouter

from src.core.utils.s3 import load_state

router = APIRouter()


@router.get("/agent/{agent_id}")
async def get_agent_status(agent_id: str = "default") -> dict[str, object]:
    """Get the current status of an agent."""
    state = load_state(agent_id)
    return {
        "agent_id": state.agent_id,
        "turns": len(state.conversation),
        "created_at": state.created_at.isoformat(),
        "updated_at": state.updated_at.isoformat(),
        "recent_turns": [
            {
                "role": t.role,
                "content": t.content[:200],
                "code": t.code[:200] if t.code else None,
                "timestamp": t.timestamp.isoformat(),
            }
            for t in state.recent_turns(5)
        ],
    }
