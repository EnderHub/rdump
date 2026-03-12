use rdump::{
    parser::{self, AstNode, PredicateKey},
    predicates,
};
use std::fs;
use std::path::Path;

/// Traverse the AST and collect every predicate key used in a query.
fn collect_predicates(ast: &AstNode, acc: &mut Vec<PredicateKey>) {
    match ast {
        AstNode::Predicate(key, _) => acc.push(key.clone()),
        AstNode::LogicalOp(_, left, right) => {
            collect_predicates(left, acc);
            collect_predicates(right, acc);
        }
        AstNode::Not(child) => collect_predicates(child, acc),
    }
}

fn load_doc_queries() -> Vec<String> {
    let path = Path::new("tests/data/doc_queries.txt");
    let content = fs::read_to_string(path)
        .expect("tests/data/doc_queries.txt should be present for coverage");
    content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.to_string())
        .collect()
}

#[test]
fn doc_queries_parse_and_use_known_predicates() {
    let queries = load_doc_queries();
    let registry = predicates::create_predicate_registry();
    let mut failures = Vec::new();

    for query in queries {
        match parser::parse_query(&query) {
            Ok(ast) => {
                let mut keys = Vec::new();
                collect_predicates(&ast, &mut keys);

                for key in keys {
                    if !registry.contains_key(&key) {
                        failures.push(format!(
                            "Unknown predicate '{}' in query '{}'",
                            key.as_ref(),
                            query
                        ));
                    }
                }
            }
            Err(err) => {
                failures.push(format!("Failed to parse '{}': {}", query, err));
            }
        }
    }

    if !failures.is_empty() {
        panic!(
            "The following doc queries are not covered by the parser/registry:\n{}",
            failures.join("\n")
        );
    }
}
