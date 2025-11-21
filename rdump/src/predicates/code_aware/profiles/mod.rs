use crate::parser::PredicateKey;
use once_cell::sync::Lazy;
use std::collections::HashMap;

use super::SqlDialect;

mod bash;
mod c;
mod cpp;
mod csharp;
mod css;
mod elixir;
mod go;
mod haskell;
mod html;
mod java;
mod javascript;
mod lua;
mod ocaml;
mod php;
mod python;
mod react; // Add react module
mod ruby;
mod rust;
mod scala;
mod sql;
mod swift;
mod typescript;
mod zig;

/// Defines the tree-sitter queries and metadata for a specific language.
pub struct LanguageProfile {
    pub name: &'static str,
    pub extensions: Vec<&'static str>,
    pub(super) language: tree_sitter::Language,
    pub queries: HashMap<PredicateKey, String>,
}

pub(super) static LANGUAGE_PROFILES: Lazy<HashMap<&'static str, LanguageProfile>> =
    Lazy::new(|| {
        let mut m = HashMap::new();
        m.insert("c", c::create_c_profile());
        m.insert("cpp", cpp::create_cpp_profile());
        m.insert("cc", cpp::create_cpp_profile());
        m.insert("cxx", cpp::create_cpp_profile());
        m.insert("hpp", cpp::create_cpp_profile());
        m.insert("hh", cpp::create_cpp_profile());
        m.insert("hxx", cpp::create_cpp_profile());
        m.insert("cs", csharp::create_csharp_profile());
        m.insert("csx", csharp::create_csharp_profile());
        m.insert("php", php::create_php_profile());
        m.insert("rb", ruby::create_ruby_profile());
        m.insert("sh", bash::create_bash_profile());
        m.insert("bash", bash::create_bash_profile());
        m.insert("css", css::create_css_profile());
        m.insert("ex", elixir::create_elixir_profile());
        m.insert("exs", elixir::create_elixir_profile());
        m.insert("html", html::create_html_profile());
        m.insert("lua", lua::create_lua_profile());
        m.insert("ml", ocaml::create_ocaml_profile());
        m.insert("mli", ocaml::create_ocaml_profile());
        m.insert("zig", zig::create_zig_profile());
        m.insert("hs", haskell::create_haskell_profile());
        m.insert("lhs", haskell::create_haskell_profile());
        m.insert("scala", scala::create_scala_profile());
        m.insert("swift", swift::create_swift_profile());
        m.insert("rs", rust::create_rust_profile());
        m.insert("py", python::create_python_profile());
        m.insert("go", go::create_go_profile());
        m.insert("java", java::create_java_profile());
        m.insert("ts", typescript::create_typescript_profile());
        m.insert("js", javascript::create_javascript_profile());
        m.insert("jsx", react::create_react_profile());
        m.insert("tsx", react::create_react_profile());
        m.insert(SqlDialect::Generic.key(), sql::create_generic_profile());
        m.insert(SqlDialect::Postgres.key(), sql::create_postgres_profile());
        m.insert(SqlDialect::Mysql.key(), sql::create_mysql_profile());
        m.insert(SqlDialect::Sqlite.key(), sql::create_sqlite_profile());
        m
    });

/// Returns a list of all configured language profiles.
pub fn list_language_profiles() -> Vec<&'static LanguageProfile> {
    LANGUAGE_PROFILES.values().collect()
}

pub(super) fn get_profile(key: &str) -> Option<&'static LanguageProfile> {
    LANGUAGE_PROFILES.get(key)
}
