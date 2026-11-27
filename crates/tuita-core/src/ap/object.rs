//! ActivityStreams objects: actors, notes, questions, attachments, tags and collections.
//!
//! Every type reads leniently (remote servers disagree on shapes) and writes the shape Mastodon
//! expects. Only `id` and `type` are required when reading.

use serde::de::Error as _;
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Value, json};

use super::origin::is_http_url;
use super::value::{
    HasId, OneOrMany, lenient_bool, lenient_first_string, lenient_id, lenient_id_or_empty,
    lenient_opt_bool, lenient_text, lenient_u64, lenient_url, type_names,
};
use crate::emoji::is_valid_shortcode;

fn kind_of<E: serde::de::Error>(value: &Value, known: &[&str]) -> Result<String, E> {
    type_names(value)
        .into_iter()
        .find(|name| known.contains(name))
        .map(str::to_owned)
        .ok_or_else(|| E::custom("unsupported object type"))
}

macro_rules! kind_enum {
    (
        $(#[$meta:meta])*
        $name:ident { $($(#[$vmeta:meta])* $variant:ident => $text:literal),+ $(,)? }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum $name {
            $($(#[$vmeta])* $variant,)+
        }

        impl $name {
            /// The `type` string on the wire.
            #[must_use]
            pub const fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $text,)+ }
            }

            fn from_text(text: &str) -> Option<Self> {
                match text { $($text => Some(Self::$variant),)+ _ => None }
            }
        }

        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(self.as_str())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let value = Value::deserialize(deserializer)?;
                let names = match &value {
                    Value::String(name) => vec![name.as_str()],
                    Value::Array(items) => items.iter().filter_map(Value::as_str).collect(),
                    _ => Vec::new(),
                };
                names
                    .into_iter()
                    .find_map(Self::from_text)
                    .ok_or_else(|| D::Error::custom("unsupported type"))
            }
        }
    };
}

kind_enum! {
    /// The `type` of an actor.
    ActorKind {
        /// A person.
        Person => "Person",
        /// A bot or other automated account.
        Service => "Service",
        /// An organization.
        Organization => "Organization",
        /// An application, such as the instance actor.
        Application => "Application",
        /// A group; read as a person.
        Group => "Group",
    }
}

kind_enum! {
    /// The `type` of a post-like object.
    NoteKind {
        /// A short post.
        Note => "Note",
        /// A poll.
        Question => "Question",
        /// A long post; read as a note.
        Article => "Article",
        /// A page; read as a note.
        Page => "Page",
    }
}

/// An `icon` or `image`: a picture by URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    /// Where the picture is.
    pub url: String,
    /// Its media type, when the server said.
    pub media_type: Option<String>,
}

impl Serialize for Image {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Image")?;
        if let Some(kind) = &self.media_type {
            map.serialize_entry("mediaType", kind)?;
        }
        map.serialize_entry("url", &self.url)?;
        map.end()
    }
}

impl<'de> Deserialize<'de> for Image {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        fn read(value: &Value) -> Option<Image> {
            match value {
                Value::String(url) => Some(Image {
                    url: url.clone(),
                    media_type: None,
                }),
                Value::Object(map) => {
                    let url = match map.get("url") {
                        Some(Value::String(url)) => url.clone(),
                        Some(Value::Object(link)) => link.get("href")?.as_str()?.to_owned(),
                        Some(Value::Array(items)) => items.iter().find_map(|item| match item {
                            Value::String(url) => Some(url.clone()),
                            Value::Object(link) => link.get("href")?.as_str().map(str::to_owned),
                            _ => None,
                        })?,
                        _ => return None,
                    };
                    Some(Image {
                        url,
                        media_type: map
                            .get("mediaType")
                            .and_then(Value::as_str)
                            .map(str::to_owned),
                    })
                }
                Value::Array(items) => items.iter().find_map(read),
                _ => None,
            }
        }
        read(&Value::deserialize(deserializer)?).ok_or_else(|| D::Error::custom("not an image"))
    }
}

