use rdump::parser::parse_query;

fn next_u64(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state
}

fn build_query(seed: &mut u64) -> String {
    const TOKENS: &[&str] = &[
        "ext:rs",
        "func:main",
        "contains:todo",
        "matches:[a-z]+",
        "path:src",
        "name:test_*.rs",
        "!",
        "&",
        "|",
        "(",
        ")",
        "\"unterminated",
        "::::",
        " ",
    ];

    let parts = (0..12)
        .map(|_| {
            let index = (next_u64(seed) as usize) % TOKENS.len();
            TOKENS[index]
        })
        .collect::<Vec<_>>();

    parts.join(" ")
}

#[test]
fn parser_fuzzish_queries_do_not_panic() {
    let mut seed = 0x5eed_u64;

    for _ in 0..512 {
        let query = build_query(&mut seed);
        let result = std::panic::catch_unwind(|| parse_query(&query));
        assert!(result.is_ok(), "parser panicked on query: {query}");
    }
}
