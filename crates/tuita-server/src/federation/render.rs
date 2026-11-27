//! Local accounts and posts as ActivityPub JSON.
//!
//! One layer for everything this server publishes: the actor and `Note` routes serve it, and
//! the delivery side wraps the same notes in `Create`, `Update` and `Delete`. Only objects of
//! local accounts are rendered; a remote post never goes back out.
//!
//! The `*_value` functions are pure over a loaded [`LocalPost`]; the async functions load the
//! rows first. Lists that Mastodon always sends as arrays (`to`, `cc`, `attachment`, `tag`,
//! `oneOf`, `anyOf`) are arrays here too, even with one item.
//!
//! Addressing follows Mastodon, with the mentioned, replied-to and quoted accounts as extra
//! recipients so their servers accept the post:
//!
//! | visibility | `to` | `cc` |
//! |---|---|---|
//! | public | Public | followers, recipients |
//! | unlisted | followers | Public, recipients |
//! | followers | followers | recipients |
//! | direct | recipients | |

use serde_json::{Value, json};
use time::OffsetDateTime;
use tuita_core::ap::{
    self, Activity, ActivityKind, Actor, ActorKind, Document, Endpoints, Image, Note, NoteKind,
    OneOrMany, PUBLIC, PollOption, PropertyValue, PublicKeyInfo, Tag, VoteCount, escape_html,
    format_time, is_http_url, rich_text_to_html,
};
use tuita_core::text::parse_post;
use uuid::Uuid;

use super::ids;
use super::keys;
use crate::config::Config;
use crate::db::ap::{self as rows, Attachment, EmojiRef, FileRef, LocalPost, Mentioned, PostRef};
use crate::db::settings;
use crate::db::users::UserRow;
use crate::dto::Visibility;
use crate::error::AppError;
use crate::state::AppState;

/// The absolute URL of the stored file `path` (`{id}.{ext}`).
#[must_use]
pub fn media_url(config: &Config, path: &str) -> String {
    format!("{}/media/{path}", config.public_url)
}

/// Wraps the values under `always` in arrays, adding an empty array where the key is missing,
/// and wraps the ones under `if_present` only when they exist.
fn arrays(doc: &mut Value, always: &[&str], if_present: &[&str]) {
    let Value::Object(map) = doc else {
        return;
    };
    for key in always {
        match map.get_mut(*key) {
            Some(Value::Array(_)) => {}
            Some(other) => *other = Value::Array(vec![other.take()]),
            None => {
                map.insert((*key).to_owned(), Value::Array(Vec::new()));
            }
        }
    }
    for key in if_present {
        if let Some(other) = map.get_mut(*key)
            && !other.is_array()
        {
            *other = Value::Array(vec![other.take()]);
        }
    }
}

fn image(config: &Config, file: Option<&FileRef>) -> Option<Image> {
    file.map(|file| Image {
        url: media_url(config, &file.path),
        media_type: Some(file.mime.clone()),
    })
}

fn website_html(url: &str) -> String {
    let shown = escape_html(url);
    if is_http_url(url) {
        format!(
            "<a href=\"{shown}\" rel=\"me nofollow noopener noreferrer\" target=\"_blank\">{shown}</a>"
        )
    } else {
        shown
    }
}

/// The profile fields: Website, and the organization the account is affiliated with.
fn fields(config: &Config, user: &UserRow) -> Vec<PropertyValue> {
    let mut out = Vec::new();
    if !user.website.trim().is_empty() {
        out.push(PropertyValue {
            name: "Website".to_owned(),
            value: website_html(user.website.trim()),
        });
    }
    if let Some(org) = &user.affiliation {
        let text = |key: &str| org.get(key).and_then(Value::as_str).unwrap_or_default();
        let handle = text("handle");
        if !handle.is_empty() {
            let name = match text("displayName") {
                "" => handle,
                name => name,
            };
            let link = format!(
                "<a href=\"{}\" rel=\"nofollow noopener noreferrer\">{}</a>",
                escape_html(&ids::profile_url(config, handle)),
                escape_html(name)
            );
            let value = match text("label") {
                "" => link,
                label => format!("{}, {link}", escape_html(label)),
            };
            out.push(PropertyValue {
                name: "Affiliated with".to_owned(),
                value,
            });
        }
    }
    out
}