/// A profile field: `{"type": "PropertyValue", "name": "Website", "value": "<html>"}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyValue {
    /// The label.
    pub name: String,
    /// The value, HTML.
    pub value: String,
}

impl Serialize for PropertyValue {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        json!({"type": "PropertyValue", "name": self.name, "value": self.value})
            .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PropertyValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;
        kind_of::<D::Error>(&value, &["PropertyValue"])?;
        let text = |key: &str| value.get(key).and_then(Value::as_str).map(str::to_owned);
        Ok(Self {
            name: text("name").ok_or_else(|| D::Error::custom("no name"))?,
            value: text("value").unwrap_or_default(),
        })
    }
}

/// `endpoints`: where shared deliveries go.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Endpoints {
    /// The shared inbox.
    #[serde(
        default,
        deserialize_with = "lenient_text",
        skip_serializing_if = "Option::is_none"
    )]
    pub shared_inbox: Option<String>,
}

/// An actor's `publicKey`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicKeyInfo {
    /// The key id, usually `{actor}#main-key`; it is what `keyId` in a signature names.
    #[serde(default, deserialize_with = "lenient_id_or_empty")]
    pub id: String,
    /// The actor that owns the key.
    #[serde(default, deserialize_with = "lenient_id_or_empty")]
    pub owner: String,
    /// The key, PEM.
    #[serde(default)]
    pub public_key_pem: String,
}

fn lenient_first_key<'de, D: Deserializer<'de>>(d: D) -> Result<Option<PublicKeyInfo>, D::Error> {
    Ok(OneOrMany::<PublicKeyInfo>::deserialize(d)?
        .into_vec()
        .into_iter()
        .next())
}

fn lenient_first_image<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Image>, D::Error> {
    Ok(OneOrMany::<Image>::deserialize(d)?
        .into_vec()
        .into_iter()
        .next())
}

fn lenient_endpoints<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Endpoints>, D::Error> {
    Ok(Endpoints::deserialize(d).ok())
}

/// A Person, Service, Organization, Application or Group.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Actor {
    /// The actor's URL, its identity.
    pub id: String,
    /// What kind of actor.
    #[serde(rename = "type")]
    pub kind: ActorKind,
    /// The handle, without `@` or domain.
    #[serde(default, deserialize_with = "lenient_text_or_empty")]
    pub preferred_username: String,
    /// Display name.
    #[serde(
        default,
        deserialize_with = "lenient_text",
        skip_serializing_if = "Option::is_none"
    )]
    pub name: Option<String>,
    /// Biography, HTML.
    #[serde(
        default,
        deserialize_with = "lenient_text",
        skip_serializing_if = "Option::is_none"
    )]
    pub summary: Option<String>,
    /// The profile page.
    #[serde(
        default,
        deserialize_with = "lenient_url",
        skip_serializing_if = "Option::is_none"
    )]
    pub url: Option<String>,
    /// The avatar.
    #[serde(
        default,
        deserialize_with = "lenient_first_image",
        skip_serializing_if = "Option::is_none"
    )]
    pub icon: Option<Image>,
    /// The banner.
    #[serde(
        default,
        deserialize_with = "lenient_first_image",
        skip_serializing_if = "Option::is_none"
    )]
    pub image: Option<Image>,
    /// The personal inbox.
    #[serde(default, deserialize_with = "lenient_text_or_empty")]
    pub inbox: String,
    /// The outbox collection.
    #[serde(
        default,
        deserialize_with = "lenient_text",
        skip_serializing_if = "Option::is_none"
    )]
    pub outbox: Option<String>,
    /// The followers collection.
    #[serde(
        default,
        deserialize_with = "lenient_text",
        skip_serializing_if = "Option::is_none"
    )]
    pub followers: Option<String>,
    /// The following collection.
    #[serde(
        default,
        deserialize_with = "lenient_text",
        skip_serializing_if = "Option::is_none"
    )]
    pub following: Option<String>,
    /// The collection of pinned posts.
    #[serde(
        default,
        deserialize_with = "lenient_text",
        skip_serializing_if = "Option::is_none"
    )]
    pub featured: Option<String>,
    /// Shared delivery endpoints.
    #[serde(
        default,
        deserialize_with = "lenient_endpoints",
        skip_serializing_if = "Option::is_none"
    )]
    pub endpoints: Option<Endpoints>,
    /// The signing key.
    #[serde(
        default,
        deserialize_with = "lenient_first_key",
        skip_serializing_if = "Option::is_none"
    )]
    pub public_key: Option<PublicKeyInfo>,
    /// Follows need the account's approval.
    #[serde(default, deserialize_with = "lenient_bool")]
    pub manually_approves_followers: bool,
    /// Listed in directories and search.
    #[serde(
        default,
        deserialize_with = "lenient_opt_bool",
        skip_serializing_if = "Option::is_none"
    )]
    pub discoverable: Option<bool>,
    /// When the account was created, RFC 3339.
    #[serde(
        default,
        deserialize_with = "lenient_text",
        skip_serializing_if = "Option::is_none"
    )]
    pub published: Option<String>,
    /// Profile fields; other attachment types are dropped.
    #[serde(default, skip_serializing_if = "OneOrMany::is_empty")]
    pub attachment: OneOrMany<PropertyValue>,
    /// Where the account moved to, when it did.
    #[serde(
        default,
        deserialize_with = "lenient_id",
        skip_serializing_if = "Option::is_none"
    )]
    pub moved_to: Option<String>,
    /// Custom emotes of the display name and biography; other tags are read as [`Tag::Other`].
    #[serde(default, skip_serializing_if = "OneOrMany::is_empty")]
    pub tag: OneOrMany<Tag>,
}

