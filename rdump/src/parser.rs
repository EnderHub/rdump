use anyhow::{anyhow, Result};
use pest::iterators::{Pair, Pairs};
use pest::pratt_parser::{Assoc, Op, PrattParser};
pub use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "rql.pest"]
pub struct RqlParser;

lazy_static::lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use Assoc::*;
        use Rule::*;

        PrattParser::new()
            .op(Op::infix(OR, Left))
            .op(Op::infix(AND, Left))
    };
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum PredicateKey {
    Ext,
    Name,
    Path,
    PathExact,
    Contains,
    Matches,
    Size,
    Modified,
    In,
    // --- SEMANTIC PREDICATES ---
    // Generic
    Def,
    Func,
    Import,
    // Granular Definitions
    Class,
    Struct,
    Enum,
    Interface,
    Trait,
    Type,
    Impl,
    Macro,
    // Syntactic Content
    Comment,
    Str,
    // Usage
    Call,
    // --- React-specific Predicates ---
    Component,
    Element,
    Hook,
    CustomHook,
    Prop,
    // A key for testing or unknown predicates
    Other(String),
}

impl AsRef<str> for PredicateKey {
    fn as_ref(&self) -> &str {
        match self {
            PredicateKey::Ext => "ext",
            PredicateKey::Name => "name",
            PredicateKey::Path => "path",
            PredicateKey::PathExact => "path_exact",
            PredicateKey::Contains => "contains",
            PredicateKey::Matches => "matches",
            PredicateKey::Size => "size",
            PredicateKey::Modified => "modified",
            PredicateKey::In => "in",
            PredicateKey::Def => "def",
            PredicateKey::Func => "func",
            PredicateKey::Import => "import",
            PredicateKey::Class => "class",
            PredicateKey::Struct => "struct",
            PredicateKey::Enum => "enum",
            PredicateKey::Interface => "interface",
            PredicateKey::Trait => "trait",
            PredicateKey::Type => "type",
            PredicateKey::Impl => "impl",
            PredicateKey::Macro => "macro",
            PredicateKey::Comment => "comment",
            PredicateKey::Str => "str",
            PredicateKey::Call => "call",
            PredicateKey::Component => "component",
            PredicateKey::Element => "element",
            PredicateKey::Hook => "hook",
            PredicateKey::CustomHook => "customhook",
            PredicateKey::Prop => "prop",
            PredicateKey::Other(s) => s.as_str(),
        }
    }
}