/// The link a mention gets: the local profile page, or the remote account's own page.
fn mention_href(config: &Config, mentioned: &Mentioned) -> Option<String> {
    match &mentioned.domain {
        None => Some(ids::profile_url(config, &mentioned.handle)),
        Some(_) => mentioned
            .url
            .clone()
            .filter(|url| is_http_url(url))
            .or_else(|| mentioned.ap_id.clone().filter(|id| is_http_url(id))),
    }
}

/// The actor id of a mentioned account.
fn mention_actor(config: &Config, mentioned: &Mentioned) -> Option<String> {
    match &mentioned.domain {
        None => Some(ids::actor_id(config, &mentioned.handle)),
        Some(_) => mentioned.ap_id.clone().filter(|id| is_http_url(id)),
    }
}

/// The actor id of the author of a replied-to or quoted post.
fn ref_actor(config: &Config, post: &PostRef) -> Option<String> {
    match &post.domain {
        None => Some(ids::actor_id(config, &post.handle)),
        Some(_) => post.actor_ap_id.clone().filter(|id| is_http_url(id)),
    }
}

/// The id a post has on the network: its computed `Note` id, or a remote post's own.
fn ref_note(config: &Config, post: &PostRef) -> Option<String> {
    match &post.domain {
        None => Some(ids::note_id(config, &post.handle, post.id)),
        Some(_) => post.ap_id.clone().filter(|id| is_http_url(id)),
    }
}

/// The HTML of `text`: paragraphs, with links for the mentions in `mentions` and for hashtags.
/// Empty for a post without text.
#[must_use]
pub fn content_html(config: &Config, text: &str, mentions: &[Mentioned]) -> String {
    if text.trim().is_empty() {
        return String::new();
    }
    let local = config.domain();
    let mention_url = |handle: &str, domain: Option<&str>| {
        let wanted = domain.filter(|domain| !domain.eq_ignore_ascii_case(local));
        mentions
            .iter()
            .find(|mentioned| {
                mentioned.handle.eq_ignore_ascii_case(handle)
                    && match (&mentioned.domain, wanted) {
                        (None, None) => true,
                        (Some(have), Some(want)) => have.eq_ignore_ascii_case(want),
                        _ => false,
                    }
            })
            .and_then(|mentioned| mention_href(config, mentioned))
    };
    let tag_url = |tag: &str| format!("{}/tags/{tag}", config.public_url);
    rich_text_to_html(&parse_post(text), &mention_url, &tag_url)
}

/// The actor document of local account `user`: `Person`, `Service` for a bot, `Organization`
/// for an organization.
///
/// # Errors
///
/// Database failures and key failures.
pub async fn actor(state: &AppState, user: &UserRow) -> Result<Value, AppError> {
    let config = state.config();
    let pem = keys::user_public_pem(state, user.id).await?;
    let (avatar, banner) = rows::user_files(state.db(), user).await?;
    let mentions = rows::mentions_in(state.db(), config.domain(), &user.bio).await?;
    let texts = [user.display_name.as_str(), user.bio.as_str()];
    let emojis = rows::local_emojis(state.db(), &texts).await?;
    Ok(actor_value(
        config,
        user,
        &pem,
        (avatar.as_ref(), banner.as_ref()),
        &mentions,
        &rows::mentioned_emojis(&emojis, &texts),
    ))
}

