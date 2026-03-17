from datetime import UTC, datetime

from pydantic import BaseModel, Field


class ConversationTurn(BaseModel):
    role: str
    content: str
    timestamp: datetime = Field(default_factory=lambda: datetime.now(UTC))
    code: str | None = None
    result: str | None = None


class AgentState(BaseModel):
    agent_id: str = "default"
    conversation: list[ConversationTurn] = Field(default_factory=list)
    created_at: datetime = Field(default_factory=lambda: datetime.now(UTC))
    updated_at: datetime = Field(default_factory=lambda: datetime.now(UTC))
    metadata: dict[str, str | int | float | bool | None] = Field(default_factory=dict)

    def add_turn(self, role: str, content: str, code: str | None = None, result: str | None = None) -> None:
        self.conversation.append(
            ConversationTurn(role=role, content=content, code=code, result=result)
        )
        self.updated_at = datetime.now(UTC)

    def recent_turns(self, n: int = 20) -> list[ConversationTurn]:
        return self.conversation[-n:]
