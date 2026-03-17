from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from src.web_api.routes import auth, chat, events, status

app = FastAPI(title="Oochy", version="0.1.0", description="Cloud-native AI Agent API")

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # TODO: restrict in production
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

app.include_router(auth.router, prefix="/auth", tags=["auth"])
app.include_router(chat.router, prefix="/chat", tags=["chat"])
app.include_router(events.router, prefix="/events", tags=["events"])
app.include_router(status.router, prefix="/status", tags=["status"])


@app.get("/health")
async def health() -> dict[str, str]:
    return {"status": "ok"}
