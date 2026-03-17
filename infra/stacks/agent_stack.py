"""Agent Loop + Code Sandbox Lambda stack."""
import aws_cdk as cdk
from aws_cdk import (
    aws_lambda as _lambda,
    aws_lambda_event_sources as event_sources,
    Duration,
)
from constructs import Construct

from infra.stacks.storage_stack import StorageStack


class AgentStack(cdk.Stack):
    def __init__(
        self,
        scope: Construct,
        construct_id: str,
        storage: StorageStack,
        **kwargs: object,
    ) -> None:
        super().__init__(scope, construct_id, **kwargs)

        # Code Sandbox Lambda
        self.sandbox_fn = _lambda.Function(
            self, "CodeSandbox",
            function_name="oochy-code-sandbox",
            runtime=_lambda.Runtime.PYTHON_3_13,
            handler="src.code_sandbox.handler.handler",
            code=_lambda.Code.from_asset(".", exclude=["web", "apps", "infra", ".venv", "cdk.out"]),
            timeout=Duration.seconds(60),
            memory_size=512,
            environment={
                "STATE_BUCKET": storage.state_bucket.bucket_name,
            },
        )

        # Agent Loop Lambda
        self.agent_fn = _lambda.Function(
            self, "AgentLoop",
            function_name="oochy-agent-loop",
            runtime=_lambda.Runtime.PYTHON_3_13,
            handler="src.agent_loop.handler.handler",
            code=_lambda.Code.from_asset(".", exclude=["web", "apps", "infra", ".venv", "cdk.out"]),
            timeout=Duration.seconds(300),
            memory_size=1024,
            environment={
                "STATE_BUCKET": storage.state_bucket.bucket_name,
                "SANDBOX_FUNCTION": self.sandbox_fn.function_name,
                "SQS_QUEUE_URL": storage.event_queue.queue_url,
            },
        )

        # Grant permissions
        storage.state_bucket.grant_read_write(self.agent_fn)
        storage.state_bucket.grant_read(self.sandbox_fn)
        self.sandbox_fn.grant_invoke(self.agent_fn)

        # Trigger agent loop from SQS
        self.agent_fn.add_event_source(
            event_sources.SqsEventSource(
                storage.event_queue,
                batch_size=1,
            )
        )
