class Desktop:
    """Desktop control skill. Executes commands on the connected macOS machine via MQTT."""

    @staticmethod
    async def bash(command: str) -> dict[str, str | int]:
        """Execute a bash command on the desktop. Returns {"stdout": str, "stderr": str, "exit_code": int}."""
        ...

    @staticmethod
    async def apple_script(script: str) -> dict[str, str]:
        """Execute an AppleScript on the desktop. Returns {"result": str}."""
        ...

    @staticmethod
    def is_connected() -> bool:
        """Check if the desktop client is currently connected."""
        ...
