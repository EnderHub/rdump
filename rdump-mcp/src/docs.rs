use crate::types::{FieldDoc, FunctionDoc, RqlReference, SdkReference, TypeDoc};

pub fn build_rql_reference() -> RqlReference {
    RqlReference {
        operators: vec!["AND: &", "OR: |", "NOT: !", "Grouping: ( )"]
            .into_iter()
            .map(String::from)
            .collect(),
        notes: vec![
            "Quote values with spaces using single or double quotes.",
            "Examples: contains:'fn main' or name:'test file.rs'.",
            "Implicit AND is not supported; use '&'.",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
        metadata_predicates: vec!["ext", "name", "path", "in", "size", "modified"]
            .into_iter()
            .map(String::from)
            .collect(),
        content_predicates: vec!["contains", "matches"]
            .into_iter()
            .map(String::from)
            .collect(),
        semantic_predicates: vec![
            "def", "func", "import", "call", "class", "struct", "enum", "interface", "trait",
            "type", "impl", "macro", "module", "object", "protocol", "comment", "str",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
        react_predicates: vec!["component", "element", "hook", "customhook", "prop"]
            .into_iter()
            .map(String::from)
            .collect(),
        examples: vec![
            "ext:rs & func:main",
            "path:src & (struct:User | enum:UserState)",
            "import:serde & contains:derive",
            "ext:tsx & component:Button & hook:useState",
            "modified:<2d & size:>10kb",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
    }
}

pub fn build_sdk_reference() -> SdkReference {
    let functions = vec![
        FunctionDoc {
            name: "search_iter".to_string(),
            signature:
                "search_iter(query: &str, options: SearchOptions) -> Result<SearchResultIterator>"
                    .to_string(),
            description: "Streaming iterator over SearchResult. Preferred for large repos."
                .to_string(),
        },
        FunctionDoc {
            name: "search".to_string(),
            signature: "search(query: &str, options: SearchOptions) -> Result<Vec<SearchResult>>"
                .to_string(),
            description: "Collects all results into memory.".to_string(),
        },
    ];

    let search_options_fields = vec![
        FieldDoc {
            name: "root".to_string(),
            description: "Root directory to search.".to_string(),
            default: "\".\"".to_string(),
        },
        FieldDoc {
            name: "presets".to_string(),
            description: "Named presets to apply (e.g., rust, python).".to_string(),
            default: "[]".to_string(),
        },
        FieldDoc {
            name: "no_ignore".to_string(),
            description: "If true, ignore .gitignore/.rdumpignore.".to_string(),
            default: "false".to_string(),
        },
        FieldDoc {
            name: "hidden".to_string(),
            description: "If true, include hidden files and directories.".to_string(),
            default: "false".to_string(),
        },
        FieldDoc {
            name: "max_depth".to_string(),
            description: "Maximum directory depth.".to_string(),
            default: "None".to_string(),
        },
        FieldDoc {
            name: "sql_dialect".to_string(),
            description: "Override SQL dialect for .sql files.".to_string(),
            default: "None".to_string(),
        },
    ];

    let result_fields = vec![
        FieldDoc {
            name: "path".to_string(),
            description: "Path to matched file.".to_string(),
            default: "".to_string(),
        },
        FieldDoc {
            name: "matches".to_string(),
            description: "Match hunks (empty for whole-file matches).".to_string(),
            default: "[]".to_string(),
        },
        FieldDoc {
            name: "content".to_string(),
            description: "Full file content.".to_string(),
            default: "".to_string(),
        },
    ];

    let match_fields = vec![
        FieldDoc {
            name: "start_line".to_string(),
            description: "1-indexed start line.".to_string(),
            default: "".to_string(),
        },
        FieldDoc {
            name: "end_line".to_string(),
            description: "1-indexed end line.".to_string(),
            default: "".to_string(),
        },
        FieldDoc {
            name: "start_column".to_string(),
            description: "0-indexed start column.".to_string(),
            default: "".to_string(),
        },
        FieldDoc {
            name: "end_column".to_string(),
            description: "0-indexed end column.".to_string(),
            default: "".to_string(),
        },
        FieldDoc {
            name: "byte_range".to_string(),
            description: "Byte range within file content.".to_string(),
            default: "".to_string(),
        },
        FieldDoc {
            name: "text".to_string(),
            description: "Matched text (may be shortened).".to_string(),
            default: "".to_string(),
        },
    ];

    let types = vec![
        TypeDoc {
            name: "SearchOptions".to_string(),
            description: "Search configuration for SDK calls.".to_string(),
            fields: search_options_fields,
        },
        TypeDoc {
            name: "SearchResult".to_string(),
            description: "Result for a matched file.".to_string(),
            fields: result_fields,
        },
        TypeDoc {
            name: "Match".to_string(),
            description: "Single match span within a file.".to_string(),
            fields: match_fields,
        },
    ];

    let notes = vec![
        "Use search_iter for large repos to avoid loading all results at once.",
        "RQL supports logical operators &, |, ! and parentheses.",
    ]
    .into_iter()
    .map(String::from)
    .collect();

    SdkReference {
        functions,
        types,
        notes,
    }
}

pub fn format_rql_reference_text() -> String {
    let reference = build_rql_reference();
    let mut lines = Vec::new();

    lines.push("RQL Operators:".to_string());
    for op in &reference.operators {
        lines.push(format!("- {op}"));
    }

    lines.push("".to_string());
    lines.push("Notes:".to_string());
    for note in &reference.notes {
        lines.push(format!("- {note}"));
    }

    lines.push("".to_string());
    lines.push("Metadata predicates:".to_string());
    lines.push(reference.metadata_predicates.join(", "));

    lines.push("".to_string());
    lines.push("Content predicates:".to_string());
    lines.push(reference.content_predicates.join(", "));

    lines.push("".to_string());
    lines.push("Semantic predicates:".to_string());
    lines.push(reference.semantic_predicates.join(", "));

    lines.push("".to_string());
    lines.push("React predicates:".to_string());
    lines.push(reference.react_predicates.join(", "));

    lines.push("".to_string());
    lines.push("Examples:".to_string());
    for example in &reference.examples {
        lines.push(format!("- {example}"));
    }

    lines.join("\n")
}

pub fn format_sdk_reference_text() -> String {
    let reference = build_sdk_reference();
    let mut lines = Vec::new();

    lines.push("SDK Functions:".to_string());
    for func in &reference.functions {
        lines.push(format!("- {}: {}", func.name, func.signature));
        lines.push(format!("  {}", func.description));
    }

    lines.push("".to_string());
    lines.push("SDK Types:".to_string());
    for ty in &reference.types {
        lines.push(format!("- {}: {}", ty.name, ty.description));
        for field in &ty.fields {
            if field.default.is_empty() {
                lines.push(format!("  - {}: {}", field.name, field.description));
            } else {
                lines.push(format!(
                    "  - {}: {} (default: {})",
                    field.name, field.description, field.default
                ));
            }
        }
    }

    lines.push("".to_string());
    lines.push("Notes:".to_string());
    for note in &reference.notes {
        lines.push(format!("- {note}"));
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rql_reference_has_examples() {
        let reference = build_rql_reference();
        assert!(!reference.examples.is_empty());
    }

    #[test]
    fn sdk_reference_has_types() {
        let reference = build_sdk_reference();
        assert!(!reference.types.is_empty());
    }

    #[test]
    fn format_rql_reference_text_contains_examples() {
        let text = format_rql_reference_text();
        assert!(text.contains("Examples:"));
    }

    #[test]
    fn sdk_reference_includes_search_options() {
        let reference = build_sdk_reference();
        let has_search_options = reference
            .types
            .iter()
            .any(|ty| ty.name == "SearchOptions");
        assert!(has_search_options);
    }
}
