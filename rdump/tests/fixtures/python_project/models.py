# Data models for the application
from dataclasses import dataclass
from typing import List, Optional

@dataclass
class User:
    id: int
    name: str
    email: Optional[str] = None

class UserService:
    def __init__(self):
        self.users: List[User] = []

    def add_user(self, user: User) -> None:
        self.users.append(user)

    async def fetch_user(self, user_id: int) -> Optional[User]:
        # TODO: Implement actual database fetch
        for user in self.users:
            if user.id == user_id:
                return user
        return None

def create_admin():
    return User(id=0, name="admin", email="admin@example.com")
