from datetime import UTC, datetime
from enum import StrEnum

from pydantic import BaseModel, Field


class EventType(StrEnum):
    TELEGRAM_MESSAGE = "telegram_message"
    WEB_CHAT = "web_chat"
    MQTT_COMMAND = "mqtt_command"
    SYSTEM = "system"
    SCHEDULE = "schedule"


class Event(BaseModel):
    id: str = Field(default_factory=lambda: "")
    type: EventType
    source: str
    payload: dict[str, str | int | float | bool | None]
    timestamp: datetime = Field(default_factory=lambda: datetime.now(UTC))
    agent_id: str = "default"

    def model_post_init(self, __context: object) -> None:
        if not self.id:
            import uuid
            self.id = str(uuid.uuid4())
