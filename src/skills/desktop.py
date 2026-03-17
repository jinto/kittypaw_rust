"""Desktop control skill runtime — sends commands via MQTT to macOS app."""
import json

from src.core.utils.mqtt import publish

TOPIC_PREFIX = "oochy/desktop"


class Desktop:
    @staticmethod
    async def bash(command: str) -> dict[str, str | int]:
        publish(f"{TOPIC_PREFIX}/command", {
            "type": "bash",
            "command": command,
        })
        # TODO: await response via MQTT reply topic
        return {"stdout": "", "stderr": "", "exit_code": -1}

    @staticmethod
    async def apple_script(script: str) -> dict[str, str]:
        publish(f"{TOPIC_PREFIX}/command", {
            "type": "applescript",
            "script": script,
        })
        return {"result": ""}

    @staticmethod
    def is_connected() -> bool:
        # TODO: check IoT Core device shadow
        return False
