//! Activities: the envelope around what an actor did (`Create`, `Follow`, `Like`, ...).

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};

use super::value::{OneOrMany, id_of, lenient_id, lenient_id_or_empty, lenient_text, type_names};

/// What an activity does. Types tuita does not handle are [`ActivityKind::Other`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActivityKind {
    /// Ask to follow.
    Follow,
    /// Accept a follow request.
    Accept,
    /// Refuse a follow request.
    Reject,
    /// Take back an earlier activity.
    Undo,
    /// Publish an object.
    Create,
    /// Change an object.
    Update,
    /// Remove an object or an actor.
    Delete,
    /// Like a post.
    Like,
    /// Repost a post.
    Announce,
    /// Block an actor.
    Block,
    /// Report to the moderators.
    Flag,
    /// An account moved; received and ignored.
    Move,
    /// Anything else.
    Other,
}

impl ActivityKind {
    /// The `type` string on the wire; `Other` is `"Other"`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Follow => "Follow",
            Self::Accept => "Accept",
            Self::Reject => "Reject",
            Self::Undo => "Undo",
            Self::Create => "Create",
            Self::Update => "Update",
            Self::Delete => "Delete",
            Self::Like => "Like",
            Self::Announce => "Announce",
            Self::Block => "Block",
            Self::Flag => "Flag",
            Self::Move => "Move",
            Self::Other => "Other",
        }
    }

    fn from_text(text: &str) -> Option<Self> {
        Some(match text {
            "Follow" => Self::Follow,
            "Accept" => Self::Accept,
            "Reject" => Self::Reject,
            "Undo" => Self::Undo,
            "Create" => Self::Create,
            "Update" => Self::Update,
            "Delete" => Self::Delete,
            "Like" => Self::Like,
            "Announce" => Self::Announce,
            "Block" => Self::Block,
            "Flag" => Self::Flag,
            "Move" => Self::Move,
            _ => return None,
        })
    }
}

impl Serialize for ActivityKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ActivityKind {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::json!({ "type": Value::deserialize(deserializer)? });
        Ok(type_names(&value)
            .into_iter()
            .find_map(Self::from_text)
            .unwrap_or(Self::Other))
    }
}

/// An activity as received or sent. `object` stays raw JSON: what it holds depends on the
/// `kind`, and the handler reads it with the accessors below.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    /// The activity's URL; some servers omit it on `Like` or `Follow`.
    #[serde(
        default,
        deserialize_with = "lenient_id",
        skip_serializing_if = "Option::is_none"
    )]
    pub id: Option<String>,
    /// What it does.
    #[serde(rename = "type")]
    pub kind: ActivityKind,
    /// The actor that did it, by id.
    #[serde(default, deserialize_with = "lenient_id_or_empty")]
    pub actor: String,
    /// What it was done to: an id, or an embedded object or activity.
    #[serde(default)]
    pub object: Value,
    /// The target, for activities that have one.
    #[serde(
        default,
        deserialize_with = "lenient_id",
        skip_serializing_if = "Option::is_none"
    )]
    pub target: Option<String>,
    /// Who it is addressed to.
    #[serde(default, skip_serializing_if = "OneOrMany::is_empty")]
    pub to: OneOrMany<String>,
    /// Who else sees it.
    #[serde(default, skip_serializing_if = "OneOrMany::is_empty")]
    pub cc: OneOrMany<String>,
    /// When it happened, RFC 3339.
    #[serde(
        default,
        deserialize_with = "lenient_text",
        skip_serializing_if = "Option::is_none"
    )]
    pub published: Option<String>,
    /// The reason of a `Flag`.
    #[serde(
        default,
        deserialize_with = "lenient_text",
        skip_serializing_if = "Option::is_none"
    )]
    pub content: Option<String>,
}

impl Activity {
    /// A new activity of `kind` by `actor` on `object`, addressed to nobody yet.
    #[must_use]
    pub fn new(kind: ActivityKind, id: Option<String>, actor: &str, object: Value) -> Self {
        Self {
            id,
            kind,
            actor: actor.to_owned(),
            object,
            target: None,
            to: OneOrMany::default(),
            cc: OneOrMany::default(),
            published: None,
            content: None,
        }
    }

    /// Sets the audience.
    #[must_use]
    pub fn addressed(mut self, to: Vec<String>, cc: Vec<String>) -> Self {
        self.to = OneOrMany::new(to);
        self.cc = OneOrMany::new(cc);
        self
    }

    /// The id the object names: the object itself when it is a string, else its `id`.
    #[must_use]
    pub fn object_id(&self) -> Option<&str> {
        id_of(&self.object)
    }

    /// The embedded object, when there is one.
    #[must_use]
    pub fn object_map(&self) -> Option<&Map<String, Value>> {
        match &self.object {
            Value::Object(map) => Some(map),
            Value::Array(items) => items.iter().find_map(Value::as_object),
            _ => None,
        }
    }

    /// The embedded object's `type`.
    #[must_use]
    pub fn object_type(&self) -> Option<&str> {
        match &self.object {
            Value::Object(_) => type_names(&self.object).into_iter().next(),
            _ => None,
        }
    }

    /// Every id in `object`, for activities that name several (`Flag`).
    #[must_use]
    pub fn object_ids(&self) -> Vec<&str> {
        match &self.object {
            Value::Array(items) => items.iter().filter_map(id_of).collect(),
            other => id_of(other).into_iter().collect(),
        }
    }

    /// The JSON to send: the activity with the ActivityStreams `@context`.
    #[must_use]
    pub fn to_document(&self) -> Value {
        super::document(self)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    #[test]
    fn unknown_types_become_other_and_arrays_of_types_work() {
        let parse = |value: Value| -> Activity { serde_json::from_value(value).expect("parse") };
        assert_eq!(
            parse(json!({"type": "EmojiReact", "actor": "a"})).kind,
            ActivityKind::Other
        );
        assert_eq!(
            parse(json!({"type": ["Like", "foo:Bar"], "actor": {"id": "a"}, "object": "o"})).kind,
            ActivityKind::Like
        );
    }

    #[test]
    fn an_embedded_object_gives_its_id_and_type() {
        let activity: Activity = serde_json::from_value(
            json!({"type": "Create", "actor": "a", "object": {"id": "n", "type": "Note"}}),
        )
        .expect("parse");
        assert_eq!(activity.object_id(), Some("n"));
        assert_eq!(activity.object_type(), Some("Note"));
        assert!(activity.object_map().is_some());
    }

    #[test]
    fn a_flag_names_several_objects() {
        let activity: Activity = serde_json::from_value(
            json!({"type": "Flag", "actor": "a", "object": ["u", {"id": "p"}, 5], "content": "why"}),
        )
        .expect("parse");
        assert_eq!(activity.object_ids(), ["u", "p"]);
        assert_eq!(activity.content.as_deref(), Some("why"));
    }
}