/// [`actor`] over loaded data. `files` are the avatar and the banner; `mentions` are the
/// accounts the bio names and `emojis` the local emotes the display name and bio use.
#[must_use]
pub fn actor_value(
    config: &Config,
    user: &UserRow,
    public_pem: &str,
    files: (Option<&FileRef>, Option<&FileRef>),
    mentions: &[Mentioned],
    emojis: &[EmojiRef],
) -> Value {
    let id = ids::actor_id(config, &user.handle);
    let kind = if user.is_bot.unwrap_or(false) {
        ActorKind::Service
    } else if user.is_org.unwrap_or(false) {
        ActorKind::Organization
    } else {
        ActorKind::Person
    };
    let bio = content_html(config, &user.bio, mentions);
    let actor = Actor {
        id: id.clone(),
        kind,
        preferred_username: user.handle.clone(),
        name: (!user.display_name.is_empty()).then(|| user.display_name.clone()),
        summary: Some(bio),
        url: Some(ids::profile_url(config, &user.handle)),
        icon: image(config, files.0),
        image: image(config, files.1),
        inbox: ids::inbox_url(config, &user.handle),
        outbox: Some(ids::outbox_url(config, &user.handle)),
        followers: Some(ids::followers_url(config, &user.handle)),
        following: Some(ids::following_url(config, &user.handle)),
        featured: Some(ids::featured_url(config, &user.handle)),
        endpoints: Some(Endpoints {
            shared_inbox: Some(ids::shared_inbox_url(config)),
        }),
        public_key: Some(PublicKeyInfo {
            id: ids::key_id(&id),
            owner: id,
            public_key_pem: public_pem.to_owned(),
        }),
        manually_approves_followers: false,
        discoverable: Some(user.discoverable.unwrap_or(true)),
        published: Some(format_time(user.created_at)),
        attachment: OneOrMany::new(fields(config, user)),
        moved_to: None,
        tag: emojis
            .iter()
            .map(|emoji| emoji_tag(config, emoji))
            .collect(),
    };
    let mut doc = ap::document(&actor);
    arrays(&mut doc, &["attachment"], &["tag"]);
    doc
}

/// The instance actor, an `Application` that signs this server's fetches. Its
/// `preferredUsername` is the domain, which is what WebFinger's `acct:{domain}@{domain}` names.
///
/// # Errors
///
/// Database failures and key failures.
pub async fn instance_actor(state: &AppState) -> Result<Value, AppError> {
    let config = state.config();
    let pem = keys::instance_public_pem(state).await?;
    let name = settings::load(state.db()).await?.name;
    let id = ids::instance_actor_id(config);
    let actor = Actor {
        id: id.clone(),
        kind: ActorKind::Application,
        preferred_username: config.domain().to_owned(),
        name: Some(name),
        summary: None,
        url: Some(config.public_url.to_string()),
        icon: None,
        image: None,
        inbox: ids::shared_inbox_url(config),
        outbox: None,
        followers: None,
        following: None,
        featured: None,
        endpoints: None,
        public_key: Some(PublicKeyInfo {
            id: ids::key_id(&id),
            owner: id,
            public_key_pem: pem,
        }),
        manually_approves_followers: true,
        discoverable: Some(false),
        published: None,
        attachment: OneOrMany::default(),
        moved_to: None,
        tag: OneOrMany::default(),
    };
    Ok(ap::document(&actor))
}

fn dedup(mut list: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    list.retain(|item| seen.insert(item.clone()));
    list
}

/// The accounts a post is also sent to: those it mentions, the author of the post it answers
/// and the author of the post it quotes, other than its own author.
///
/// Local posts are always public today. A non-public post must not go out like this: it needs
/// addressing to the mentioned accounts only, and delivery to their personal inboxes, before
/// local visibility ships (the reply and quote authors would otherwise learn of a private post).
#[must_use]
pub fn recipients(config: &Config, post: &LocalPost) -> Vec<String> {
    let own = ids::actor_id(config, &post.author.handle);
    let all = post
        .mentions
        .iter()
        .filter_map(|mentioned| mention_actor(config, mentioned))
        .chain(
            post.reply_to
                .iter()
                .filter_map(|parent| ref_actor(config, parent)),
        )
        .chain(
            post.quote_of
                .iter()
                .filter_map(|quoted| ref_actor(config, quoted)),
        )
        .filter(|actor| *actor != own)
        .collect();
    dedup(all)
}

