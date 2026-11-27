//! Instance settings: the `instance_settings` key/value table with built-in defaults.

use std::collections::HashMap;

use serde_json::{Value, json};
use sqlx::{PgConnection, PgPool};

use crate::dto::{AdminSettingsDto, ContentFilterDto, SignupsMode};
use crate::error::AppError;
use crate::filter::ContentFilter;

/// Shown when the operator set no name.
pub const DEFAULT_NAME: &str = "tuita";
/// Shown when the operator set no description.
pub const DEFAULT_DESCRIPTION: &str =
    "A small social platform with a 1-bit dither look that federates over ActivityPub.";
/// Minimum age when the operator set none.
pub const DEFAULT_MIN_AGE: u8 = 13;
/// House rules when the operator set none.
pub const DEFAULT_RULES: [&str; 5] = [
    "Be kind. No harassment, threats or hate.",
    "No spam, scams or platform manipulation.",
    "No illegal content, and nothing sexual involving minors, ever.",
    "Don\u{2019}t impersonate other people or organisations.",
    "Respect privacy: don\u{2019}t post other people\u{2019}s private information.",
];

/// What the operator configured, with defaults filled in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstanceSettings {
    /// Instance name.
    pub name: String,
    /// Instance description.
    pub description: String,
    /// Contact address or URL.
    pub contact: String,
    /// House rules.
    pub rules: Vec<String>,
    /// Minimum age.
    pub min_age: u8,
    /// Who may sign up.
    pub signups: SignupsMode,
    /// The word filter.
    pub filter: ContentFilterDto,
}

impl InstanceSettings {
    /// The defaults, as when the operator configured nothing.
    #[must_use]
    pub fn defaults() -> Self {
        Self::from_rows(HashMap::new())
    }

    fn from_rows(mut rows: HashMap<String, Value>) -> Self {
        let text =
            |rows: &mut HashMap<String, Value>, key: &str, default: &str| match rows.remove(key) {
                Some(Value::String(text)) if !text.trim().is_empty() => text,
                _ => default.to_owned(),
            };
        let rules = match rows.remove("rules") {
            Some(Value::Array(items)) => items
                .into_iter()
                .filter_map(|item| item.as_str().map(|rule| rule.trim().to_owned()))
                .filter(|rule| !rule.is_empty())
                .collect(),
            _ => DEFAULT_RULES
                .iter()
                .map(|rule| (*rule).to_owned())
                .collect(),
        };
        let min_age = rows
            .remove("min_age")
            .and_then(|value| value.as_u64())
            .and_then(|age| u8::try_from(age).ok())
            .unwrap_or(DEFAULT_MIN_AGE);
        let signups = rows
            .remove("signups")
            .and_then(|value| value.as_str().and_then(SignupsMode::parse))
            .unwrap_or(SignupsMode::Open);
        Self {
            name: text(&mut rows, "name", DEFAULT_NAME),
            description: text(&mut rows, "description", DEFAULT_DESCRIPTION),
            contact: text(&mut rows, "contact", ""),
            rules,
            min_age,
            signups,
            filter: filter_from(rows.remove("content_filter")),
        }
    }
}

/// Reads the settings. Missing or malformed keys fall back to their defaults.
///
/// # Errors
///
/// Database failures.
pub async fn load(db: &PgPool) -> Result<InstanceSettings, AppError> {
    let rows: Vec<(String, Value)> = sqlx::query_as("select key, value from instance_settings")
        .fetch_all(db)
        .await?;
    Ok(InstanceSettings::from_rows(rows.into_iter().collect()))
}

/// Reads the content filter the handlers apply.
///
/// # Errors
///
/// Database failures.
pub async fn content_filter(db: &PgPool) -> Result<ContentFilter, AppError> {
    let value: Option<Value> =
        sqlx::query_scalar("select value from instance_settings where key = 'content_filter'")
            .fetch_optional(db)
            .await?;
    Ok(ContentFilter::new(&filter_from(value)))
}

