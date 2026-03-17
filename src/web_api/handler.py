"""AWS Lambda handler for the Web API.

Uses Mangum to adapt FastAPI to Lambda + API Gateway.
"""
from mangum import Mangum

from src.web_api.app import app

handler = Mangum(app, lifespan="off")
