//! Shapes ActivityPub allows for the same field: one value or an array, an id or an embedded
//! object. Remote servers differ, so reading is lenient: an item that does not fit is skipped
//! instead of failing the whole document.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

/// Most items kept from one array; the rest of a hostile list is dropped.
pub(super) const MAX_ITEMS: usize = 256;

/// A field that is a value, an array of values, or absent. Items that do not parse as `T` are
/// dropped. Serializes as the bare value when there is exactly one.
///
/// ```
/// use tuita_core::ap::OneOrMany;
///
/// let one: OneOrMany<String> = serde_json::from_str(r#""a""#).unwrap();
/// let many: OneOrMany<String> = serde_json::from_str(r#"["a", 3, "b"]"#).unwrap();
/// assert_eq!(one.as_slice(), ["a"]);
/// assert_eq!(many.as_slice(), ["a", "b"]);
/// assert_eq!(serde_json::to_string(&one).unwrap(), r#""a""#);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OneOrMany<T>(Vec<T>);

impl<T> Default for OneOrMany<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T> OneOrMany<T> {
    /// Wraps `items`.
    #[must_use]
    pub fn new(items: Vec<T>) -> Self {
        Self(items)
    }

    /// The items.
    #[must_use]
    pub fn as_slice(&self) -> &[T] {
        &self.0
    }

    /// Iterates the items.
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }

    /// The first item.
    #[must_use]
    pub fn first(&self) -> Option<&T> {
        self.0.first()
    }

    /// How many items.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether there are no items.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The items, owned.
    #[must_use]
    pub fn into_vec(self) -> Vec<T> {
        self.0
    }
}

impl<'a, T> IntoIterator for &'a OneOrMany<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<T> From<Vec<T>> for OneOrMany<T> {
    fn from(items: Vec<T>) -> Self {
        Self(items)
    }
}

impl<T> FromIterator<T> for OneOrMany<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<T: Serialize> Serialize for OneOrMany<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self.0.as_slice() {
            [one] => one.serialize(serializer),
            many => many.serialize(serializer),
        }
    }
}

impl<'de, T: DeserializeOwned> Deserialize<'de> for OneOrMany<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self(match Value::deserialize(deserializer)? {
            Value::Null => Vec::new(),
            Value::Array(items) => items
                .into_iter()
                .take(MAX_ITEMS)
                .filter_map(|item| serde_json::from_value(item).ok())
                .collect(),
            one => serde_json::from_value(one).ok().into_iter().collect(),
        }))
    }
}

/// Types that carry their own `id`, so [`IdOrObject::id`] can name them.
pub trait HasId {
    /// The object's `id`, when it has one.
    fn object_id(&self) -> Option<&str>;
}

/// A reference by id or an embedded object: `"https://x/1"` or `{"id": "https://x/1", ...}`.
///
/// An object that does not parse as `T` but has a string `id` becomes the id.
///
/// ```
/// use tuita_core::ap::{IdOrObject, Tombstone};
///
/// let id: IdOrObject<Tombstone> = serde_json::from_str(r#""https://x/1""#).unwrap();
/// let object: IdOrObject<Tombstone> =
///     serde_json::from_str(r#"{"type": "Tombstone", "id": "https://x/2"}"#).unwrap();
/// assert_eq!(id.id(), Some("https://x/1"));
/// assert_eq!(object.id(), Some("https://x/2"));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum IdOrObject<T> {
    /// Only the id.
    Id(String),
    /// The embedded object.
    Object(Box<T>),
}

impl<T: HasId> IdOrObject<T> {
    /// The id of the reference or of the embedded object.
    #[must_use]
    pub fn id(&self) -> Option<&str> {
        match self {
            Self::Id(id) => Some(id),
            Self::Object(object) => object.object_id(),
        }
    }
}

impl<T: Serialize> Serialize for IdOrObject<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Id(id) => id.serialize(serializer),
            Self::Object(object) => object.serialize(serializer),
        }
    }
}

impl<'de, T: DeserializeOwned> Deserialize<'de> for IdOrObject<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match Value::deserialize(deserializer)? {
            Value::String(id) => Ok(Self::Id(id)),
            Value::Object(map) => {
                let id = map.get("id").and_then(Value::as_str).map(str::to_owned);
                match serde_json::from_value::<T>(Value::Object(map)) {
                    Ok(object) => Ok(Self::Object(Box::new(object))),
                    Err(err) => id
                        .map(Self::Id)
                        .ok_or_else(|| serde::de::Error::custom(err)),
                }
            }
            _ => Err(serde::de::Error::custom("expected an id or an object")),
        }
    }
}

/// The id a reference names: a string, an object's `id`, or the first such in an array.
#[must_use]
pub fn id_of(value: &Value) -> Option<&str> {
    match value {
        Value::String(id) => Some(id),
        Value::Object(map) => map.get("id").and_then(Value::as_str),
        Value::Array(items) => items.iter().find_map(id_of),
        _ => None,
    }
}

