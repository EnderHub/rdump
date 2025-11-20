// Deeply nested structures
mod outer {
    mod middle {
        mod inner {
            pub fn deeply_nested_func() {
                let closure = || {
                    let nested_closure = || {
                        println!("Very deep!");
                    };
                    nested_closure();
                };
                closure();
            }
        }
    }
}
