import json
import os

import boto3

IOT_ENDPOINT = os.environ.get("IOT_ENDPOINT", "")


def _client() -> boto3.client:
    return boto3.client("iot-data", endpoint_url=f"https://{IOT_ENDPOINT}" if IOT_ENDPOINT else None)


def publish(topic: str, payload: dict[str, object]) -> None:
    _client().publish(
        topic=topic,
        qos=1,
        payload=json.dumps(payload),
    )
