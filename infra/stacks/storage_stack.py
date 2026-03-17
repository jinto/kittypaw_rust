"""S3 + SQS FIFO stack for agent state and event queuing."""
import aws_cdk as cdk
from aws_cdk import (
    aws_s3 as s3,
    aws_sqs as sqs,
    Duration,
)
from constructs import Construct


class StorageStack(cdk.Stack):
    def __init__(self, scope: Construct, construct_id: str, **kwargs: object) -> None:
        super().__init__(scope, construct_id, **kwargs)

        # Agent state bucket
        self.state_bucket = s3.Bucket(
            self, "StateBucket",
            bucket_name=f"oochy-state-{cdk.Aws.ACCOUNT_ID}",
            removal_policy=cdk.RemovalPolicy.RETAIN,
            versioned=True,
            encryption=s3.BucketEncryption.S3_MANAGED,
            lifecycle_rules=[
                s3.LifecycleRule(
                    noncurrent_version_expiration=Duration.days(30),
                ),
            ],
        )

        # Dead letter queue
        dlq = sqs.Queue(
            self, "DeadLetterQueue",
            queue_name="oochy-events-dlq.fifo",
            fifo=True,
            retention_period=Duration.days(14),
        )

        # Event queue (FIFO for ordering)
        self.event_queue = sqs.Queue(
            self, "EventQueue",
            queue_name="oochy-events.fifo",
            fifo=True,
            content_based_deduplication=False,
            visibility_timeout=Duration.seconds(300),
            dead_letter_queue=sqs.DeadLetterQueue(
                max_receive_count=3,
                queue=dlq,
            ),
        )

        # Outputs
        cdk.CfnOutput(self, "StateBucketName", value=self.state_bucket.bucket_name)
        cdk.CfnOutput(self, "EventQueueUrl", value=self.event_queue.queue_url)
