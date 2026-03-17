import json
import os

import boto3

from src.core.types.agent_state import AgentState

BUCKET = os.environ.get("STATE_BUCKET", "oochy-state")


def _client() -> boto3.client:
    return boto3.client("s3")


def load_state(agent_id: str = "default") -> AgentState:
    key = f"state/{agent_id}/latest.json"
    try:
        resp = _client().get_object(Bucket=BUCKET, Key=key)
        data = json.loads(resp["Body"].read())
        return AgentState.model_validate(data)
    except _client().exceptions.NoSuchKey:
        return AgentState(agent_id=agent_id)
    except Exception:
        return AgentState(agent_id=agent_id)


def save_state(state: AgentState) -> None:
    key = f"state/{state.agent_id}/latest.json"
    _client().put_object(
        Bucket=BUCKET,
        Key=key,
        Body=state.model_dump_json(indent=2),
        ContentType="application/json",
    )
