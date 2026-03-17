class Chat:
    """Web chat skill. Use this to send messages to the web chat interface."""

    @staticmethod
    async def send_message(session_id: str, text: str) -> dict[str, str]:
        """Send a message to a web chat session. Returns {"status": "sent"}."""
        ...

    @staticmethod
    def get_session_context() -> dict[str, str]:
        """Get the current web chat session context."""
        ...