/// The `to` and `cc` of a post, by visibility (see the module table).
///
/// Local posts are always public today; see [`recipients`] for what a non-public post needs
/// before it can ship.
#[must_use]
pub fn audience(config: &Config, post: &LocalPost) -> (Vec<String>, Vec<String>) {
    let followers = ids::followers_url(config, &post.author.handle);
    let extra = recipients(config, post);
    match post.visibility {
        Visibility::Public => (
            vec![PUBLIC.to_owned()],
            std::iter::once(followers).chain(extra).collect(),
        ),
        Visibility::Unlisted => (
            vec![followers],
            std::iter::once(PUBLIC.to_owned()).chain(extra).collect(),
        ),
        Visibility::Followers => (vec![followers], extra),
        Visibility::Direct => (extra, Vec::new()),
    }
}

fn attachment(config: &Config, media: &Attachment) -> Document {
    Document {
        kind: "Document".to_owned(),
        media_type: Some(media.file.mime.clone()),
        url: Some(media_url(config, &media.file.path)),
        name: (!media.alt.is_empty()).then(|| media.alt.clone()),
        width: media.width.and_then(|n| u32::try_from(n).ok()),
        height: media.height.and_then(|n| u32::try_from(n).ok()),
    }
}

/// The `Emoji` tag of a local emote, with the absolute address of its image.
fn emoji_tag(config: &Config, emoji: &EmojiRef) -> Tag {
    Tag::Emoji {
        id: format!("{}/emojis/{}", config.public_url, emoji.shortcode),
        name: emoji.shortcode.clone(),
        icon: media_url(config, &emoji.file.path),
        media_type: Some(emoji.file.mime.clone()),
    }
}

/// The `Note` (or `Question`, for a poll) of `post`, without `@context`.
#[must_use]
pub fn note_object(config: &Config, post: &LocalPost) -> Value {
    let author = ids::actor_id(config, &post.author.handle);
    let (to, cc) = audience(config, post);
    let rich = parse_post(&post.text);
    let tag: Vec<Tag> = post
        .mentions
        .iter()
        .filter_map(|mentioned| {
            let href = mention_actor(config, mentioned)?;
            let name = match &mentioned.domain {
                None => format!("@{}", mentioned.handle),
                Some(domain) => format!("@{}@{domain}", mentioned.handle),
            };
            Some(Tag::Mention {
                href,
                name: Some(name),
            })
        })
        .chain(rich.hashtags().into_iter().map(|tag| Tag::Hashtag {
            name: format!("#{tag}"),
            href: Some(format!("{}/tags/{tag}", config.public_url)),
        }))
        .chain(post.emojis.iter().map(|emoji| emoji_tag(config, emoji)))
        .collect();
    let options: Vec<PollOption> = post
        .poll
        .iter()
        .flat_map(|poll| {
            poll.options.iter().map(|(name, votes)| PollOption {
                name: name.clone(),
                replies: Some(VoteCount {
                    total_items: Some(u64::try_from(*votes).unwrap_or(0)),
                }),
            })
        })
        .collect();
    let multiple = post.poll.as_ref().is_some_and(|poll| poll.multiple);
    let now = OffsetDateTime::now_utc();
    let note = Note {
        id: ids::note_id(config, &post.author.handle, post.id),
        kind: if post.poll.is_some() {
            NoteKind::Question
        } else {
            NoteKind::Note
        },
        attributed_to: Some(author),
        content: Some(content_html(config, &post.text, &post.mentions)),
        name: None,
        summary: (!post.cw.is_empty()).then(|| post.cw.clone()),
        sensitive: post.sensitive || !post.cw.is_empty(),
        published: Some(format_time(post.created_at)),
        updated: post.edited_at.map(format_time),
        in_reply_to: post
            .reply_to
            .as_ref()
            .and_then(|parent| ref_note(config, parent)),
        url: Some(ids::note_url(config, &post.author.handle, post.id)),
        to: OneOrMany::new(to),
        cc: OneOrMany::new(cc),
        attachment: post
            .media
            .iter()
            .map(|media| attachment(config, media))
            .collect(),
        tag: OneOrMany::new(tag),
        quote_url: post
            .quote_of
            .as_ref()
            .and_then(|quoted| ref_note(config, quoted)),
        misskey_quote: post
            .quote_of
            .as_ref()
            .and_then(|quoted| ref_note(config, quoted)),
        one_of: if multiple {
            OneOrMany::default()
        } else {
            OneOrMany::new(options.clone())
        },
        any_of: if multiple {
            OneOrMany::new(options)
        } else {
            OneOrMany::default()
        },
        end_time: post.poll.as_ref().map(|poll| format_time(poll.ends_at)),
        closed: post
            .poll
            .as_ref()
            .filter(|poll| poll.ends_at <= now)
            .map(|poll| format_time(poll.ends_at)),
        voters_count: post
            .poll
            .as_ref()
            .map(|poll| u64::try_from(poll.voters).unwrap_or(0)),
    };
    let mut doc = serde_json::to_value(&note).unwrap_or(Value::Null);
    arrays(
        &mut doc,
        &["to", "cc", "attachment", "tag"],
        &["oneOf", "anyOf"],
    );
    doc
}

