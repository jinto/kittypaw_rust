#!/usr/bin/env python3
"""CDK app entry point for Oochy infrastructure."""
import aws_cdk as cdk

from infra.stacks.storage_stack import StorageStack
from infra.stacks.agent_stack import AgentStack
from infra.stacks.api_stack import ApiStack
from infra.stacks.mqtt_stack import MqttStack

app = cdk.App()

env = cdk.Environment(
    account=app.node.try_get_context("account"),
    region=app.node.try_get_context("region") or "ap-northeast-2",
)

storage = StorageStack(app, "OochyStorage", env=env)
agent = AgentStack(app, "OochyAgent", storage=storage, env=env)
api = ApiStack(app, "OochyApi", storage=storage, env=env)
mqtt = MqttStack(app, "OochyMqtt", env=env)

app.synth()
