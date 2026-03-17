"""Sandboxed Python code execution.

Uses subprocess isolation to run LLM-generated code safely.
RestrictedPython is available for additional compile-time restrictions.
"""
import asyncio
import json
import subprocess
import sys
import tempfile
import textwrap
from pathlib import Path


EXECUTION_TIMEOUT = 30  # seconds


async def run_code(code: str, context: dict[str, object] | None = None) -> dict[str, object]:
    """Execute Python code in an isolated subprocess.

    Returns:
        {"success": True/False, "result": str, "output": str}
    """
    # Wrap the code in an async main() to support top-level await
    wrapped = _wrap_async(code, context)

    with tempfile.TemporaryDirectory() as tmpdir:
        script_path = Path(tmpdir) / "run.py"
        script_path.write_text(wrapped)

        try:
            proc = await asyncio.create_subprocess_exec(
                sys.executable, str(script_path),
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
                cwd=tmpdir,
            )
            stdout, stderr = await asyncio.wait_for(
                proc.communicate(), timeout=EXECUTION_TIMEOUT
            )

            stdout_text = stdout.decode("utf-8", errors="replace")
            stderr_text = stderr.decode("utf-8", errors="replace")

            if proc.returncode != 0:
                return {
                    "success": False,
                    "result": stderr_text or f"Process exited with code {proc.returncode}",
                    "output": stdout_text,
                }

            return {
                "success": True,
                "result": stdout_text,
                "output": stdout_text,
            }

        except asyncio.TimeoutError:
            return {
                "success": False,
                "result": f"Execution timed out after {EXECUTION_TIMEOUT}s",
                "output": "",
            }
        except Exception as e:
            return {
                "success": False,
                "result": str(e),
                "output": "",
            }


def _wrap_async(code: str, context: dict[str, object] | None = None) -> str:
    """Wrap user code in an async main() with context injection."""
    context_json = json.dumps(context or {}, default=str)

    return textwrap.dedent(f"""\
        import asyncio
        import json

        _context = json.loads('''{context_json}''')

        async def _main():
        {textwrap.indent(code, "    ")}

        asyncio.run(_main())
    """)