/// The `Note` document of `post`, with `@context`.
#[must_use]
pub fn note_document(config: &Config, post: &LocalPost) -> Value {
    let mut doc = note_object(config, post);
    if let Value::Object(map) = &mut doc {
        map.insert("@context".to_owned(), ap::context());
    }
    doc
}

/// The id of the `Create` of `post`: `{note id}/activity`, which the route of that name serves.
#[must_use]
pub fn create_id(config: &Config, post: &LocalPost) -> String {
    format!(
        "{}/activity",
        ids::note_id(config, &post.author.handle, post.id)
    )
}

/// The id of the `Update` for the latest edit of `post`: `{note id}#updates/{unix time of the
/// edit}`, so every edit has its own id.
#[must_use]
pub fn update_id(config: &Config, post: &LocalPost) -> String {
    let at = post.edited_at.unwrap_or(post.created_at).unix_timestamp();
    format!(
        "{}#updates/{at}",
        ids::note_id(config, &post.author.handle, post.id)
    )
}

/// The id of the `Delete` of `post`: `{note id}#delete`.
#[must_use]
pub fn delete_id(config: &Config, post: &LocalPost) -> String {
    format!(
        "{}#delete",
        ids::note_id(config, &post.author.handle, post.id)
    )
}

fn wrap(config: &Config, post: &LocalPost, kind: ActivityKind, id: String, object: Value) -> Value {
    let (to, cc) = audience(config, post);
    let mut activity = Activity::new(
        kind,
        Some(id),
        &ids::actor_id(config, &post.author.handle),
        object,
    )
    .addressed(to, cc);
    activity.published = Some(format_time(match kind {
        ActivityKind::Update => post.edited_at.unwrap_or(post.created_at),
        ActivityKind::Delete => post.deleted_at.unwrap_or_else(OffsetDateTime::now_utc),
        _ => post.created_at,
    }));
    let mut doc = serde_json::to_value(&activity).unwrap_or(Value::Null);
    arrays(&mut doc, &["to", "cc"], &[]);
    doc
}

/// The `Create` of `post` around its note, without `@context`: an item of the outbox.
#[must_use]
pub fn create_object(config: &Config, post: &LocalPost) -> Value {
    wrap(
        config,
        post,
        ActivityKind::Create,
        create_id(config, post),
        note_object(config, post),
    )
}

fn with_context(mut doc: Value) -> Value {
    if let Value::Object(map) = &mut doc {
        map.insert("@context".to_owned(), ap::context());
    }
    doc
}

/// `Create` of `post` around its note, with `@context`.
#[must_use]
pub fn create_value(config: &Config, post: &LocalPost) -> Value {
    with_context(create_object(config, post))
}