/// `Option<String>` from an id, an embedded object with an `id`, or an array of them.
pub(super) fn lenient_id<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    Ok(id_of(&Value::deserialize(deserializer)?).map(str::to_owned))
}

/// `String` from [`lenient_id`], empty when absent or unreadable.
pub(super) fn lenient_id_or_empty<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<String, D::Error> {
    Ok(lenient_id(deserializer)?.unwrap_or_default())
}

/// `Option<String>` from a string, or a number or bool written as text. Other shapes are `None`.
pub(super) fn lenient_text<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    Ok(match Value::deserialize(deserializer)? {
        Value::String(text) => Some(text),
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    })
}

/// The first string of `value`: a string, or an array's first string.
pub(super) fn first_string(value: &Value) -> Option<&str> {
    match value {
        Value::String(text) => Some(text),
        Value::Array(items) => items.iter().find_map(first_string),
        _ => None,
    }
}

/// `Option<String>` from `value`, taking the first string of an array.
pub(super) fn lenient_first_string<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    Ok(first_string(&Value::deserialize(deserializer)?).map(str::to_owned))
}

/// A `url` field: a string, a `Link` object (`href`), or an array of either. A Link with a
/// `text/html` media type wins over the others.
pub(super) fn lenient_url<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    fn href(value: &Value) -> Option<(&str, bool)> {
        match value {
            Value::String(url) => Some((url, false)),
            Value::Object(map) => {
                let url = map.get("href").and_then(Value::as_str)?;
                let html = map
                    .get("mediaType")
                    .and_then(Value::as_str)
                    .is_some_and(|kind| kind.eq_ignore_ascii_case("text/html"));
                Some((url, html))
            }
            _ => None,
        }
    }
    let value = Value::deserialize(deserializer)?;
    Ok(match &value {
        Value::Array(items) => {
            let found: Vec<(&str, bool)> = items.iter().take(MAX_ITEMS).filter_map(href).collect();
            found
                .iter()
                .find(|(_, html)| *html)
                .or_else(|| found.first())
                .map(|(url, _)| (*url).to_owned())
        }
        other => href(other).map(|(url, _)| url.to_owned()),
    })
}

/// A boolean that may arrive as the text `"true"`.
pub(super) fn lenient_bool<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
    Ok(match Value::deserialize(deserializer)? {
        Value::Bool(flag) => flag,
        Value::String(text) => text.eq_ignore_ascii_case("true"),
        _ => false,
    })
}

/// `Option<bool>` that may arrive as text.
pub(super) fn lenient_opt_bool<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<bool>, D::Error> {
    Ok(match Value::deserialize(deserializer)? {
        Value::Bool(flag) => Some(flag),
        Value::String(text) => Some(text.eq_ignore_ascii_case("true")),
        _ => None,
    })
}

/// `Option<u64>` from a number or numeric text.
pub(super) fn lenient_u64<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<u64>, D::Error> {
    Ok(match Value::deserialize(deserializer)? {
        Value::Number(number) => number.as_u64(),
        Value::String(text) => text.trim().parse().ok(),
        _ => None,
    })
}

/// The type names of an object: `"Note"` or `["Note", "foaf:Document"]`.
#[must_use]
pub fn type_names(value: &Value) -> Vec<&str> {
    match value.get("type") {
        Some(Value::String(name)) => vec![name.as_str()],
        Some(Value::Array(names)) => names.iter().filter_map(Value::as_str).collect(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    #[test]
    fn one_or_many_reads_every_shape() {
        let read =
            |value: Value| -> OneOrMany<String> { serde_json::from_value(value).expect("parse") };
        assert!(read(json!(null)).is_empty());
        assert_eq!(read(json!("a")).as_slice(), ["a"]);
        assert_eq!(read(json!(["a", {"x": 1}, "b"])).as_slice(), ["a", "b"]);
        assert!(read(json!({"id": "a"})).is_empty());
    }

    #[test]
    fn one_or_many_caps_hostile_arrays() {
        let items: Vec<Value> = (0..10_000).map(|n| json!(n.to_string())).collect();
        let got: OneOrMany<String> = serde_json::from_value(Value::Array(items)).expect("parse");
        assert_eq!(got.len(), MAX_ITEMS);
    }

    #[test]
    fn one_or_many_writes_one_as_a_bare_value() {
        assert_eq!(
            serde_json::to_value(OneOrMany::new(vec!["a".to_owned()])).expect("json"),
            json!("a")
        );
        assert_eq!(
            serde_json::to_value(OneOrMany::new(vec!["a".to_owned(), "b".to_owned()]))
                .expect("json"),
            json!(["a", "b"])
        );
    }

    #[test]
    fn ids_come_from_strings_objects_and_arrays() {
        assert_eq!(id_of(&json!("a")), Some("a"));
        assert_eq!(id_of(&json!({"id": "b"})), Some("b"));
        assert_eq!(id_of(&json!([1, {"id": "c"}])), Some("c"));
        assert_eq!(id_of(&json!(5)), None);
    }
}
