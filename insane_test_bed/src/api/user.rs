// TODO: user api
// HACK: demo file for rdump tests
use crate::models::user::User;
use crate::models::role::Role;

pub type UserId = u64;

pub enum Status { Active, Inactive }

pub struct Cli { pub pattern: String }

pub struct Order { pub id: u32 }

pub trait Name { fn name(&self) -> String; }

pub trait Summary { fn summarize(&self) -> String; }

pub struct Database { pub url: String }

impl Summary for Order { fn summarize(&self) -> String { format!("{}", self.id) } }
impl Name for User { fn name(&self) -> String { self.name.clone() } }

pub fn process_data() { println!("value * 2"); }
pub fn process_payment() { println!("processing payment"); }
pub fn user_service() { println!("user_service"); }

pub fn main() {
    println!("MAIN FUNCTION");
    process_data();
    process_payment();
    user_service();
}
