use super::{profiles, CodeAwareSettings, SqlDialect};
use crate::evaluator::FileContext;
use anyhow::Result;
use once_cell::sync::Lazy;
use regex::Regex;

pub(super) fn select_language_profile(
    settings: &CodeAwareSettings,
    extension: &str,
    context: &mut FileContext,
) -> Result<Option<(String, &'static profiles::LanguageProfile)>> {
    if let Some(override_key) = settings.language_override.as_deref() {
        let override_key = override_key.to_ascii_lowercase();
        if let Some(profile) = profiles::get_profile(&override_key) {
            emit_language_debug(
                settings,
                context,
                format!("Selected semantic profile `{override_key}` via explicit override."),
            );
            return Ok(Some((override_key, profile)));
        }
        if let Some(profile) = profiles::find_canonical_language_profile(&override_key) {
            emit_language_debug(
                settings,
                context,
                format!(
                    "Selected semantic profile `{}` via canonical override `{override_key}`.",
                    profile.id
                ),
            );
            return Ok(Some((profile.id.to_string(), profile.profile)));
        }
    }

    if extension.eq_ignore_ascii_case("sql") {
        let selected_key = select_sql_profile(settings, context)?;
        if let Some(profile) = profiles::get_profile(&selected_key) {
            emit_language_debug(
                settings,
                context,
                format!("Selected SQL semantic profile `{selected_key}` for `.sql` file."),
            );
            return Ok(Some((selected_key, profile)));
        }
        emit_language_debug(
            settings,
            context,
            "No SQL semantic profile was available after dialect selection.".to_string(),
        );
        return Ok(None);
    }

    let key = extension.to_lowercase();
    if let Some(profile) = profiles::get_profile(&key) {
        emit_language_debug(
            settings,
            context,
            format!("Selected semantic profile `{key}` from extension `.{extension}`."),
        );
        return Ok(Some((key, profile)));
    }

    if key.is_empty() {
        if let Some(key) = detect_shebang_profile(context)? {
            if let Some(profile) = profiles::get_profile(&key) {
                emit_language_debug(
                    settings,
                    context,
                    format!("Selected semantic profile `{key}` from shebang detection."),
                );
                return Ok(Some((key, profile)));
            }
        }
        emit_language_debug(
            settings,
            context,
            "No semantic profile matched this extensionless file.".to_string(),
        );
    } else {
        emit_language_debug(
            settings,
            context,
            format!("No semantic profile matched extension `.{extension}`."),
        );
    }

    Ok(None)
}

pub(super) fn select_sql_profile(
    settings: &CodeAwareSettings,
    context: &mut FileContext,
) -> Result<String> {
    if let Some(cached) = context.sql_profile_key() {
        let cached = cached.to_string();
        emit_sql_trace(
            settings,
            context,
            format!("Reused cached SQL profile `{cached}` for this file."),
        );
        return Ok(cached);
    }

    if let Some(dialect) = &settings.sql_dialect {
        let key = dialect.key().to_string();
        context.set_sql_profile_key(&key);
        emit_sql_trace(
            settings,
            context,
            format!("Selected SQL profile `{key}` from explicit dialect override."),
        );
        return Ok(key);
    }

    let content = context.get_content()?;
    let (detected, trace) = detect_sql_dialect_with_trace(content);
    let key = detected.unwrap_or(SqlDialect::Generic).key().to_string();
    context.set_sql_profile_key(&key);
    emit_sql_trace(settings, context, trace);
    Ok(key)
}

pub(crate) fn detect_sql_dialect(content: &str) -> Option<SqlDialect> {
    detect_sql_dialect_with_trace(content).0
}

pub(crate) fn detect_sql_dialect_with_trace(content: &str) -> (Option<SqlDialect>, String) {
    if MYSQL_DELIMITER_RE.is_match(content) {
        return (
            Some(SqlDialect::Mysql),
            "Detected `sqlmysql` because the file contains a `DELIMITER //` directive.".to_string(),
        );
    }
    if SQLITE_BEGIN_ATOMIC_RE.is_match(content) {
        return (
            Some(SqlDialect::Sqlite),
            "Detected `sqlsqlite` because the file contains `BEGIN ATOMIC`.".to_string(),
        );
    }
    if POSTGRES_RETURNS_TABLE_RE.is_match(content) || content.contains("language plpgsql") {
        return (
            Some(SqlDialect::Postgres),
            "Detected `sqlpostgres` because the file contains PostgreSQL-specific `RETURNS TABLE` or `LANGUAGE plpgsql` syntax."
                .to_string(),
        );
    }
    (
        None,
        "No dialect-specific SQL heuristic matched; falling back to `sqlgeneric`.".to_string(),
    )
}

fn detect_shebang_profile(context: &mut FileContext) -> Result<Option<String>> {
    let first_line = context.get_content()?.lines().next().unwrap_or("");
    if !first_line.starts_with("#!") {
        return Ok(None);
    }

    let shebang = first_line.to_ascii_lowercase();
    let key = if shebang.contains("bash") || shebang.contains("/sh") {
        Some("sh")
    } else if shebang.contains("python") {
        Some("py")
    } else if shebang.contains("node") || shebang.contains("deno") {
        Some("js")
    } else if shebang.contains("ruby") {
        Some("rb")
    } else if shebang.contains("php") {
        Some("php")
    } else if shebang.contains("lua") {
        Some("lua")
    } else {
        None
    };

    Ok(key.map(str::to_string))
}

fn emit_language_debug(settings: &CodeAwareSettings, context: &mut FileContext, message: String) {
    if settings.language_debug {
        context.push_diagnostic(crate::SearchDiagnostic::language_selection(
            context.path.clone(),
            message,
        ));
    }
}

fn emit_sql_trace(settings: &CodeAwareSettings, context: &mut FileContext, message: String) {
    if settings.sql_trace {
        context.push_diagnostic(crate::SearchDiagnostic::sql_dialect_trace(
            context.path.clone(),
            message,
        ));
    }
}

static MYSQL_DELIMITER_RE: Lazy<Regex> = Lazy::new(|| Regex::new("(?i)delimiter\\s+//").unwrap());
static SQLITE_BEGIN_ATOMIC_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new("(?i)begin\\s+atomic").unwrap());
static POSTGRES_RETURNS_TABLE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new("(?i)returns\\s+table").unwrap());