fn lenient_text_or_empty<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    Ok(lenient_text(d)?.unwrap_or_default())
}

impl HasId for Actor {
    fn object_id(&self) -> Option<&str> {
        Some(&self.id)
    }
}

impl Actor {
    /// The inbox deliveries should go to: the shared inbox when there is one, else the personal.
    #[must_use]
    pub fn delivery_inbox(&self) -> &str {
        self.endpoints
            .as_ref()
            .and_then(|endpoints| endpoints.shared_inbox.as_deref())
            .filter(|inbox| !inbox.is_empty())
            .unwrap_or(&self.inbox)
    }
}

/// An attached file: image, video or audio.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    /// `Document`, `Image`, `Video` or `Audio`.
    #[serde(rename = "type", deserialize_with = "document_kind")]
    pub kind: String,
    /// The media type, such as `image/png`.
    #[serde(
        default,
        deserialize_with = "lenient_text",
        skip_serializing_if = "Option::is_none"
    )]
    pub media_type: Option<String>,
    /// Where the file is.
    #[serde(default, deserialize_with = "lenient_url")]
    pub url: Option<String>,
    /// Alt text.
    #[serde(
        default,
        deserialize_with = "lenient_text",
        skip_serializing_if = "Option::is_none"
    )]
    pub name: Option<String>,
    /// Width in pixels.
    #[serde(
        default,
        deserialize_with = "lenient_u32",
        skip_serializing_if = "Option::is_none"
    )]
    pub width: Option<u32>,
    /// Height in pixels.
    #[serde(
        default,
        deserialize_with = "lenient_u32",
        skip_serializing_if = "Option::is_none"
    )]
    pub height: Option<u32>,
}

fn lenient_u32<'de, D: Deserializer<'de>>(d: D) -> Result<Option<u32>, D::Error> {
    Ok(lenient_u64(d)?.and_then(|n| u32::try_from(n).ok()))
}

fn document_kind<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    let value = json!({ "type": Value::deserialize(d)? });
    kind_of::<D::Error>(&value, &["Document", "Image", "Video", "Audio"])
}

