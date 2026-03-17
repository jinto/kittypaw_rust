"""Web chat skill runtime."""


class Chat:
    @staticmethod
    async def send_message(session_id: str, text: str) -> dict[str, str]:
        # In production, this would push to a WebSocket/SSE connection
        return {"status": "sent"}

    @staticmethod
    def get_session_context() -> dict[str, str]:
        return {}
