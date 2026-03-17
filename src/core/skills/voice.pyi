class Voice:
    """Voice processing skill. Handles speech-to-text and text-to-speech."""

    @staticmethod
    async def transcribe(audio_url: str) -> dict[str, str]:
        """Transcribe audio to text. Returns {"text": str}."""
        ...

    @staticmethod
    async def synthesize(text: str, voice: str = "default") -> dict[str, str]:
        """Convert text to speech. Returns {"audio_url": str}."""
        ...
