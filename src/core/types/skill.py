from pydantic import BaseModel, Field


class SkillParameter(BaseModel):
    name: str
    type: str
    description: str = ""
    required: bool = True


class SkillDefinition(BaseModel):
    name: str
    description: str
    stub_code: str = ""
    parameters: list[SkillParameter] = Field(default_factory=list)
    enabled: bool = True