/// A `tag` entry. Anything tuita does not use is [`Tag::Other`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tag {
    /// A mention of an actor.
    Mention {
        /// The mentioned actor's id.
        href: String,
        /// How it is written, such as `@ada@example.org`.
        name: Option<String>,
    },
    /// A hashtag.
    Hashtag {
        /// The tag with its `#`.
        name: String,
        /// The tag's page.
        href: Option<String>,
    },
    /// A custom emote, `:name:` in the text.
    Emoji {
        /// The emote's id; the icon URL when the sender gave none.
        id: String,
        /// The shortcode without colons; always a valid [`crate::emoji`] shortcode.
        name: String,
        /// The image URL; always an [`is_http_url`].
        icon: String,
        /// The image's media type, when given.
        media_type: Option<String>,
    },
    /// Anything else.
    Other,
}

/// The `Emoji` tag `value` describes, or [`Tag::Other`] when its shortcode or icon is unusable.
fn emoji_tag(value: &Value) -> Tag {
    let text = |value: &Value, key: &str| value.get(key).and_then(Value::as_str).map(str::to_owned);
    let icon = match value.get("icon") {
        Some(Value::Array(items)) => items.iter().find(|item| item.is_object()),
        other => other,
    };
    let name = text(value, "name");
    let name = name
        .as_deref()
        .and_then(|name| name.strip_prefix(':')?.strip_suffix(':'))
        .filter(|name| is_valid_shortcode(name));
    let url = icon.and_then(|icon| match icon {
        Value::String(url) => Some(url.clone()),
        _ => text(icon, "url"),
    });
    let (Some(name), Some(icon_url)) = (name, url.filter(|url| is_http_url(url))) else {
        return Tag::Other;
    };
    Tag::Emoji {
        id: text(value, "id")
            .filter(|id| is_http_url(id))
            .unwrap_or_else(|| icon_url.clone()),
        name: name.to_owned(),
        media_type: icon.and_then(|icon| text(icon, "mediaType")),
        icon: icon_url,
    }
}

impl Serialize for Tag {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Mention { href, name } => {
                json!({"type": "Mention", "href": href, "name": name})
            }
            Self::Hashtag { name, href } => {
                json!({"type": "Hashtag", "name": name, "href": href})
            }
            Self::Emoji {
                id,
                name,
                icon,
                media_type,
            } => {
                let mut image = json!({"type": "Image", "url": icon});
                if let Some(media_type) = media_type {
                    image["mediaType"] = json!(media_type);
                }
                json!({"type": "Emoji", "id": id, "name": format!(":{name}:"), "icon": image})
            }
            Self::Other => json!({}),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Tag {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;
        let text = |key: &str| value.get(key).and_then(Value::as_str).map(str::to_owned);
        let names = type_names(&value);
        Ok(if names.contains(&"Mention") {
            match text("href") {
                Some(href) => Self::Mention {
                    href,
                    name: text("name"),
                },
                None => Self::Other,
            }
        } else if names.contains(&"Hashtag") {
            match text("name") {
                Some(name) => Self::Hashtag {
                    name,
                    href: text("href"),
                },
                None => Self::Other,
            }
        } else if names.contains(&"Emoji") {
            emoji_tag(&value)
        } else {
            Self::Other
        })
    }
}

/// One option of a poll.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PollOption {
    /// The option text.
    pub name: String,
    /// Replies, which count the votes: `{"totalItems": 3}`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replies: Option<VoteCount>,
}

/// `replies` of a poll option.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoteCount {
    /// How many votes.
    #[serde(default, deserialize_with = "lenient_u64")]
    pub total_items: Option<u64>,
}

