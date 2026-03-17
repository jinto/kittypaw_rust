"""AWS IoT Core MQTT stack for desktop client communication."""
import aws_cdk as cdk
from aws_cdk import aws_iot as iot
from constructs import Construct


class MqttStack(cdk.Stack):
    def __init__(self, scope: Construct, construct_id: str, **kwargs: object) -> None:
        super().__init__(scope, construct_id, **kwargs)

        # IoT Policy for desktop clients
        self.desktop_policy = iot.CfnPolicy(
            self, "DesktopPolicy",
            policy_name="oochy-desktop-policy",
            policy_document={
                "Version": "2012-10-17",
                "Statement": [
                    {
                        "Effect": "Allow",
                        "Action": ["iot:Connect"],
                        "Resource": [f"arn:aws:iot:{cdk.Aws.REGION}:{cdk.Aws.ACCOUNT_ID}:client/oochy-desktop-*"],
                    },
                    {
                        "Effect": "Allow",
                        "Action": ["iot:Subscribe"],
                        "Resource": [f"arn:aws:iot:{cdk.Aws.REGION}:{cdk.Aws.ACCOUNT_ID}:topicfilter/oochy/desktop/*"],
                    },
                    {
                        "Effect": "Allow",
                        "Action": ["iot:Publish", "iot:Receive"],
                        "Resource": [f"arn:aws:iot:{cdk.Aws.REGION}:{cdk.Aws.ACCOUNT_ID}:topic/oochy/*"],
                    },
                ],
            },
        )

        # IoT Thing for desktop client
        self.desktop_thing = iot.CfnThing(
            self, "DesktopThing",
            thing_name="oochy-desktop-default",
        )
