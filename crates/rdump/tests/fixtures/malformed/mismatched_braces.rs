// Mismatched braces
fn func_one() {
    if true {
        println!("nested");
    // missing closing brace
}

fn func_two() {
    let x = vec![1, 2, 3;  // wrong bracket
}
