"""mypy type checker for LLM-generated code.

Writes code to a temp file, runs mypy --strict, and returns any errors.
"""
import subprocess
import tempfile
from pathlib import Path

SKILL_STUBS_DIR = Path(__file__).parent.parent / "core" / "skills"


def type_check(code: str) -> list[str]:
    """Run mypy on the given code. Returns a list of error strings, empty if clean."""
    with tempfile.TemporaryDirectory() as tmpdir:
        code_path = Path(tmpdir) / "generated.py"

        # Write the generated code with skill imports prepended
        full_code = _prepend_skill_stubs(code)
        code_path.write_text(full_code)

        # Copy .pyi stubs into temp dir so mypy can find them
        for stub in SKILL_STUBS_DIR.glob("*.pyi"):
            (Path(tmpdir) / stub.name).write_text(stub.read_text())

        # Run mypy
        result = subprocess.run(
            ["mypy", "--strict", "--no-error-summary", str(code_path)],
            capture_output=True,
            text=True,
            timeout=30,
            cwd=tmpdir,
        )

        if result.returncode == 0:
            return []

        # Parse errors, filtering noise
        errors: list[str] = []
        for line in result.stdout.strip().split("\n"):
            if line and "error:" in line:
                # Remove the temp file path prefix for cleaner errors
                clean = line.split("generated.py:")[-1] if "generated.py:" in line else line
                errors.append(clean.strip())

        return errors


def _prepend_skill_stubs(code: str) -> str:
    """Prepend skill class stubs so mypy can resolve types."""
    stubs: list[str] = []
    for stub_file in sorted(SKILL_STUBS_DIR.glob("*.pyi")):
        stubs.append(stub_file.read_text())

    # Wrap in TYPE_CHECKING to avoid runtime import issues
    header = "from __future__ import annotations\nfrom typing import TYPE_CHECKING\n\n"
    if stubs:
        header += "if TYPE_CHECKING:\n    pass\n\n"
        header += "\n\n".join(stubs)
        header += "\n\n"

    return header + code
