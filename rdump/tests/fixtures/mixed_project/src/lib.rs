// This is a library file.
use serde::Serialize;

pub type UserId = u64;

pub struct User {
    id: UserId,
    name: String,
}

impl User {
    pub fn new() -> Self {
        Self { id: 0, name: "".into() }
    }
}

pub enum Role {
    Admin,
    User,
}
