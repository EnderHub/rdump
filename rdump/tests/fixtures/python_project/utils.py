# Utility functions
import re
from typing import Any

def validate_email(email: str) -> bool:
    """Validate email format."""
    pattern = r'^[\w\.-]+@[\w\.-]+\.\w+$'
    return bool(re.match(pattern, email))

def format_name(first: str, last: str) -> str:
    """Format full name."""
    return f"{first.title()} {last.title()}"

class ConfigLoader:
    """Load configuration from various sources."""

    def __init__(self, path: str):
        self.path = path
        self._config: dict[str, Any] = {}

    def load(self) -> dict[str, Any]:
        # HACK: Simplified for testing
        return {"debug": True, "port": 8080}
