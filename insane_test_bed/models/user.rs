// REVIEW: model file
// FIXME: improve later
// comment:deprecated marker
pub struct User { pub id: u64, pub name: String }
impl User { pub fn new(name: &str) -> Self { Self { id: 0, name: name.into() } } }
