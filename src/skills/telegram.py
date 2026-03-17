"""Telegram skill runtime — actual API calls to Telegram Bot API."""
import os

import httpx

BOT_TOKEN = os.environ.get("TELEGRAM_BOT_TOKEN", "")
BASE_URL = f"https://api.telegram.org/bot{BOT_TOKEN}"


class Telegram:
    @staticmethod
    async def send_message(chat_id: str, text: str) -> dict[str, int]:
        async with httpx.AsyncClient() as client:
            resp = await client.post(
                f"{BASE_URL}/sendMessage",
                json={"chat_id": chat_id, "text": text, "parse_mode": "Markdown"},
            )
            data = resp.json()
            return {"message_id": data.get("result", {}).get("message_id", 0)}

    @staticmethod
    async def send_voice(chat_id: str, audio_url: str) -> dict[str, int]:
        async with httpx.AsyncClient() as client:
            resp = await client.post(
                f"{BASE_URL}/sendVoice",
                json={"chat_id": chat_id, "voice": audio_url},
            )
            data = resp.json()
            return {"message_id": data.get("result", {}).get("message_id", 0)}

    @staticmethod
    def get_chat_context() -> dict[str, str]:
        # Populated by the sandbox runtime from event context
        return {}