impl From<&str> for PredicateKey {
    fn from(s: &str) -> Self {
        match s {
            "ext" => Self::Ext,
            "name" => Self::Name,
            "path" => Self::Path,
            "path_exact" => Self::PathExact,
            "contains" => Self::Contains,
            "c" => Self::Contains,
            "matches" => Self::Matches,
            "m" => Self::Matches,
            "size" => Self::Size,
            "modified" => Self::Modified,
            "in" => Self::In,
            // --- SEMANTIC ---
            "def" => Self::Def,
            "func" => Self::Func,
            "import" => Self::Import,
            "class" => Self::Class,
            "struct" => Self::Struct,
            "enum" => Self::Enum,
            "interface" => Self::Interface,
            "trait" => Self::Trait,
            "type" => Self::Type,
            "impl" => Self::Impl,
            "macro" => Self::Macro,
            "comment" => Self::Comment,
            "str" => Self::Str,
            "call" => Self::Call,
            // --- REACT ---
            "component" => Self::Component,
            "element" => Self::Element,
            "hook" => Self::Hook,
            "customhook" => Self::CustomHook,
            "prop" => Self::Prop,
            // Any other key is captured here.
            other => Self::Other(other.to_string()),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum AstNode {
    Predicate(PredicateKey, String),
    LogicalOp(LogicalOperator, Box<AstNode>, Box<AstNode>),
    Not(Box<AstNode>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum LogicalOperator {
    And,
    Or,
}

pub fn parse_query(query: &str) -> Result<AstNode> {
    if query.trim().is_empty() {
        return Err(anyhow!("Query cannot be empty."));
    }

    match RqlParser::parse(Rule::query, query) {
        Ok(mut pairs) => {
            // Unpack query -> expression to get the token stream for the parser.
            let expression = pairs.next().unwrap().into_inner().next().unwrap();
            build_ast_from_expression_pairs(expression.into_inner())
        }
        Err(e) => Err(anyhow!("Invalid query syntax:\n{e}")),
    }
}

// This function is the heart of the parser, using the Pratt method. It consumes
// the token stream for a single expression level.
fn build_ast_from_expression_pairs(pairs: Pairs<Rule>) -> Result<AstNode> {
    if pairs
        .clone()
        .last()
        .is_some_and(|p| matches!(p.as_rule(), Rule::AND | Rule::OR))
    {
        return Err(anyhow!(
            "Invalid query syntax: query cannot end with an operator."
        ));
    }

    // Check for implicit AND operators, which are not supported.
    // This prevents a panic in the Pratt parser when two terms are adjacent.
    let mut last_was_term = false;
    for pair in pairs.clone() {
        let current_is_term = matches!(pair.as_rule(), Rule::term);
        if current_is_term && last_was_term {
            return Err(anyhow!("Invalid query syntax: missing logical operator (like '&' or '|') between predicates. Implicit operators are not supported."));
        }
        last_was_term = current_is_term;
    }

    PRATT_PARSER
        .map_primary(|primary| build_ast_from_term(primary))
        .map_infix(|lhs, op, rhs| {
            let op = match op.as_rule() {
                Rule::AND => LogicalOperator::And,
                Rule::OR => LogicalOperator::Or,
                _ => unreachable!(),
            };
            Ok(AstNode::LogicalOp(op, Box::new(lhs?), Box::new(rhs?)))
        })
        .parse(pairs)
}

// This function handles the "primary" parts of the grammar: predicates,
// parenthesized expressions, and negation.
fn build_ast_from_term(pair: Pair<Rule>) -> Result<AstNode> {
    match pair.as_rule() {
        Rule::predicate => {
            let mut inner = pair.into_inner();
            if let Some(inner_predicate) = inner.next() {
                let rule = inner_predicate.as_rule(); // Get the rule before moving
                let mut predicate_parts = inner_predicate.into_inner();
                let key_pair = predicate_parts
                    .next()
                    .ok_or_else(|| anyhow!("Missing key in predicate for rule {rule:?}"))?;
                let value_pair = predicate_parts.next().ok_or_else(|| {
                    anyhow!("Missing value in predicate for key '{}'", key_pair.as_str())
                })?;
                let key = PredicateKey::from(key_pair.as_str());
                let value = unescape_value(value_pair.as_str());
                Ok(AstNode::Predicate(key, value))
            } else {
                Err(anyhow!("Invalid predicate: empty inner rule"))
            }
        }
        Rule::expression => {
            // A parenthesized expression. Recurse by parsing its inner pairs.
            build_ast_from_expression_pairs(pair.into_inner())
        }
        Rule::term => {
            let mut inner = pair.into_inner();
            let first = inner.next().unwrap();
            if first.as_rule() == Rule::NOT {
                let factor = inner.next().unwrap();
                let ast = build_ast_from_term(factor)?;
                Ok(AstNode::Not(Box::new(ast)))
            } else {
                build_ast_from_term(first)
            }
        }
        Rule::factor => build_ast_from_term(pair.into_inner().next().unwrap()),
        _ => Err(anyhow!("Unknown primary rule: {:?}", pair.as_rule())),
    }
}

fn unescape_value(value: &str) -> String {
    let quote_char = value.chars().next();
    if quote_char == Some('"') || quote_char == Some('\'') {
        let inner = &value[1..value.len() - 1];
        let mut unescaped = String::with_capacity(inner.len());
        let mut chars = inner.chars();
        while let Some(c) = chars.next() {
            if c == '\\' {
                if let Some(next_c) = chars.next() {
                    unescaped.push(next_c);
                }
            } else {
                unescaped.push(c);
            }
        }
        return unescaped;
    }
    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create a predicate node for cleaner tests.
    fn predicate(key: PredicateKey, value: &str) -> Box<AstNode> {
        Box::new(AstNode::Predicate(key, value.to_string()))
    }

    #[test]
    fn test_parse_simple_predicate() {
        let ast = parse_query("ext:rs").unwrap();
        assert_eq!(ast, *predicate(PredicateKey::Ext, "rs"));
    }

    #[test]
    fn test_parse_predicate_with_quoted_value() {
        let ast = parse_query("name:\"foo bar\"").unwrap();
        assert_eq!(ast, *predicate(PredicateKey::Name, "foo bar"));
    }

    #[test]
    fn test_parse_logical_and() {
        let ast = parse_query("ext:rs & name:\"foo\"").unwrap();
        assert_eq!(
            ast,
            AstNode::LogicalOp(
                LogicalOperator::And,
                predicate(PredicateKey::Ext, "rs"),
                predicate(PredicateKey::Name, "foo")
            )
        );
    }

    #[test]
    fn test_parse_logical_or() {
        let ast = parse_query("ext:rs | ext:toml").unwrap();
        assert_eq!(
            ast,
            AstNode::LogicalOp(
                LogicalOperator::Or,
                predicate(PredicateKey::Ext, "rs"),
                predicate(PredicateKey::Ext, "toml")
            )
        );
    }

    #[test]
    fn test_parse_negation() {
        let ast = parse_query("!ext:rs").unwrap();
        assert_eq!(ast, AstNode::Not(predicate(PredicateKey::Ext, "rs")));
    }

    #[test]
    fn test_parse_complex_query() {
        let ast = parse_query("ext:rs & (name:\"foo\" | name:\"bar\") & !path:tests").unwrap();
        let inner_or = AstNode::LogicalOp(
            LogicalOperator::Or,
            predicate(PredicateKey::Name, "foo"),
            predicate(PredicateKey::Name, "bar"),
        );
        let and_with_or = AstNode::LogicalOp(
            LogicalOperator::And,
            predicate(PredicateKey::Ext, "rs"),
            Box::new(inner_or),
        );
        let final_ast = AstNode::LogicalOp(
            LogicalOperator::And,
            Box::new(and_with_or),
            Box::new(AstNode::Not(predicate(PredicateKey::Path, "tests"))),
        );
        assert_eq!(ast, final_ast);
    }

    #[test]
    fn test_unescape_value() {
        assert_eq!(unescape_value(r#""hello \"world\"""#), "hello \"world\"");
        assert_eq!(unescape_value(r#"'hello \'world\'""#), "hello 'world'");
        assert_eq!(unescape_value(r#""a \\ b""#), "a \\ b");
        assert_eq!(unescape_value("no_quotes"), "no_quotes");
    }

    #[test]
    fn test_parse_predicate_with_special_chars_in_value() {
        let ast = parse_query(r#"name:"foo&bar""#).unwrap();
        assert_eq!(ast, *predicate(PredicateKey::Name, "foo&bar"));
    }

    #[test]
    fn test_parse_semantic_predicates() {
        let ast_def = parse_query("def:User").unwrap();
        assert_eq!(ast_def, *predicate(PredicateKey::Def, "User"));

        let ast_func = parse_query("func:get_user").unwrap();
        assert_eq!(ast_func, *predicate(PredicateKey::Func, "get_user"));

        let ast_import = parse_query("import:serde").unwrap();
        assert_eq!(ast_import, *predicate(PredicateKey::Import, "serde"));
    }

    #[test]
    fn test_parse_granular_and_syntactic_predicates() {
        assert_eq!(
            parse_query("class:Foo").unwrap(),
            *predicate(PredicateKey::Class, "Foo")
        );
        assert_eq!(
            parse_query("struct:Bar").unwrap(),
            *predicate(PredicateKey::Struct, "Bar")
        );
        assert_eq!(
            parse_query("comment:TODO").unwrap(),
            *predicate(PredicateKey::Comment, "TODO")
        );
        assert_eq!(
            parse_query("str:'api_key'").unwrap(),
            *predicate(PredicateKey::Str, "api_key")
        );
        assert_eq!(
            parse_query("call:my_func").unwrap(),
            *predicate(PredicateKey::Call, "my_func")
        );
    }

    #[test]
    fn test_parse_react_and_new_rust_predicates() {
        assert_eq!(
            parse_query("component:App").unwrap(),
            *predicate(PredicateKey::Component, "App")
        );
        assert_eq!(
            parse_query("hook:useState").unwrap(),
            *predicate(PredicateKey::Hook, "useState")
        );
        assert_eq!(
            parse_query("macro:my_macro").unwrap(),
            *predicate(PredicateKey::Macro, "my_macro")
        );
        assert_eq!(
            parse_query("impl:User").unwrap(),
            *predicate(PredicateKey::Impl, "User")
        );
    }

    #[test]
    fn test_error_on_trailing_operator() {
        let result = parse_query("ext:rs &");
        let err = result.unwrap_err();
        assert!(err
            .to_string()
            .contains("query cannot end with an operator"));
    }

    #[test]
    fn test_error_on_missing_value() {
        let result = parse_query("ext:");
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid query syntax:"));
    }

    #[test]
    fn test_error_on_unclosed_parenthesis() {
        let result = parse_query("(ext:rs | path:src");
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid query syntax:"));
    }

    #[test]
    fn test_error_on_empty_query() {
        let result = parse_query("");
        assert_eq!(result.unwrap_err().to_string(), "Query cannot be empty.");
    }

    #[test]
    fn test_error_on_whitespace_query() {
        let result = parse_query("   ");
        assert_eq!(result.unwrap_err().to_string(), "Query cannot be empty.");
    }

    #[test]
    fn test_parse_keyword_operators() {
        // AND
        let ast_and = parse_query("ext:rs and name:\"foo\"").unwrap();
        assert_eq!(
            ast_and,
            AstNode::LogicalOp(
                LogicalOperator::And,
                predicate(PredicateKey::Ext, "rs"),
                predicate(PredicateKey::Name, "foo")
            )
        );

        // OR
        let ast_or = parse_query("ext:rs or ext:toml").unwrap();
        assert_eq!(
            ast_or,
            AstNode::LogicalOp(
                LogicalOperator::Or,
                predicate(PredicateKey::Ext, "rs"),
                predicate(PredicateKey::Ext, "toml")
            )
        );

        // NOT
        let ast_not = parse_query("not ext:rs").unwrap();
        assert_eq!(ast_not, AstNode::Not(predicate(PredicateKey::Ext, "rs")));
    }

    #[test]
    fn test_parse_mixed_operators() {
        let ast = parse_query("ext:rs and (name:foo or name:bar) & not path:tests").unwrap();
        let inner_or = AstNode::LogicalOp(
            LogicalOperator::Or,
            predicate(PredicateKey::Name, "foo"),
            predicate(PredicateKey::Name, "bar"),
        );
        let and_with_or = AstNode::LogicalOp(
            LogicalOperator::And,
            predicate(PredicateKey::Ext, "rs"),
            Box::new(inner_or),
        );
        let final_ast = AstNode::LogicalOp(
            LogicalOperator::And,
            Box::new(and_with_or),
            Box::new(AstNode::Not(predicate(PredicateKey::Path, "tests"))),
        );
        assert_eq!(ast, final_ast);
    }

    #[test]
    fn test_parse_unknown_predicate() {
        let ast = parse_query("unknown:predicate").unwrap();
        assert_eq!(
            ast,
            *predicate(PredicateKey::Other("unknown".to_string()), "predicate")
        );
    }

    #[test]
    fn test_parse_all_metadata_predicates() {
        // Test all metadata predicates to exercise PredicateKey::from and as_ref
        assert_eq!(
            parse_query("path_exact:/foo/bar").unwrap(),
            *predicate(PredicateKey::PathExact, "/foo/bar")
        );
        assert_eq!(
            parse_query("contains:TODO").unwrap(),
            *predicate(PredicateKey::Contains, "TODO")
        );
        assert_eq!(
            parse_query("matches:^test_").unwrap(),
            *predicate(PredicateKey::Matches, "^test_")
        );
        assert_eq!(
            parse_query("size:>1000").unwrap(),
            *predicate(PredicateKey::Size, ">1000")
        );
        assert_eq!(
            parse_query("modified:<1d").unwrap(),
            *predicate(PredicateKey::Modified, "<1d")
        );
        assert_eq!(
            parse_query("in:src/").unwrap(),
            *predicate(PredicateKey::In, "src/")
        );
    }

    #[test]
    fn test_parse_all_semantic_predicates() {
        // Test all semantic predicates
        assert_eq!(
            parse_query("enum:Status").unwrap(),
            *predicate(PredicateKey::Enum, "Status")
        );
        assert_eq!(
            parse_query("interface:IUser").unwrap(),
            *predicate(PredicateKey::Interface, "IUser")
        );
        assert_eq!(
            parse_query("trait:Display").unwrap(),
            *predicate(PredicateKey::Trait, "Display")
        );
        assert_eq!(
            parse_query("type:UserId").unwrap(),
            *predicate(PredicateKey::Type, "UserId")
        );
    }

    #[test]
    fn test_parse_all_react_predicates() {
        // Test all React-specific predicates
        assert_eq!(
            parse_query("element:div").unwrap(),
            *predicate(PredicateKey::Element, "div")
        );
        assert_eq!(
            parse_query("customhook:useAuth").unwrap(),
            *predicate(PredicateKey::CustomHook, "useAuth")
        );
        assert_eq!(
            parse_query("prop:onClick").unwrap(),
            *predicate(PredicateKey::Prop, "onClick")
        );
    }

    #[test]
    fn test_predicate_key_as_ref() {
        // Directly test as_ref() for all PredicateKey variants
        assert_eq!(PredicateKey::Ext.as_ref(), "ext");
        assert_eq!(PredicateKey::Name.as_ref(), "name");
        assert_eq!(PredicateKey::Path.as_ref(), "path");
        assert_eq!(PredicateKey::PathExact.as_ref(), "path_exact");
        assert_eq!(PredicateKey::Contains.as_ref(), "contains");
        assert_eq!(PredicateKey::Matches.as_ref(), "matches");
        assert_eq!(PredicateKey::Size.as_ref(), "size");
        assert_eq!(PredicateKey::Modified.as_ref(), "modified");
        assert_eq!(PredicateKey::In.as_ref(), "in");
        assert_eq!(PredicateKey::Def.as_ref(), "def");
        assert_eq!(PredicateKey::Func.as_ref(), "func");
        assert_eq!(PredicateKey::Import.as_ref(), "import");
        assert_eq!(PredicateKey::Class.as_ref(), "class");
        assert_eq!(PredicateKey::Struct.as_ref(), "struct");
        assert_eq!(PredicateKey::Enum.as_ref(), "enum");
        assert_eq!(PredicateKey::Interface.as_ref(), "interface");
        assert_eq!(PredicateKey::Trait.as_ref(), "trait");
        assert_eq!(PredicateKey::Type.as_ref(), "type");
        assert_eq!(PredicateKey::Impl.as_ref(), "impl");
        assert_eq!(PredicateKey::Macro.as_ref(), "macro");
        assert_eq!(PredicateKey::Comment.as_ref(), "comment");
        assert_eq!(PredicateKey::Str.as_ref(), "str");
        assert_eq!(PredicateKey::Call.as_ref(), "call");
        assert_eq!(PredicateKey::Component.as_ref(), "component");
        assert_eq!(PredicateKey::Element.as_ref(), "element");
        assert_eq!(PredicateKey::Hook.as_ref(), "hook");
        assert_eq!(PredicateKey::CustomHook.as_ref(), "customhook");
        assert_eq!(PredicateKey::Prop.as_ref(), "prop");
        assert_eq!(PredicateKey::Other("custom".to_string()).as_ref(), "custom");
    }

    #[test]
    fn test_parse_deeply_nested_expression() {
        // Test deeply nested parentheses
        let ast = parse_query("((ext:rs))").unwrap();
        assert_eq!(ast, *predicate(PredicateKey::Ext, "rs"));
    }

    #[test]
    fn test_parse_multiple_negations() {
        // Test double negation with parentheses
        let ast = parse_query("!(!ext:rs)").unwrap();
        assert_eq!(
            ast,
            AstNode::Not(Box::new(AstNode::Not(predicate(PredicateKey::Ext, "rs"))))
        );
    }

    #[test]
    fn test_parse_complex_nested_with_negation() {
        // Complex query with nested groups and negations
        let ast = parse_query("!(ext:rs | ext:py) & func:main").unwrap();
        let inner_or = AstNode::LogicalOp(
            LogicalOperator::Or,
            predicate(PredicateKey::Ext, "rs"),
            predicate(PredicateKey::Ext, "py"),
        );
        let expected = AstNode::LogicalOp(
            LogicalOperator::And,
            Box::new(AstNode::Not(Box::new(inner_or))),
            predicate(PredicateKey::Func, "main"),
        );
        assert_eq!(ast, expected);
    }

    #[test]
    fn test_error_on_consecutive_predicates() {
        // Missing operator between predicates
        let result = parse_query("ext:rs name:foo");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_on_missing_closing_paren() {
        let result = parse_query("(ext:rs | name:foo");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_on_empty_parentheses() {
        let result = parse_query("()");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_value_with_colon() {
        // Value containing a colon
        let ast = parse_query("contains:\"key:value\"").unwrap();
        assert_eq!(ast, *predicate(PredicateKey::Contains, "key:value"));
    }

    #[test]
    fn test_parse_value_with_operators() {
        // Value containing operator characters
        let ast = parse_query("contains:\"a & b | c\"").unwrap();
        assert_eq!(ast, *predicate(PredicateKey::Contains, "a & b | c"));
    }
}
