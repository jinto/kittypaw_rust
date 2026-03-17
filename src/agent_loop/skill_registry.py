import os
from pathlib import Path

from src.core.types.skill import SkillDefinition

SKILLS_DIR = Path(__file__).parent.parent / "core" / "skills"


def load_skill_stubs(skills_dir: Path | None = None) -> list[SkillDefinition]:
    """Load all .pyi stub files from the skills directory."""
    directory = skills_dir or SKILLS_DIR
    skills: list[SkillDefinition] = []

    for pyi_file in sorted(directory.glob("*.pyi")):
        stub_code = pyi_file.read_text()
        name = pyi_file.stem
        # Extract docstring from class as description
        description = _extract_class_docstring(stub_code) or f"{name} skill"
        skills.append(
            SkillDefinition(
                name=name,
                description=description,
                stub_code=stub_code,
            )
        )

    return skills


def _extract_class_docstring(code: str) -> str | None:
    """Extract the first class docstring from a .pyi file."""
    lines = code.split("\n")
    in_class = False
    for line in lines:
        if line.startswith("class "):
            in_class = True
            continue
        if in_class and '"""' in line:
            return line.strip().strip('"""')
    return None


def build_skill_context(skills: list[SkillDefinition] | None = None) -> str:
    """Build the skill context string to inject into the LLM prompt."""
    if skills is None:
        skills = load_skill_stubs()

    parts = [
        "# Available Skills",
        "# You can use these classes directly in your code.",
        "# All async methods must be awaited.",
        "",
    ]

    for skill in skills:
        if not skill.enabled:
            continue
        parts.append(f"# --- {skill.name} ---")
        parts.append(skill.stub_code)
        parts.append("")

    return "\n".join(parts)
