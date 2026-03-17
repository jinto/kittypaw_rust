"""Voice skill runtime — STT and TTS via external APIs."""


class Voice:
    @staticmethod
    async def transcribe(audio_url: str) -> dict[str, str]:
        # TODO: integrate Whisper API or AWS Transcribe
        return {"text": ""}

    @staticmethod
    async def synthesize(text: str, voice: str = "default") -> dict[str, str]:
        # TODO: integrate TTS API (OpenAI, ElevenLabs, or AWS Polly)
        return {"audio_url": ""}
