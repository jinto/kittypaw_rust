"""API Gateway + Web API Lambda stack."""
import aws_cdk as cdk
from aws_cdk import (
    aws_apigatewayv2 as apigw,
    aws_lambda as _lambda,
    Duration,
)
from constructs import Construct

from infra.stacks.storage_stack import StorageStack


class ApiStack(cdk.Stack):
    def __init__(
        self,
        scope: Construct,
        construct_id: str,
        storage: StorageStack,
        **kwargs: object,
    ) -> None:
        super().__init__(scope, construct_id, **kwargs)

        # Web API Lambda
        self.api_fn = _lambda.Function(
            self, "WebApi",
            function_name="oochy-web-api",
            runtime=_lambda.Runtime.PYTHON_3_13,
            handler="src.web_api.handler.handler",
            code=_lambda.Code.from_asset(".", exclude=["web", "apps", "infra", ".venv", "cdk.out"]),
            timeout=Duration.seconds(30),
            memory_size=512,
            environment={
                "STATE_BUCKET": storage.state_bucket.bucket_name,
                "SQS_QUEUE_URL": storage.event_queue.queue_url,
            },
        )

        # Grant permissions
        storage.state_bucket.grant_read_write(self.api_fn)
        storage.event_queue.grant_send_messages(self.api_fn)

        # HTTP API Gateway
        self.http_api = apigw.HttpApi(
            self, "HttpApi",
            api_name="oochy-api",
            cors_preflight=apigw.CorsPreflightOptions(
                allow_origins=["*"],
                allow_methods=[apigw.CorsHttpMethod.ANY],
                allow_headers=["*"],
            ),
        )

        # Lambda integration
        integration = apigw.HttpIntegration(
            self, "LambdaIntegration",
            http_api=self.http_api,
            integration_type=apigw.HttpIntegrationType.AWS_PROXY,
            integration_uri=self.api_fn.function_arn,
        )

        cdk.CfnOutput(self, "ApiUrl", value=self.http_api.url or "")
