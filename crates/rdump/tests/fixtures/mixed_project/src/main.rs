#[macro_use]
mod macros;
mod lib;
mod traits;

// TODO: Refactor this later
use crate::lib::{User, Role};

struct Cli {
    pattern: String,
}

impl Cli {
    fn new() -> Self { Self { pattern: "".into() } }
}

pub fn main() {
    // This is the main function
    let _u = User::new();
    println!("Hello, world!");
    my_macro!();
}