/// `Update` of `post` around its current note, which carries `updated`.
#[must_use]
pub fn update_value(config: &Config, post: &LocalPost) -> Value {
    with_context(wrap(
        config,
        post,
        ActivityKind::Update,
        update_id(config, post),
        note_object(config, post),
    ))
}

/// `Delete` of `post` around a `Tombstone` of its note id.
#[must_use]
pub fn delete_value(config: &Config, post: &LocalPost) -> Value {
    let tombstone = json!({
        "id": ids::note_id(config, &post.author.handle, post.id),
        "type": "Tombstone",
    });
    with_context(wrap(
        config,
        post,
        ActivityKind::Delete,
        delete_id(config, post),
        tombstone,
    ))
}

/// Loads the post of a local account, deleted or not; `None` for an unknown id or a remote
/// author.
///
/// # Errors
///
/// Database failures.
pub async fn load(state: &AppState, post: Uuid) -> Result<Option<LocalPost>, AppError> {
    rows::load_post(state.db(), state.config().domain(), post).await
}

/// The `Note` document of a local post; `None` for an unknown, remote or deleted post. The
/// caller decides who may read it (the route serves public and unlisted posts only).
///
/// # Errors
///
/// Database failures.
pub async fn note(state: &AppState, post: Uuid) -> Result<Option<Value>, AppError> {
    Ok(load(state, post)
        .await?
        .filter(|post| post.deleted_at.is_none())
        .map(|post| note_document(state.config(), &post)))
}

/// The `Create` activity of a local post; `None` for an unknown, remote or deleted post.
///
/// # Errors
///
/// Database failures.
pub async fn create(state: &AppState, post: Uuid) -> Result<Option<Value>, AppError> {
    Ok(load(state, post)
        .await?
        .filter(|post| post.deleted_at.is_none())
        .map(|post| create_value(state.config(), &post)))
}

/// The `Update` activity of a local post; `None` for an unknown, remote or deleted post.
///
/// # Errors
///
/// Database failures.
pub async fn update(state: &AppState, post: Uuid) -> Result<Option<Value>, AppError> {
    Ok(load(state, post)
        .await?
        .filter(|post| post.deleted_at.is_none())
        .map(|post| update_value(state.config(), &post)))
}