/// A Note, Question, Article or Page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    /// The post's URL, its identity.
    pub id: String,
    /// Note, Question, ...
    #[serde(rename = "type")]
    pub kind: NoteKind,
    /// The author's actor id.
    #[serde(
        default,
        deserialize_with = "lenient_id",
        skip_serializing_if = "Option::is_none"
    )]
    pub attributed_to: Option<String>,
    /// The body, HTML.
    #[serde(
        default,
        deserialize_with = "lenient_text",
        skip_serializing_if = "Option::is_none"
    )]
    pub content: Option<String>,
    /// A title; on a poll vote, the chosen option.
    #[serde(
        default,
        deserialize_with = "lenient_text",
        skip_serializing_if = "Option::is_none"
    )]
    pub name: Option<String>,
    /// The content warning.
    #[serde(
        default,
        deserialize_with = "lenient_text",
        skip_serializing_if = "Option::is_none"
    )]
    pub summary: Option<String>,
    /// Media is sensitive.
    #[serde(default, deserialize_with = "lenient_bool")]
    pub sensitive: bool,
    /// Creation time, RFC 3339.
    #[serde(
        default,
        deserialize_with = "lenient_text",
        skip_serializing_if = "Option::is_none"
    )]
    pub published: Option<String>,
    /// Last edit, RFC 3339.
    #[serde(
        default,
        deserialize_with = "lenient_text",
        skip_serializing_if = "Option::is_none"
    )]
    pub updated: Option<String>,
    /// The post this answers.
    #[serde(
        default,
        deserialize_with = "lenient_id",
        skip_serializing_if = "Option::is_none"
    )]
    pub in_reply_to: Option<String>,
    /// The HTML page of the post.
    #[serde(
        default,
        deserialize_with = "lenient_url",
        skip_serializing_if = "Option::is_none"
    )]
    pub url: Option<String>,
    /// Who it is addressed to.
    #[serde(default, skip_serializing_if = "OneOrMany::is_empty")]
    pub to: OneOrMany<String>,
    /// Who else sees it.
    #[serde(default, skip_serializing_if = "OneOrMany::is_empty")]
    pub cc: OneOrMany<String>,
    /// Attached files.
    #[serde(default, skip_serializing_if = "OneOrMany::is_empty")]
    pub attachment: OneOrMany<Document>,
    /// Mentions, hashtags and custom emotes.
    #[serde(default, skip_serializing_if = "OneOrMany::is_empty")]
    pub tag: OneOrMany<Tag>,
    /// The quoted post (`quoteUrl`, also read from `quoteUri`).
    #[serde(
        default,
        alias = "quoteUri",
        deserialize_with = "lenient_first_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub quote_url: Option<String>,
    /// The quoted post as Misskey writes it.
    #[serde(
        default,
        rename = "_misskey_quote",
        deserialize_with = "lenient_first_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub misskey_quote: Option<String>,
    /// Poll options, one choice.
    #[serde(default, skip_serializing_if = "OneOrMany::is_empty")]
    pub one_of: OneOrMany<PollOption>,
    /// Poll options, several choices.
    #[serde(default, skip_serializing_if = "OneOrMany::is_empty")]
    pub any_of: OneOrMany<PollOption>,
    /// When the poll ends, RFC 3339.
    #[serde(
        default,
        deserialize_with = "lenient_text",
        skip_serializing_if = "Option::is_none"
    )]
    pub end_time: Option<String>,
    /// When the poll was closed, when it was.
    #[serde(
        default,
        deserialize_with = "lenient_text",
        skip_serializing_if = "Option::is_none"
    )]
    pub closed: Option<String>,
    /// How many accounts voted.
    #[serde(
        default,
        deserialize_with = "lenient_u64",
        skip_serializing_if = "Option::is_none"
    )]
    pub voters_count: Option<u64>,
}

impl HasId for Note {
    fn object_id(&self) -> Option<&str> {
        Some(&self.id)
    }
}

impl Note {
    /// The quoted post, from either spelling.
    #[must_use]
    pub fn quoted(&self) -> Option<&str> {
        self.quote_url.as_deref().or(self.misskey_quote.as_deref())
    }

    /// Whether the poll takes several choices.
    #[must_use]
    pub fn poll_is_multiple(&self) -> bool {
        self.one_of.is_empty() && !self.any_of.is_empty()
    }

    /// The poll options, whichever list holds them.
    #[must_use]
    pub fn poll_options(&self) -> &[PollOption] {
        if self.one_of.is_empty() {
            self.any_of.as_slice()
        } else {
            self.one_of.as_slice()
        }
    }
}