/// Everything the admin settings page shows.
///
/// # Errors
///
/// Database failures.
pub async fn admin_view(db: &PgPool, domain: &str) -> Result<AdminSettingsDto, AppError> {
    let settings = load(db).await?;
    Ok(AdminSettingsDto {
        name: settings.name,
        description: settings.description,
        contact: settings.contact,
        rules: settings.rules,
        min_age: i32::from(settings.min_age),
        signups: settings.signups,
        filter: settings.filter,
        domain: domain.to_owned(),
    })
}

/// Validated changes to the settings; `None` leaves a setting alone.
#[derive(Debug, Default)]
pub struct Changes {
    /// Instance name.
    pub name: Option<String>,
    /// Description.
    pub description: Option<String>,
    /// Contact email.
    pub contact: Option<String>,
    /// House rules.
    pub rules: Option<Vec<String>>,
    /// Minimum age.
    pub min_age: Option<u8>,
    /// Who may sign up.
    pub signups: Option<SignupsMode>,
    /// The word filter.
    pub filter: Option<ContentFilterDto>,
}

/// Stores the present changes.
///
/// # Errors
///
/// Database failures.
pub async fn apply(conn: &mut PgConnection, changes: &Changes) -> Result<(), AppError> {
    let mut rows: Vec<(&str, Value)> = Vec::new();
    if let Some(name) = &changes.name {
        rows.push(("name", Value::from(name.as_str())));
    }
    if let Some(description) = &changes.description {
        rows.push(("description", Value::from(description.as_str())));
    }
    if let Some(contact) = &changes.contact {
        rows.push(("contact", Value::from(contact.as_str())));
    }
    if let Some(rules) = &changes.rules {
        rows.push(("rules", Value::from(rules.clone())));
    }
    if let Some(age) = changes.min_age {
        rows.push(("min_age", Value::from(age)));
    }
    if let Some(mode) = changes.signups {
        rows.push(("signups", Value::from(mode.as_str())));
    }
    if let Some(filter) = &changes.filter {
        rows.push((
            "content_filter",
            json!({
                "blocked": filter.blocked,
                "allowed": filter.allowed,
                "reserved": filter.reserved,
            }),
        ));
    }
    for (key, value) in rows {
        sqlx::query(
            "insert into instance_settings (key, value) values ($1, $2)
             on conflict (key) do update set value = excluded.value",
        )
        .bind(key)
        .bind(value)
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

/// The stored `content_filter` object with defaults for whatever is missing or malformed.
fn filter_from(value: Option<Value>) -> ContentFilterDto {
    let value = value.unwrap_or(Value::Null);
    let text = |key: &str| {
        value
            .get(key)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned()
    };
    ContentFilterDto {
        blocked: text("blocked"),
        allowed: text("allowed"),
        reserved: text("reserved"),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    #[test]
    fn defaults_fill_every_gap() {
        let settings = InstanceSettings::from_rows(HashMap::new());
        assert_eq!(settings.name, "tuita");
        assert_eq!(settings.rules.len(), 5);
        assert_eq!(settings.min_age, 13);
        assert_eq!(settings.signups, SignupsMode::Open);
        assert_eq!(settings.contact, "");
    }

    #[test]
    fn configured_values_win_and_junk_is_ignored() {
        let rows = HashMap::from([
            ("name".to_owned(), json!("Quiet corner")),
            ("rules".to_owned(), json!(["One.", "  ", 7, "Two."])),
            ("min_age".to_owned(), json!(16)),
            ("signups".to_owned(), json!("closed")),
            ("description".to_owned(), json!("   ")),
        ]);
        let settings = InstanceSettings::from_rows(rows);
        assert_eq!(settings.name, "Quiet corner");
        assert_eq!(settings.rules, ["One.", "Two."]);
        assert_eq!(settings.min_age, 16);
        assert_eq!(settings.signups, SignupsMode::Closed);
        assert_eq!(settings.description, DEFAULT_DESCRIPTION);
    }
}
