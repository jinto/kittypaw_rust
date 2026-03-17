"""Google OAuth 2.0 authentication routes."""
import os

import jwt
from fastapi import APIRouter, HTTPException
from pydantic import BaseModel

router = APIRouter()

GOOGLE_CLIENT_ID = os.environ.get("GOOGLE_CLIENT_ID", "")
JWT_SECRET = os.environ.get("JWT_SECRET", "dev-secret-change-me")


class TokenRequest(BaseModel):
    google_token: str


class TokenResponse(BaseModel):
    access_token: str
    token_type: str = "bearer"


@router.post("/login", response_model=TokenResponse)
async def login(request: TokenRequest) -> TokenResponse:
    """Exchange a Google OAuth token for a JWT."""
    # TODO: Verify google_token with Google's API
    # For now, generate a JWT directly (dev mode)
    user_info = _verify_google_token(request.google_token)

    token = jwt.encode(
        {"sub": user_info["email"], "name": user_info.get("name", "")},
        JWT_SECRET,
        algorithm="HS256",
    )

    return TokenResponse(access_token=token)


def _verify_google_token(token: str) -> dict[str, str]:
    """Verify Google OAuth token and return user info.

    TODO: Implement proper verification with Google's tokeninfo endpoint.
    """
    # Placeholder — in production, call Google's API
    return {"email": "dev@oochy.local", "name": "Developer"}