/// What a deleted object leaves behind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tombstone {
    /// The deleted object's id.
    pub id: String,
    /// The type it had.
    #[serde(
        default,
        deserialize_with = "lenient_first_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub former_type: Option<String>,
}

impl HasId for Tombstone {
    fn object_id(&self) -> Option<&str> {
        Some(&self.id)
    }
}

/// An `OrderedCollection`, `OrderedCollectionPage`, `Collection` or `CollectionPage`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    /// The collection's URL.
    #[serde(
        default,
        deserialize_with = "lenient_id",
        skip_serializing_if = "Option::is_none"
    )]
    pub id: Option<String>,
    /// The `type` as written.
    #[serde(rename = "type", deserialize_with = "collection_kind")]
    pub kind: String,
    /// How many items the whole collection has.
    #[serde(
        default,
        deserialize_with = "lenient_u64",
        skip_serializing_if = "Option::is_none"
    )]
    pub total_items: Option<u64>,
    /// The first page: its URL, or the embedded page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first: Option<Value>,
    /// The next page's URL.
    #[serde(
        default,
        deserialize_with = "lenient_id",
        skip_serializing_if = "Option::is_none"
    )]
    pub next: Option<String>,
    /// The collection a page belongs to.
    #[serde(
        default,
        deserialize_with = "lenient_id",
        skip_serializing_if = "Option::is_none"
    )]
    pub part_of: Option<String>,
    /// Items of an ordered page.
    #[serde(default, skip_serializing_if = "OneOrMany::is_empty")]
    pub ordered_items: OneOrMany<Value>,
    /// Items of an unordered page.
    #[serde(default, skip_serializing_if = "OneOrMany::is_empty")]
    pub items: OneOrMany<Value>,
}

fn collection_kind<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    let value = json!({ "type": Value::deserialize(d)? });
    kind_of::<D::Error>(
        &value,
        &[
            "OrderedCollection",
            "OrderedCollectionPage",
            "Collection",
            "CollectionPage",
        ],
    )
}

impl Collection {
    /// An `OrderedCollection` that only counts: followers, following and the like.
    #[must_use]
    pub fn counting(id: &str, total: u64) -> Self {
        Self {
            id: Some(id.to_owned()),
            kind: "OrderedCollection".to_owned(),
            total_items: Some(total),
            first: None,
            next: None,
            part_of: None,
            ordered_items: OneOrMany::default(),
            items: OneOrMany::default(),
        }
    }

    /// An `OrderedCollection` whose first page is at `first`.
    #[must_use]
    pub fn paged(id: &str, total: u64, first: &str) -> Self {
        Self {
            first: Some(Value::String(first.to_owned())),
            ..Self::counting(id, total)
        }
    }

    /// An `OrderedCollectionPage` of `items`.
    #[must_use]
    pub fn page(id: &str, part_of: &str, items: Vec<Value>, next: Option<&str>) -> Self {
        Self {
            id: Some(id.to_owned()),
            kind: "OrderedCollectionPage".to_owned(),
            total_items: None,
            first: None,
            next: next.map(str::to_owned),
            part_of: Some(part_of.to_owned()),
            ordered_items: OneOrMany::new(items),
            items: OneOrMany::default(),
        }
    }

    /// The items of this page, ordered or not.
    #[must_use]
    pub fn page_items(&self) -> &[Value] {
        if self.ordered_items.is_empty() {
            self.items.as_slice()
        } else {
            self.ordered_items.as_slice()
        }
    }

    /// The URL of the first page, or `None` when the page is embedded or absent.
    #[must_use]
    pub fn first_url(&self) -> Option<&str> {
        match self.first.as_ref()? {
            Value::String(url) => Some(url),
            Value::Object(map) => map.get("id").and_then(Value::as_str),
            _ => None,
        }
    }

    /// The embedded first page, when the collection carries it.
    #[must_use]
    pub fn first_page(&self) -> Option<Collection> {
        match self.first.as_ref()? {
            Value::Object(_) => serde_json::from_value(self.first.clone()?).ok(),
            _ => None,
        }
    }
}
