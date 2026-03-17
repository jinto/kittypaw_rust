class Telegram:
    """Telegram messaging skill. Use this to send and receive messages via Telegram."""

    @staticmethod
    async def send_message(chat_id: str, text: str) -> dict[str, int]:
        """Send a text message to a Telegram chat. Returns {"message_id": int}."""
        ...

    @staticmethod
    async def send_voice(chat_id: str, audio_url: str) -> dict[str, int]:
        """Send a voice message to a Telegram chat. Returns {"message_id": int}."""
        ...

    @staticmethod
    def get_chat_context() -> dict[str, str]:
        """Get the current chat context including chat_id and user info."""
        ...