/// The `Delete` activity of a local post, which may already be soft-deleted; `None` for an
/// unknown or remote post.
///
/// # Errors
///
/// Database failures.
pub async fn delete(state: &AppState, post: Uuid) -> Result<Option<Value>, AppError> {
    Ok(load(state, post)
        .await?
        .map(|post| delete_value(state.config(), &post)))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::db::ap::{Attachment, LocalAuthor, PollData};

    fn config() -> Config {
        Config::from_lookup(|var| match var {
            "DATABASE_URL" => Some("postgres://x".to_owned()),
            "PUBLIC_URL" => Some("https://a.example".to_owned()),
            _ => None,
        })
        .expect("config")
    }

    fn post(visibility: Visibility) -> LocalPost {
        let created_at = OffsetDateTime::from_unix_timestamp(1_700_000_000).expect("time");
        LocalPost {
            id: Uuid::from_u128(1),
            author: LocalAuthor {
                id: Uuid::from_u128(2),
                handle: "ada".to_owned(),
                display_name: "Ada".to_owned(),
                bio: String::new(),
                bot: false,
                org: false,
                discoverable: true,
                created_at,
                followers: 0,
                following: 0,
                posts: 1,
                suspended: false,
                deleted: false,
                avatar: None,
                banner: None,
            },
            text: "hello & <b>".to_owned(),
            cw: String::new(),
            sensitive: false,
            created_at,
            edited_at: None,
            deleted_at: None,
            visibility,
            client: None,
            views: 0,
            like_count: 0,
            repost_count: 0,
            reply_count: 0,
            quote_count: 0,
            reply_to: None,
            quote_of: None,
            media: Vec::new(),
            poll: None,
            mentions: Vec::new(),
            emojis: Vec::new(),
        }
    }

    const NOTE_ID: &str =
        "https://a.example/users/ada/statuses/00000000-0000-0000-0000-000000000001";
    const FOLLOWERS: &str = "https://a.example/users/ada/followers";

    #[test]
    fn a_plain_note_has_the_mastodon_shape() {
        let doc = note_object(&config(), &post(Visibility::Public));
        assert_eq!(
            doc,
            json!({
                "id": NOTE_ID,
                "type": "Note",
                "attributedTo": "https://a.example/users/ada",
                "content": "<p>hello &amp; &lt;b&gt;</p>",
                "sensitive": false,
                "published": "2023-11-14T22:13:20Z",
                "url": "https://a.example/@ada/00000000-0000-0000-0000-000000000001",
                "to": [PUBLIC],
                "cc": [FOLLOWERS],
                "attachment": [],
                "tag": [],
            })
        );
    }

    #[test]
    fn addressing_follows_visibility() {
        let config = config();
        let mut direct = post(Visibility::Direct);
        direct.mentions.push(Mentioned {
            id: Uuid::from_u128(3),
            handle: "bob".to_owned(),
            domain: Some("b.example".to_owned()),
            ap_id: Some("https://b.example/users/bob".to_owned()),
            url: None,
        });
        let cases = [
            (
                Visibility::Public,
                vec![PUBLIC],
                vec![FOLLOWERS, "https://b.example/users/bob"],
            ),
            (
                Visibility::Unlisted,
                vec![FOLLOWERS],
                vec![PUBLIC, "https://b.example/users/bob"],
            ),
            (
                Visibility::Followers,
                vec![FOLLOWERS],
                vec!["https://b.example/users/bob"],
            ),
            (
                Visibility::Direct,
                vec!["https://b.example/users/bob"],
                vec![],
            ),
        ];
        for (visibility, to, cc) in cases {
            direct.visibility = visibility;
            let (got_to, got_cc) = audience(&config, &direct);
            assert_eq!(got_to, to, "{visibility:?}");
            assert_eq!(got_cc, cc, "{visibility:?}");
        }
    }

    #[test]
    fn replies_quotes_and_the_reply_parents_author_are_addressed() {
        let config = config();
        let mut reply = post(Visibility::Public);
        reply.reply_to = Some(PostRef {
            id: Uuid::from_u128(9),
            author_id: Uuid::from_u128(8),
            handle: "bob".to_owned(),
            domain: Some("b.example".to_owned()),
            ap_id: Some("https://b.example/notes/9".to_owned()),
            actor_ap_id: Some("https://b.example/users/bob".to_owned()),
        });
        reply.quote_of = Some(PostRef {
            id: Uuid::from_u128(7),
            author_id: Uuid::from_u128(2),
            handle: "ada".to_owned(),
            domain: None,
            ap_id: None,
            actor_ap_id: None,
        });
        let doc = note_object(&config, &reply);
        assert_eq!(doc["inReplyTo"], "https://b.example/notes/9");
        let quote = "https://a.example/users/ada/statuses/00000000-0000-0000-0000-000000000007";
        assert_eq!(doc["quoteUrl"], quote);
        assert_eq!(doc["_misskey_quote"], quote);
        assert_eq!(doc["cc"], json!([FOLLOWERS, "https://b.example/users/bob"]));
    }

    #[test]
    fn media_and_hashtags_become_documents_and_tags() {
        let mut media_post = post(Visibility::Public);
        media_post.text = "look #Rust".to_owned();
        media_post.cw = "spoiler".to_owned();
        media_post.media.push(Attachment {
            id: Uuid::from_u128(5),
            file: FileRef {
                path: "5.png".to_owned(),
                mime: "image/png".to_owned(),
            },
            kind: "image".to_owned(),
            alt: "a cat".to_owned(),
            width: Some(640),
            height: Some(480),
            poster: None,
        });
        let doc = note_object(&config(), &media_post);
        assert_eq!(doc["summary"], "spoiler");
        assert_eq!(doc["sensitive"], true);
        assert_eq!(
            doc["attachment"],
            json!([{
                "type": "Document",
                "mediaType": "image/png",
                "url": "https://a.example/media/5.png",
                "name": "a cat",
                "width": 640,
                "height": 480,
            }])
        );
        assert_eq!(
            doc["tag"],
            json!([{"type": "Hashtag", "name": "#rust", "href": "https://a.example/tags/rust"}])
        );
        assert!(
            doc["content"]
                .as_str()
                .is_some_and(|html| html.contains("href=\"https://a.example/tags/rust\""))
        );
    }

    #[test]
    fn a_poll_is_a_question_with_counted_options() {
        let mut poll_post = post(Visibility::Public);
        poll_post.poll = Some(PollData {
            options: vec![("yes".to_owned(), 3), ("no".to_owned(), 1)],
            multiple: false,
            ends_at: OffsetDateTime::from_unix_timestamp(4_000_000_000).expect("time"),
            voters: 4,
        });
        let doc = note_object(&config(), &poll_post);
        assert_eq!(doc["type"], "Question");
        assert_eq!(doc["endTime"], "2096-10-02T07:06:40Z");
        assert_eq!(doc["votersCount"], 4);
        assert_eq!(
            doc["oneOf"],
            json!([
                {"name": "yes", "replies": {"totalItems": 3}},
                {"name": "no", "replies": {"totalItems": 1}},
            ])
        );
        assert!(doc.get("anyOf").is_none());
        assert!(doc.get("closed").is_none());
        poll_post.poll.as_mut().expect("poll").multiple = true;
        let doc = note_object(&config(), &poll_post);
        assert!(doc.get("oneOf").is_none());
        assert_eq!(doc["anyOf"].as_array().map(Vec::len), Some(2));
    }

    #[test]
    fn activities_wrap_the_note_with_stable_ids() {
        let config = config();
        let mut edited = post(Visibility::Public);
        edited.edited_at = Some(OffsetDateTime::from_unix_timestamp(1_700_000_100).expect("time"));
        let create = create_value(&config, &edited);
        assert_eq!(create["id"], format!("{NOTE_ID}/activity"));
        assert_eq!(create["type"], "Create");
        assert_eq!(create["actor"], "https://a.example/users/ada");
        assert_eq!(create["object"]["id"], NOTE_ID);
        assert_eq!(create["object"]["updated"], "2023-11-14T22:15:00Z");
        assert!(create["object"].get("@context").is_none());
        assert_eq!(create["to"], json!([PUBLIC]));
        assert_eq!(
            create["@context"][0],
            "https://www.w3.org/ns/activitystreams"
        );
        let update = update_value(&config, &edited);
        assert_eq!(update["id"], format!("{NOTE_ID}#updates/1700000100"));
        assert_eq!(update["type"], "Update");
        let delete = delete_value(&config, &edited);
        assert_eq!(delete["id"], format!("{NOTE_ID}#delete"));
        assert_eq!(delete["type"], "Delete");
        assert_eq!(
            delete["object"],
            json!({"id": NOTE_ID, "type": "Tombstone"})
        );
    }

    #[test]
    fn local_emotes_are_emoji_tags_with_absolute_urls() {
        let emoji = EmojiRef {
            shortcode: "blob".to_owned(),
            file: FileRef {
                path: "abc.webp".to_owned(),
                mime: "image/webp".to_owned(),
            },
        };
        let mut with = post(Visibility::Public);
        with.text = "hi :blob:".to_owned();
        with.emojis = vec![emoji];
        assert_eq!(
            note_object(&config(), &with)["tag"],
            json!([{
                "type": "Emoji",
                "id": "https://a.example/emojis/blob",
                "name": ":blob:",
                "icon": {
                    "type": "Image",
                    "mediaType": "image/webp",
                    "url": "https://a.example/media/abc.webp"
                },
            }])
        );
        assert_eq!(
            note_object(&config(), &post(Visibility::Public))["tag"],
            json!([])
        );
    }
}
