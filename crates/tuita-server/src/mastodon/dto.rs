//! The Mastodon entities of the client API subset, and the mappers from tuita's own values.
//!
//! The JSON shapes follow <https://docs.joinmastodon.org/entities/>: snake_case, ids as UUID
//! strings (post ids for statuses, entry ids for boosts, user ids for accounts), arrays where
//! Mastodon sends arrays (never `null`), timestamps as `2026-11-09T10:00:00.000Z`. These types
//! are serialized only; they are not exported to other languages.
//!
//! The mappers are pure. Everything a mapper needs besides its input value is in a
//! [`Ctx`]; a handler builds one per response with [`Ctx::for_posts`] or [`Ctx::for_users`].
//!
//! | tuita value | mapper | Mastodon entity |
//! |---|---|---|
//! | [`UserDto`] | [`account`] | `Account` |
//! | [`UserRefDto`] | [`account_from_ref`] | `Account` |
//! | [`MeDto`] | [`credential_account`] | `CredentialAccount` |
//! | [`Relation`] | [`relationship`] | `Relationship` |
//! | [`PostDto`] | [`status`] | `Status` (a boost is a status with `reblog`) |
//! | [`MediaDto`] | [`media_attachment`] | `MediaAttachment` |
//! | [`PollDto`] | [`poll`] | `Poll` |
//! | [`LinkCardDto`] | [`preview_card`] | `PreviewCard` |
//! | notification fields | [`notification`] | `Notification` |
//! | [`ListDto`] | [`list`] | `List` |
//! | [`MessageDto`], [`ConversationDto`] | [`message_status`], [`conversation`] | `Status` (direct), `Conversation` |

use serde::{Serialize, Serializer};
use time::{OffsetDateTime, UtcOffset};
use tuita_core::ap::escape_html;
use uuid::Uuid;

use super::ctx::Ctx;
use crate::config::Config;
use crate::db::ap::Mentioned;
use crate::db::mastodon_misc::DayUse;
use crate::dto::{
    AnnouncementDto, ConversationDto, EmojiDto, LinkCardDto, ListDto, MeDto, MediaDto, MediaKind,
    MessageDto, NotificationKindDto, PollDto, PostDto, Relation, SegmentDto, UserDto, UserRefDto,
    Visibility,
};
use crate::federation::ids;
use crate::federation::render::content_html;

#[allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "serde's `serialize_with` passes the field by reference"
)]
fn serialize_time<S: Serializer>(at: &OffsetDateTime, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&iso(*at))
}

#[allow(
    clippy::ref_option,
    clippy::trivially_copy_pass_by_ref,
    reason = "serde's `serialize_with` passes the field by reference"
)]
fn serialize_time_option<S: Serializer>(
    at: &Option<OffsetDateTime>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    match at {
        Some(at) => serializer.serialize_str(&iso(*at)),
        None => serializer.serialize_none(),
    }
}

/// `2026-11-09T10:00:00.000Z`: the timestamp format Mastodon sends, UTC with milliseconds.
#[must_use]
pub fn iso(at: OffsetDateTime) -> String {
    let at = at.to_offset(UtcOffset::UTC);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        at.year(),
        u8::from(at.month()),
        at.day(),
        at.hour(),
        at.minute(),
        at.second(),
        at.millisecond()
    )
}

/// `2026-11-09`: the date format of `last_status_at`.
#[must_use]
pub fn date(at: OffsetDateTime) -> String {
    let at = at.to_offset(UtcOffset::UTC);
    format!(
        "{:04}-{:02}-{:02}",
        at.year(),
        u8::from(at.month()),
        at.day()
    )
}

/// The creation time inside a UUIDv7, which is how every id of this server is made. The Unix
/// epoch for any other id.
#[must_use]
pub fn uuid_time(id: Uuid) -> OffsetDateTime {
    id.get_timestamp()
        .and_then(|timestamp| {
            let (seconds, nanos) = timestamp.to_unix();
            let nanos = i128::from(seconds) * 1_000_000_000 + i128::from(nanos);
            OffsetDateTime::from_unix_timestamp_nanos(nanos).ok()
        })
        .unwrap_or(OffsetDateTime::UNIX_EPOCH)
}

/// `url` as an absolute URL: stored media paths (`/media/...`) get the `PUBLIC_URL` origin.
#[must_use]
pub fn absolute(config: &Config, url: &str) -> String {
    if url.starts_with('/') {
        format!("{}{url}", config.public_url)
    } else {
        url.to_owned()
    }
}

// ------------------------------------------------------------------ accounts

/// A custom emote: a `:shortcode:` image, local or copied from another instance.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CustomEmoji {
    /// The name between colons.
    pub shortcode: String,
    /// The image.
    pub url: String,
    /// The still image.
    pub static_url: String,
    /// Whether pickers list it.
    pub visible_in_picker: bool,
    /// The picker category.
    pub category: Option<String>,
}

/// `emoji` as Mastodon shows it. Tuita keeps one image per emote, so `static_url` is `url`.
#[must_use]
pub fn custom_emoji(config: &Config, emoji: &EmojiDto) -> CustomEmoji {
    let url = absolute(config, &emoji.url);
    CustomEmoji {
        shortcode: emoji.shortcode.clone(),
        static_url: url.clone(),
        url,
        visible_in_picker: true,
        category: emoji.category.clone(),
    }
}

/// [`custom_emoji`] for each of `emojis`.
#[must_use]
pub fn custom_emojis(config: &Config, emojis: &[EmojiDto]) -> Vec<CustomEmoji> {
    emojis
        .iter()
        .map(|emoji| custom_emoji(config, emoji))
        .collect()
}

/// One profile field: a label and an HTML value.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AccountField {
    /// The label.
    pub name: String,
    /// The value as HTML.
    pub value: String,
    /// When the link was verified; always `null` here.
    pub verified_at: Option<String>,
}

/// An account, local or remote.
#[allow(
    clippy::struct_excessive_bools,
    reason = "wire format: one JSON flag per fact"
)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Account {
    /// The user id.
    pub id: Uuid,
    /// The handle without `@`.
    pub username: String,
    /// `handle` for local accounts, `handle@domain` for remote ones.
    pub acct: String,
    /// The profile page.
    pub url: String,
    /// The ActivityPub actor id.
    pub uri: String,
    /// The display name.
    pub display_name: String,
    /// The biography as HTML.
    pub note: String,
    /// The avatar image.
    pub avatar: String,
    /// The avatar image, not animated.
    pub avatar_static: String,
    /// The banner image, or empty.
    pub header: String,
    /// The banner image, not animated, or empty.
    pub header_static: String,
    /// Follows need approval.
    pub locked: bool,
    /// Website and location as fields.
    pub fields: Vec<AccountField>,
    /// The custom emotes the display name, bio and fields use.
    pub emojis: Vec<CustomEmoji>,
    /// An automated account.
    pub bot: bool,
    /// A group actor; never.
    pub group: bool,
    /// Listed in directories.
    pub discoverable: Option<bool>,
    /// Sign-up time.
    #[serde(serialize_with = "serialize_time")]
    pub created_at: OffsetDateTime,
    /// The date of the last public or unlisted post.
    pub last_status_at: Option<String>,
    /// Posts that are not deleted.
    pub statuses_count: i64,
    /// Accounts following this one.
    pub followers_count: i64,
    /// Accounts this one follows.
    pub following_count: i64,
}

/// The role of an account on the instance.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AccountRole {
    /// The role id; `-99` is the everyone role.
    pub id: String,
    /// The role name; empty for everyone.
    pub name: String,
    /// The permission bitmask as a decimal string.
    pub permissions: String,
    /// A CSS color, or empty.
    pub color: String,
    /// Shown on the profile.
    pub highlighted: bool,
}

/// A profile field as typed, without HTML.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SourceField {
    /// The label.
    pub name: String,
    /// The value as typed.
    pub value: String,
    /// When the link was verified; always `null` here.
    pub verified_at: Option<String>,
}

/// The private part of the signed-in account: what the edit form needs.
#[allow(
    clippy::struct_excessive_bools,
    reason = "wire format: one JSON flag per fact"
)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AccountSource {
    /// Default visibility of new posts.
    pub privacy: StatusVisibility,
    /// Default sensitivity of new posts.
    pub sensitive: bool,
    /// Default language of new posts.
    pub language: Option<String>,
    /// The biography as typed.
    pub note: String,
    /// The profile fields as typed.
    pub fields: Vec<SourceField>,
    /// Pending follow requests.
    pub follow_requests_count: i64,
    /// Hide followers and following; never here.
    pub hide_collections: Option<bool>,
    /// Listed in directories.
    pub discoverable: Option<bool>,
    /// Posts may be indexed by search engines.
    pub indexable: bool,
}

/// The account returned by `verify_credentials` and `update_credentials`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CredentialAccount {
    /// The public part.
    #[serde(flatten)]
    pub account: Account,
    /// The private part.
    pub source: AccountSource,
    /// The role.
    pub role: AccountRole,
}

/// How the viewer and an account relate.
#[allow(
    clippy::struct_excessive_bools,
    reason = "wire format: one JSON flag per fact"
)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Relationship {
    /// The account id.
    pub id: Uuid,
    /// The viewer follows the account.
    pub following: bool,
    /// The viewer sees the account's boosts.
    pub showing_reblogs: bool,
    /// The viewer gets a notification for each post.
    pub notifying: bool,
    /// Languages the viewer follows the account in.
    pub languages: Option<Vec<String>>,
    /// The account follows the viewer.
    pub followed_by: bool,
    /// The viewer blocked the account.
    pub blocking: bool,
    /// The account blocked the viewer.
    pub blocked_by: bool,
    /// The viewer muted the account.
    pub muting: bool,
    /// The mute covers notifications.
    pub muting_notifications: bool,
    /// The viewer asked to follow and waits.
    pub requested: bool,
    /// The account asked to follow the viewer.
    pub requested_by: bool,
    /// The viewer blocked the account's domain.
    pub domain_blocking: bool,
    /// The viewer features the account on the profile.
    pub endorsed: bool,
    /// The viewer's private note.
    pub note: String,
}

// ------------------------------------------------------------------ statuses

/// Who sees a status, in Mastodon's words.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StatusVisibility {
    /// Everyone, listed.
    Public,
    /// Everyone with the link.
    Unlisted,
    /// Followers (`followers` in tuita).
    Private,
    /// The people mentioned.
    Direct,
}

impl From<Visibility> for StatusVisibility {
    fn from(visibility: Visibility) -> Self {
        match visibility {
            Visibility::Public => Self::Public,
            Visibility::Unlisted => Self::Unlisted,
            Visibility::Followers => Self::Private,
            Visibility::Direct => Self::Direct,
        }
    }
}

impl StatusVisibility {
    /// The tuita visibility this one means.
    #[must_use]
    pub const fn to_tuita(self) -> Visibility {
        match self {
            Self::Public => Visibility::Public,
            Self::Unlisted => Visibility::Unlisted,
            Self::Private => Visibility::Followers,
            Self::Direct => Visibility::Direct,
        }
    }
}

/// An account a status mentions.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Mention {
    /// The user id.
    pub id: Uuid,
    /// The handle without `@`.
    pub username: String,
    /// The profile page.
    pub url: String,
    /// `handle` or `handle@domain`.
    pub acct: String,
}

/// A hashtag, as listed on a status or on its own.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Tag {
    /// The lower-case tag without `#`.
    pub name: String,
    /// The page that lists the tag.
    pub url: String,
    /// Usage in the last days; sent by the tag endpoints only.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub history: Vec<TagHistory>,
    /// Whether the viewer follows the tag; sent by the tag endpoints only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub following: Option<bool>,
}

/// Usage of a hashtag on one day. Mastodon sends every number as a string.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TagHistory {
    /// The day as a Unix timestamp.
    pub day: String,
    /// Posts that used the tag.
    pub uses: String,
    /// Accounts that used it.
    pub accounts: String,
}

/// The size of an image or video.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct MediaSize {
    /// Width in pixels.
    pub width: i32,
    /// Height in pixels.
    pub height: i32,
    /// `{width}x{height}`.
    pub size: String,
    /// Width divided by height.
    pub aspect: f64,
}

/// The `meta` of a media attachment; `{}` when the size is unknown.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct MediaMeta {
    /// The file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original: Option<MediaSize>,
    /// The preview, which is the file itself here.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small: Option<MediaSize>,
}

/// An image or video on a status, or an upload not attached yet.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct MediaAttachment {
    /// The media id.
    pub id: Uuid,
    /// `image` or `video`.
    #[serde(rename = "type")]
    pub kind: &'static str,
    /// The file.
    pub url: String,
    /// The preview: a video's poster, an image itself.
    pub preview_url: String,
    /// The file's original URL on another server; never set.
    pub remote_url: Option<String>,
    /// Deprecated; never set.
    pub text_url: Option<String>,
    /// Sizes.
    pub meta: MediaMeta,
    /// The description for screen readers.
    pub description: Option<String>,
    /// A blurhash; never set.
    pub blurhash: Option<String>,
}

/// One answer of a poll.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PollOption {
    /// The answer text.
    pub title: String,
    /// Votes for it.
    pub votes_count: Option<i32>,
}

/// A poll. Its id is the id of the post that carries it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Poll {
    /// The post id.
    pub id: Uuid,
    /// When voting ends.
    #[serde(serialize_with = "serialize_time_option")]
    pub expires_at: Option<OffsetDateTime>,
    /// Voting has ended.
    pub expired: bool,
    /// A voter may pick several answers.
    pub multiple: bool,
    /// All votes.
    pub votes_count: i32,
    /// Accounts that voted; `null` unless `multiple`.
    pub voters_count: Option<i32>,
    /// The viewer voted.
    pub voted: bool,
    /// Indexes of the answers the viewer picked.
    pub own_votes: Vec<u8>,
    /// The answers.
    pub options: Vec<PollOption>,
    /// The custom emotes the status text uses.
    pub emojis: Vec<CustomEmoji>,
}

/// A link preview.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PreviewCard {
    /// The linked URL.
    pub url: String,
    /// The page title.
    pub title: String,
    /// The page description.
    pub description: String,
    /// `link`.
    #[serde(rename = "type")]
    pub kind: &'static str,
    /// The page's author.
    pub author_name: String,
    /// The author's page.
    pub author_url: String,
    /// The site name.
    pub provider_name: String,
    /// The site's address.
    pub provider_url: String,
    /// Embed HTML; empty for a link.
    pub html: String,
    /// Embed width.
    pub width: i32,
    /// Embed height.
    pub height: i32,
    /// The preview image.
    pub image: Option<String>,
    /// Embed URL; empty for a link.
    pub embed_url: String,
    /// A blurhash; never set.
    pub blurhash: Option<String>,
}

/// The app a status was posted with.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct StatusApplication {
    /// The app's name.
    pub name: String,
    /// The app's website.
    pub website: Option<String>,
}

/// A post, or a boost of one (then `reblog` holds the post).
#[allow(
    clippy::struct_excessive_bools,
    reason = "wire format: one JSON flag per fact"
)]
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Status {
    /// The post id; for a boost, the repost entry id.
    pub id: Uuid,
    /// The ActivityPub id.
    pub uri: String,
    /// When it was posted; for a boost, when it was boosted.
    #[serde(serialize_with = "serialize_time")]
    pub created_at: OffsetDateTime,
    /// The author; for a boost, the booster.
    pub account: Account,
    /// The text as HTML; empty for a boost.
    pub content: String,
    /// Who sees it.
    pub visibility: StatusVisibility,
    /// Media is sensitive, or the status has a content warning.
    pub sensitive: bool,
    /// The content warning.
    pub spoiler_text: String,
    /// Attached media.
    pub media_attachments: Vec<MediaAttachment>,
    /// The app that posted it; never set.
    pub application: Option<StatusApplication>,
    /// Mentioned accounts.
    pub mentions: Vec<Mention>,
    /// Hashtags.
    pub tags: Vec<Tag>,
    /// The custom emotes the text and content warning use.
    pub emojis: Vec<CustomEmoji>,
    /// Boosts.
    pub reblogs_count: i32,
    /// Likes.
    pub favourites_count: i32,
    /// Replies.
    pub replies_count: i32,
    /// The page of the status; `null` for a boost.
    pub url: Option<String>,
    /// The status this one answers.
    pub in_reply_to_id: Option<Uuid>,
    /// The author of the status this one answers.
    pub in_reply_to_account_id: Option<Uuid>,
    /// The boosted status.
    pub reblog: Option<Box<Status>>,
    /// The poll.
    pub poll: Option<Poll>,
    /// The link preview.
    pub card: Option<PreviewCard>,
    /// The language; never set.
    pub language: Option<String>,
    /// The raw text; only the answer of `DELETE /statuses/{id}` has it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// When it was last edited.
    #[serde(serialize_with = "serialize_time_option")]
    pub edited_at: Option<OffsetDateTime>,
    /// The viewer liked it.
    pub favourited: bool,
    /// The viewer boosted it.
    pub reblogged: bool,
    /// The viewer muted the conversation; never set.
    pub muted: bool,
    /// The viewer bookmarked it.
    pub bookmarked: bool,
    /// Pinned on its author's profile.
    pub pinned: bool,
}

/// A status with its conversation: `GET /statuses/{id}/context`.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Context {
    /// The statuses above it, oldest first.
    pub ancestors: Vec<Status>,
    /// The replies below it.
    pub descendants: Vec<Status>,
}

// ------------------------------------------------------------------ the rest

/// A notification.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Notification {
    /// The notification id.
    pub id: Uuid,
    /// `mention`, `reblog`, `favourite`, `follow`, `follow_request` or `poll`.
    #[serde(rename = "type")]
    pub kind: &'static str,
    /// When it happened.
    #[serde(serialize_with = "serialize_time")]
    pub created_at: OffsetDateTime,
    /// Who did it.
    pub account: Account,
    /// The status it is about.
    pub status: Option<Status>,
}

/// A list of accounts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct List {
    /// The list id.
    pub id: Uuid,
    /// The name.
    pub title: String,
    /// Which replies show: `list`, `followed` or `none`.
    pub replies_policy: &'static str,
    /// Members are hidden from the home timeline; never here.
    pub exclusive: bool,
}

/// A direct-message conversation.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Conversation {
    /// The id of the other account, which names the conversation.
    pub id: Uuid,
    /// Unread messages.
    pub unread: bool,
    /// The other participants.
    pub accounts: Vec<Account>,
    /// The newest message as a direct status.
    pub last_status: Option<Status>,
}

/// `GET /api/v2/search`.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct SearchResults {
    /// Matching accounts.
    pub accounts: Vec<Account>,
    /// Matching statuses.
    pub statuses: Vec<Status>,
    /// Matching hashtags.
    pub hashtags: Vec<Tag>,
}

/// `GET /api/v1/preferences`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Preferences {
    /// Default visibility of new posts.
    #[serde(rename = "posting:default:visibility")]
    pub default_visibility: StatusVisibility,
    /// Default sensitivity of new posts.
    #[serde(rename = "posting:default:sensitive")]
    pub default_sensitive: bool,
    /// Default language of new posts.
    #[serde(rename = "posting:default:language")]
    pub default_language: Option<String>,
    /// `default`, `show_all` or `hide_all`.
    #[serde(rename = "reading:expand:media")]
    pub expand_media: &'static str,
    /// Always open content warnings.
    #[serde(rename = "reading:expand:spoilers")]
    pub expand_spoilers: bool,
}

/// An account on `GET /api/v1/mutes`: the account and when the mute ends.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct MutedAccount {
    /// The muted account.
    #[serde(flatten)]
    pub account: Account,
    /// Always `null`: a tuita mute does not expire.
    pub mute_expires_at: Option<String>,
}

/// One entry of `GET /api/v1/accounts/familiar_followers`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FamiliarFollowers {
    /// The account that was asked about.
    pub id: Uuid,
    /// Followers of it whom the viewer follows too.
    pub accounts: Vec<Account>,
}

/// One entry of `GET /api/v2/suggestions`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Suggestion {
    /// Why the account is suggested, in the legacy words: `staff`, `past_interactions`, `global`.
    pub source: &'static str,
    /// Why, in the current words.
    pub sources: Vec<&'static str>,
    /// The suggested account.
    pub account: Account,
}

/// A reaction to an announcement. Announcements here have none; the type gives the array its
/// element type.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AnnouncementReaction {
    /// The emoji.
    pub name: String,
    /// How many accounts reacted.
    pub count: i64,
    /// Whether the viewer reacted.
    pub me: bool,
}

/// An instance announcement.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Announcement {
    /// The announcement id.
    pub id: Uuid,
    /// The text as HTML.
    pub content: String,
    /// Always `null`: an announcement shows from the moment it is put up.
    pub starts_at: Option<String>,
    /// `null`: the public announcement card does not carry the expiry.
    pub ends_at: Option<String>,
    /// Always true: only shown announcements are listed.
    pub published: bool,
    /// Always false.
    pub all_day: bool,
    /// When it was put up.
    #[serde(serialize_with = "serialize_time")]
    pub published_at: OffsetDateTime,
    /// When it was put up: announcements cannot be edited.
    #[serde(serialize_with = "serialize_time")]
    pub updated_at: OffsetDateTime,
    /// Always false: tuita does not record who read what.
    pub read: bool,
    /// Always empty.
    pub mentions: Vec<Mention>,
    /// Always empty.
    pub statuses: Vec<Status>,
    /// Always empty.
    pub tags: Vec<Tag>,
    /// Always empty.
    pub emojis: Vec<CustomEmoji>,
    /// Always empty.
    pub reactions: Vec<AnnouncementReaction>,
}

/// A word of a v2 filter.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FilterKeyword {
    /// The keyword id.
    pub id: String,
    /// The word or phrase.
    pub keyword: String,
    /// Match whole words only; tuita matches phrases anywhere.
    pub whole_word: bool,
}

/// A status a v2 filter names. Tuita filters name none; the type gives the array its element type.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FilterStatus {
    /// The entry id.
    pub id: String,
    /// The status.
    pub status_id: Uuid,
}

/// A v2 filter.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Filter {
    /// The filter id.
    pub id: String,
    /// The name.
    pub title: String,
    /// Where it applies.
    pub context: [&'static str; 5],
    /// Always `null`.
    pub expires_at: Option<String>,
    /// `hide`: the server drops the posts.
    pub filter_action: &'static str,
    /// The words.
    pub keywords: Vec<FilterKeyword>,
    /// Always empty.
    pub statuses: Vec<FilterStatus>,
}

/// A v1 filter: one phrase.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FilterV1 {
    /// The filter id.
    pub id: String,
    /// The word or phrase.
    pub phrase: String,
    /// Where it applies.
    pub context: [&'static str; 5],
    /// Always `null`.
    pub expires_at: Option<String>,
    /// True: the server drops the posts, the client cannot bring them back.
    pub irreversible: bool,
    /// Match whole words only; tuita matches phrases anywhere.
    pub whole_word: bool,
}

/// One group of `GET /api/v2/notifications`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct NotificationGroup {
    /// Opaque; `ungrouped-` and the notification id: tuita does not group.
    pub group_key: String,
    /// Notifications in the group; always 1.
    pub notifications_count: u32,
    /// The notification type.
    #[serde(rename = "type")]
    pub kind: &'static str,
    /// The newest notification of the group.
    pub most_recent_notification_id: Uuid,
    /// The oldest notification of the group.
    pub page_min_id: Uuid,
    /// The newest notification of the group.
    pub page_max_id: Uuid,
    /// When the newest one happened.
    #[serde(serialize_with = "serialize_time")]
    pub latest_page_notification_at: OffsetDateTime,
    /// Who did it.
    pub sample_account_ids: Vec<Uuid>,
    /// The status it is about.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_id: Option<Uuid>,
}

/// `GET /api/v2/notifications`.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct GroupedNotifications {
    /// The accounts the groups name.
    pub accounts: Vec<Account>,
    /// The statuses the groups name.
    pub statuses: Vec<Status>,
    /// The groups, newest first.
    pub notification_groups: Vec<NotificationGroup>,
}

// ------------------------------------------------------------------ OAuth

/// An app as `GET /api/v1/apps/verify_credentials` shows it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Application {
    /// The app id.
    pub id: Uuid,
    /// The name.
    pub name: String,
    /// The website.
    pub website: Option<String>,
    /// The scopes it registered for.
    pub scopes: Vec<String>,
    /// The redirect URIs it registered.
    pub redirect_uris: Vec<String>,
    /// The redirect URIs joined by newlines (the deprecated single-URI field).
    pub redirect_uri: String,
    /// Always empty: there is no Web Push.
    pub vapid_key: String,
}

/// The answer of `POST /api/v1/apps`: the app and its secrets. The only time the secret is shown.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CredentialApplication {
    /// The app.
    #[serde(flatten)]
    pub application: Application,
    /// The public client id.
    pub client_id: String,
    /// The client secret.
    pub client_secret: String,
    /// Secrets do not expire.
    pub client_secret_expires_at: i64,
}

/// The answer of `POST /oauth/token`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Token {
    /// The secret.
    pub access_token: String,
    /// `Bearer`.
    pub token_type: &'static str,
    /// The granted scopes, joined by spaces.
    pub scope: String,
    /// When it was made, as Unix seconds.
    pub created_at: i64,
}

// ------------------------------------------------------------------ instance

/// A house rule.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Rule {
    /// The rule number as a string.
    pub id: String,
    /// The rule.
    pub text: String,
    /// More words; empty.
    pub hint: String,
}

/// Limits on statuses.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct StatusesConfig {
    /// Characters in a status.
    pub max_characters: u32,
    /// Media in a status.
    pub max_media_attachments: u32,
    /// Characters a link counts.
    pub characters_reserved_per_url: u32,
}

/// Limits on uploads.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct MediaConfig {
    /// MIME types the server takes.
    pub supported_mime_types: Vec<String>,
    /// Longest description.
    pub description_limit: u32,
    /// Largest image in bytes.
    pub image_size_limit: u64,
    /// Largest image in pixels.
    pub image_matrix_limit: u64,
    /// Largest video in bytes.
    pub video_size_limit: u64,
    /// Highest video frame rate.
    pub video_frame_rate_limit: u32,
    /// Largest video in pixels.
    pub video_matrix_limit: u64,
}

/// Limits on polls.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PollsConfig {
    /// Answers in a poll.
    pub max_options: u32,
    /// Characters in an answer.
    pub max_characters_per_option: u32,
    /// Shortest poll in seconds.
    pub min_expiration: u32,
    /// Longest poll in seconds.
    pub max_expiration: u32,
}

/// `configuration` of `GET /api/v1/instance`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct InstanceConfigurationV1 {
    /// Status limits.
    pub statuses: StatusesConfig,
    /// Upload limits.
    pub media_attachments: MediaConfig,
    /// Poll limits.
    pub polls: PollsConfig,
}

/// URLs of `GET /api/v1/instance`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct InstanceUrlsV1 {
    /// The websocket origin of the streaming API.
    pub streaming_api: String,
}

/// Totals of `GET /api/v1/instance`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct InstanceStatsV1 {
    /// Local accounts.
    pub user_count: i64,
    /// Local posts.
    pub status_count: i64,
    /// Other instances this one knows.
    pub domain_count: i64,
}

/// `GET /api/v1/instance`, the Mastodon fields.
#[allow(
    clippy::struct_excessive_bools,
    reason = "wire format: one JSON flag per fact"
)]
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct InstanceV1 {
    /// The domain.
    pub uri: String,
    /// The instance name.
    pub title: String,
    /// A short description.
    pub short_description: String,
    /// The description.
    pub description: String,
    /// The contact address.
    pub email: String,
    /// The server version, in the form Mastodon clients parse.
    pub version: String,
    /// URLs.
    pub urls: InstanceUrlsV1,
    /// Totals.
    pub stats: InstanceStatsV1,
    /// The instance image.
    pub thumbnail: Option<String>,
    /// Languages.
    pub languages: Vec<String>,
    /// Sign-ups are open.
    pub registrations: bool,
    /// Sign-ups need approval.
    pub approval_required: bool,
    /// Anyone may invite.
    pub invites_enabled: bool,
    /// Limits.
    pub configuration: InstanceConfigurationV1,
    /// The contact account.
    pub contact_account: Option<Account>,
    /// The house rules.
    pub rules: Vec<Rule>,
}

/// `usage` of `GET /api/v2/instance`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct InstanceUsage {
    /// Accounts.
    pub users: InstanceUsers,
}

/// Accounts of `usage`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct InstanceUsers {
    /// Accounts active in the last month.
    pub active_month: i64,
}

/// The image of `GET /api/v2/instance`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct InstanceThumbnail {
    /// The image.
    pub url: String,
}

/// URLs of `configuration` in `GET /api/v2/instance`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct InstanceUrlsV2 {
    /// The websocket origin of the streaming API.
    pub streaming: String,
}

/// Limits on accounts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct AccountsConfig {
    /// Featured hashtags; none here.
    pub max_featured_tags: u32,
    /// Pinned statuses.
    pub max_pinned_statuses: u32,
}

/// Translation support.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct TranslationConfig {
    /// Whether the server translates; never.
    pub enabled: bool,
}

/// `configuration` of `GET /api/v2/instance`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct InstanceConfigurationV2 {
    /// URLs.
    pub urls: InstanceUrlsV2,
    /// Account limits.
    pub accounts: AccountsConfig,
    /// Status limits.
    pub statuses: StatusesConfig,
    /// Upload limits.
    pub media_attachments: MediaConfig,
    /// Poll limits.
    pub polls: PollsConfig,
    /// Translation.
    pub translation: TranslationConfig,
}

/// `registrations` of `GET /api/v2/instance`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RegistrationsV2 {
    /// Sign-ups are open.
    pub enabled: bool,
    /// Sign-ups need approval.
    pub approval_required: bool,
    /// A message for people who cannot sign up.
    pub message: Option<String>,
    /// Where to sign up.
    pub url: Option<String>,
}

/// `contact` of `GET /api/v2/instance`.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ContactV2 {
    /// The contact address.
    pub email: String,
    /// The contact account.
    pub account: Option<Account>,
}

/// `GET /api/v2/instance`.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct InstanceV2 {
    /// The domain.
    pub domain: String,
    /// The instance name.
    pub title: String,
    /// The server version, in the form Mastodon clients parse.
    pub version: String,
    /// Where the running source is.
    pub source_url: String,
    /// The description.
    pub description: String,
    /// Usage.
    pub usage: InstanceUsage,
    /// The instance image.
    pub thumbnail: InstanceThumbnail,
    /// Languages.
    pub languages: Vec<String>,
    /// Limits and URLs.
    pub configuration: InstanceConfigurationV2,
    /// Sign-ups.
    pub registrations: RegistrationsV2,
    /// Contact.
    pub contact: ContactV2,
    /// The house rules.
    pub rules: Vec<Rule>,
}

// ================================================================== mappers

/// The account of a full profile card.
#[must_use]
pub fn account(user: &UserDto, ctx: &Ctx<'_>) -> Account {
    let config = ctx.config();
    let extra = ctx.extra(user.id);
    let (url, uri) = match &user.domain {
        None => (
            ids::profile_url(config, &user.handle),
            ids::actor_id(config, &user.handle),
        ),
        Some(_) => {
            let url = user.url.clone().or_else(|| extra.ap_id.clone());
            (
                url.clone().unwrap_or_default(),
                extra.ap_id.clone().or(url).unwrap_or_default(),
            )
        }
    };
    let avatar = user.avatar_url.as_deref().map_or_else(
        || format!("{}/icon-512.png", config.public_url),
        |avatar| absolute(config, avatar),
    );
    let header = user
        .banner_url
        .as_deref()
        .map_or_else(String::new, |banner| absolute(config, banner));
    Account {
        id: user.id,
        username: user.handle.clone(),
        acct: acct(&user.handle, user.domain.as_deref()),
        url,
        uri,
        display_name: user.display_name.clone(),
        note: content_html(config, &user.bio, &[]),
        avatar_static: avatar.clone(),
        avatar,
        header_static: header.clone(),
        header,
        locked: user.locked,
        fields: fields(user)
            .into_iter()
            .map(|(name, raw)| AccountField {
                value: field_html(&name, &raw),
                name,
                verified_at: None,
            })
            .collect(),
        emojis: custom_emojis(config, &user.emojis),
        bot: user.bot,
        group: false,
        discoverable: extra.discoverable,
        created_at: user.created_at,
        last_status_at: extra.last_status_at.map(date),
        statuses_count: i64::from(user.counts.posts),
        followers_count: i64::from(user.counts.followers),
        following_count: i64::from(user.counts.following),
    }
}

/// The account behind a small author card. The profile comes from the context; without it the
/// account is built from the card, with zero counts and the sign-up time read from the id.
#[must_use]
pub fn account_from_ref(card: &UserRefDto, ctx: &Ctx<'_>) -> Account {
    if let Some(user) = ctx.user(card.id) {
        return account(user, ctx);
    }
    let bare = UserDto {
        id: card.id,
        handle: card.handle.clone(),
        domain: card.domain.clone(),
        display_name: card.display_name.clone(),
        bio: String::new(),
        location: String::new(),
        website: String::new(),
        avatar_url: card.avatar_url.clone(),
        banner_url: None,
        avatar_shape: card.avatar_shape,
        ink: card.ink,
        verified: card.verified,
        staff: None,
        org: false,
        bot: card.bot,
        bot_owner: None,
        affiliation: None,
        pinned_post_id: None,
        created_at: uuid_time(card.id),
        counts: crate::dto::UserCounts {
            followers: 0,
            following: 0,
            posts: 0,
        },
        relation: Relation::default(),
        suspended: false,
        url: None,
        locked: false,
        emojis: card.emojis.clone(),
    };
    account(&bare, ctx)
}

/// The signed-in account with its private part. `follow_requests` is the number of pending
/// follow requests.
#[must_use]
pub fn credential_account(me: &MeDto, follow_requests: i64, ctx: &Ctx<'_>) -> CredentialAccount {
    let account = account(&me.user, ctx);
    let source_fields = fields(&me.user)
        .into_iter()
        .map(|(name, value)| SourceField {
            name,
            value,
            verified_at: None,
        })
        .collect();
    let role = match me.role {
        crate::dto::Role::Admin => ("1", "Admin", "1"),
        crate::dto::Role::Moderator => ("2", "Moderator", "2"),
        crate::dto::Role::User => ("-99", "", "65536"),
    };
    CredentialAccount {
        source: AccountSource {
            privacy: StatusVisibility::Public,
            sensitive: false,
            language: me.settings.language.clone(),
            note: me.user.bio.clone(),
            fields: source_fields,
            follow_requests_count: follow_requests,
            hide_collections: None,
            discoverable: Some(me.settings.discoverable),
            indexable: me.settings.discoverable,
        },
        role: AccountRole {
            id: role.0.to_owned(),
            name: role.1.to_owned(),
            permissions: role.2.to_owned(),
            color: String::new(),
            highlighted: me.role != crate::dto::Role::User,
        },
        account,
    }
}

/// `handle` for a local account, `handle@domain` for a remote one.
#[must_use]
pub fn acct(handle: &str, domain: Option<&str>) -> String {
    match domain {
        Some(domain) => format!("{handle}@{domain}"),
        None => handle.to_owned(),
    }
}

/// The profile fields Mastodon shows for tuita's website and location, as
/// `(label, raw value)`. Empty values are left out.
fn fields(user: &UserDto) -> Vec<(String, String)> {
    [("Website", &user.website), ("Location", &user.location)]
        .into_iter()
        .filter(|(_, value)| !value.trim().is_empty())
        .map(|(name, value)| (name.to_owned(), value.trim().to_owned()))
        .collect()
}

fn field_html(name: &str, raw: &str) -> String {
    if name == "Website" && (raw.starts_with("https://") || raw.starts_with("http://")) {
        let shown = raw
            .split_once("://")
            .map_or(raw, |(_, rest)| rest)
            .trim_end_matches('/');
        format!(
            "<a href=\"{}\" target=\"_blank\" rel=\"nofollow noopener noreferrer me\">{}</a>",
            escape_html(raw),
            escape_html(shown)
        )
    } else {
        escape_html(raw)
    }
}

/// How the viewer relates to the account `id`.
#[must_use]
pub fn relationship(id: Uuid, relation: Relation) -> Relationship {
    Relationship {
        id,
        following: relation.following,
        showing_reblogs: true,
        notifying: false,
        languages: None,
        followed_by: relation.follows_you,
        blocking: relation.blocked,
        blocked_by: relation.blocks_you,
        muting: relation.muted,
        muting_notifications: false,
        requested: relation.follow_pending,
        requested_by: false,
        domain_blocking: false,
        endorsed: false,
        note: String::new(),
    }
}

/// An upload or an attachment of a post.
#[must_use]
pub fn media_attachment(media: &MediaDto, ctx: &Ctx<'_>) -> MediaAttachment {
    let config = ctx.config();
    let url = absolute(config, &media.url);
    let preview_url = media
        .poster_url
        .as_deref()
        .map_or_else(|| url.clone(), |poster| absolute(config, poster));
    let size = match (media.width, media.height) {
        (Some(width), Some(height)) if width > 0 && height > 0 => Some(MediaSize {
            width,
            height,
            size: format!("{width}x{height}"),
            aspect: f64::from(width) / f64::from(height),
        }),
        _ => None,
    };
    MediaAttachment {
        id: media.id,
        kind: match media.kind {
            MediaKind::Image => "image",
            MediaKind::Video => "video",
        },
        url,
        preview_url,
        remote_url: None,
        text_url: None,
        meta: MediaMeta {
            small: size.clone(),
            original: size,
        },
        description: (!media.alt.is_empty()).then(|| media.alt.clone()),
        blurhash: None,
    }
}

/// The poll of post `post`; `emojis` are the emotes of the post's text, which Mastodon shows on
/// the poll too.
#[must_use]
pub fn poll(post: Uuid, poll: &PollDto, emojis: &[CustomEmoji]) -> Poll {
    Poll {
        id: post,
        expires_at: Some(poll.ends_at),
        expired: poll.closed,
        multiple: poll.multiple,
        votes_count: poll.total,
        voters_count: poll.multiple.then_some(poll.voters),
        voted: !poll.voted.is_empty(),
        own_votes: poll.voted.clone(),
        options: poll
            .options
            .iter()
            .map(|option| PollOption {
                title: option.text.clone(),
                votes_count: Some(option.votes),
            })
            .collect(),
        emojis: emojis.to_vec(),
    }
}

/// The preview card of a link.
#[must_use]
pub fn preview_card(card: &LinkCardDto, ctx: &Ctx<'_>) -> PreviewCard {
    PreviewCard {
        url: card.url.clone(),
        title: card.title.clone(),
        description: card.description.clone(),
        kind: "link",
        author_name: String::new(),
        author_url: String::new(),
        provider_name: card.site.clone(),
        provider_url: String::new(),
        html: String::new(),
        width: 0,
        height: 0,
        image: card
            .image
            .as_deref()
            .map(|image| absolute(ctx.config(), image)),
        embed_url: String::new(),
        blurhash: None,
    }
}

/// A hashtag with its page.
#[must_use]
pub fn tag(name: &str, config: &Config) -> Tag {
    Tag {
        name: name.to_owned(),
        url: format!("{}/tags/{name}", config.public_url),
        history: Vec::new(),
        following: None,
    }
}

fn mention(config: &Config, mentioned: &Mentioned) -> Mention {
    let url = match &mentioned.domain {
        None => ids::profile_url(config, &mentioned.handle),
        Some(_) => mentioned
            .url
            .clone()
            .or_else(|| mentioned.ap_id.clone())
            .unwrap_or_default(),
    };
    Mention {
        id: mentioned.id,
        username: mentioned.handle.clone(),
        url,
        acct: acct(&mentioned.handle, mentioned.domain.as_deref()),
    }
}

fn tags(post: &PostDto, config: &Config) -> Vec<Tag> {
    let mut seen: Vec<&str> = Vec::new();
    let mut out = Vec::new();
    for segment in &post.rich_text {
        if let SegmentDto::Hashtag { tag: name, .. } = segment
            && !seen.contains(&name.as_str())
        {
            seen.push(name);
            out.push(tag(name, config));
        }
    }
    out
}

/// A post as a Mastodon status.
///
/// A repost entry (`post.reposted_by` is set) becomes the boost wrapper Mastodon uses: the
/// status id is the entry id, `account` is the booster, `created_at` is the boost time read from
/// the entry id, and `reblog` holds the post itself.
///
/// The HTML is the one federation sends (`federation::render::content_html`): paragraphs,
/// `<br>`, hashtag links, mention links (`h-card`) for accounts in the context, and links. A
/// remote post has no ActivityPub id here, so its `uri` is its page URL.
#[must_use]
pub fn status(post: &PostDto, ctx: &Ctx<'_>) -> Status {
    let inner = plain_status(post, ctx);
    let Some(booster) = &post.reposted_by else {
        return inner;
    };
    let account = account_from_ref(booster, ctx);
    Status {
        id: post.entry_id,
        uri: format!("{}/statuses/{}/activity", account.uri, post.entry_id),
        created_at: uuid_time(post.entry_id),
        content: String::new(),
        visibility: inner.visibility,
        sensitive: false,
        spoiler_text: String::new(),
        media_attachments: Vec::new(),
        application: None,
        mentions: Vec::new(),
        tags: Vec::new(),
        emojis: Vec::new(),
        reblogs_count: inner.reblogs_count,
        favourites_count: inner.favourites_count,
        replies_count: inner.replies_count,
        url: None,
        in_reply_to_id: None,
        in_reply_to_account_id: None,
        poll: None,
        card: None,
        language: None,
        text: None,
        edited_at: None,
        favourited: inner.favourited,
        reblogged: inner.reblogged,
        muted: false,
        bookmarked: inner.bookmarked,
        pinned: false,
        reblog: Some(Box::new(inner)),
        account,
    }
}

/// Maps a page of posts.
#[must_use]
pub fn statuses(posts: &[PostDto], ctx: &Ctx<'_>) -> Vec<Status> {
    posts.iter().map(|post| status(post, ctx)).collect()
}

fn plain_status(post: &PostDto, ctx: &Ctx<'_>) -> Status {
    let config = ctx.config();
    let mentioned = ctx.mentions(post.id);
    let account = account_from_ref(&post.author, ctx);
    let emojis = custom_emojis(config, &post.emojis);
    let (uri, url) = match &post.author.domain {
        None => (
            ids::note_id(config, &post.author.handle, post.id),
            Some(ids::note_url(config, &post.author.handle, post.id)),
        ),
        Some(_) => {
            let page = post.url.clone();
            (page.clone().unwrap_or_default(), page)
        }
    };
    Status {
        id: post.id,
        uri,
        created_at: post.created_at,
        content: content_html(config, &post.text, mentioned),
        visibility: post.visibility.into(),
        sensitive: post.sensitive || !post.cw.is_empty(),
        spoiler_text: post.cw.clone(),
        media_attachments: post
            .media
            .iter()
            .map(|media| media_attachment(media, ctx))
            .collect(),
        application: None,
        mentions: mentioned
            .iter()
            .map(|mentioned| mention(config, mentioned))
            .collect(),
        tags: tags(post, config),
        emojis: emojis.clone(),
        reblogs_count: post.counts.reposts,
        favourites_count: post.counts.likes,
        replies_count: post.counts.replies,
        url,
        in_reply_to_id: post.reply_to.as_ref().map(|reply| reply.id),
        in_reply_to_account_id: post
            .reply_to
            .as_ref()
            .and_then(|reply| ctx.post_author(reply.id)),
        reblog: None,
        poll: post
            .poll
            .as_ref()
            .map(|value| poll(post.id, value, &emojis)),
        card: post.card.as_ref().map(|card| preview_card(card, ctx)),
        language: None,
        text: None,
        edited_at: post.edited_at,
        favourited: post.viewer.liked,
        reblogged: post.viewer.reposted,
        muted: false,
        bookmarked: post.viewer.bookmarked,
        pinned: post.pinned,
        account,
    }
}

/// The Mastodon type of a tuita notification, or `None` for a kind Mastodon has no word for
/// (affiliations): those are left out of the list. A reply and a quote are mentions.
#[must_use]
pub const fn notification_kind(kind: NotificationKindDto) -> Option<&'static str> {
    match kind {
        NotificationKindDto::Like => Some("favourite"),
        NotificationKindDto::Repost => Some("reblog"),
        NotificationKindDto::Reply | NotificationKindDto::Mention | NotificationKindDto::Quote => {
            Some("mention")
        }
        NotificationKindDto::Follow => Some("follow"),
        NotificationKindDto::FollowRequest => Some("follow_request"),
        NotificationKindDto::Poll => Some("poll"),
        NotificationKindDto::Affiliation => None,
    }
}

/// A notification. `post` is the post it is about; follows have none. `None` when the kind has no
/// Mastodon type, see [`notification_kind`].
#[must_use]
pub fn notification(
    id: Uuid,
    kind: NotificationKindDto,
    created_at: OffsetDateTime,
    actor: &UserRefDto,
    post: Option<&PostDto>,
    ctx: &Ctx<'_>,
) -> Option<Notification> {
    Some(Notification {
        id,
        kind: notification_kind(kind)?,
        created_at,
        account: account_from_ref(actor, ctx),
        status: post.map(|post| status(post, ctx)),
    })
}

/// A list of accounts.
#[must_use]
pub fn list(list: &ListDto) -> List {
    List {
        id: list.id,
        title: list.name.clone(),
        replies_policy: "list",
        exclusive: false,
    }
}

/// The reading and posting defaults of every account. Tuita keeps none per account: posts are
/// public, and nothing marks media or content warnings as expanded.
#[must_use]
pub const fn preferences() -> Preferences {
    Preferences {
        default_visibility: StatusVisibility::Public,
        default_sensitive: false,
        default_language: None,
        expand_media: "default",
        expand_spoilers: false,
    }
}

/// An account of `GET /api/v1/mutes`.
#[must_use]
pub const fn muted_account(account: Account) -> MutedAccount {
    MutedAccount {
        account,
        mute_expires_at: None,
    }
}

/// A suggested account: tuita ranks verified accounts, then the most followed.
#[must_use]
pub fn suggestion(account: Account) -> Suggestion {
    Suggestion {
        source: "global",
        sources: vec!["most_followed"],
        account,
    }
}

/// An announcement. The link, if any, is a second paragraph; the colour has no Mastodon field.
#[must_use]
pub fn announcement(card: &AnnouncementDto, config: &Config) -> Announcement {
    let mut content = format!("<p>{}</p>", escape_html(&card.text));
    if !card.link.is_empty() {
        let href = escape_html(&absolute(config, &card.link));
        content.push_str(&format!(
            "<p><a href=\"{href}\" target=\"_blank\" rel=\"nofollow noopener noreferrer\">{href}</a></p>"
        ));
    }
    Announcement {
        id: card.id,
        content,
        starts_at: None,
        ends_at: None,
        published: true,
        all_day: false,
        published_at: card.created_at,
        updated_at: card.created_at,
        read: false,
        mentions: Vec::new(),
        statuses: Vec::new(),
        tags: Vec::new(),
        emojis: Vec::new(),
        reactions: Vec::new(),
    }
}

/// Where a muted word applies: everywhere a post shows.
const FILTER_CONTEXTS: [&str; 5] = ["home", "notifications", "public", "thread", "account"];

/// The v2 filters for a viewer's muted words: one filter that holds every word, none when the
/// viewer muted nothing.
#[must_use]
pub fn filters(words: &[String]) -> Vec<Filter> {
    if words.is_empty() {
        return Vec::new();
    }
    vec![Filter {
        id: "1".to_owned(),
        title: "Muted words".to_owned(),
        context: FILTER_CONTEXTS,
        expires_at: None,
        filter_action: "hide",
        keywords: words
            .iter()
            .enumerate()
            .map(|(index, word)| FilterKeyword {
                id: (index + 1).to_string(),
                keyword: word.clone(),
                whole_word: false,
            })
            .collect(),
        statuses: Vec::new(),
    }]
}

/// The v1 filters for a viewer's muted words: one per word.
#[must_use]
pub fn filters_v1(words: &[String]) -> Vec<FilterV1> {
    words
        .iter()
        .enumerate()
        .map(|(index, word)| FilterV1 {
            id: (index + 1).to_string(),
            phrase: word.clone(),
            context: FILTER_CONTEXTS,
            expires_at: None,
            irreversible: true,
            whole_word: false,
        })
        .collect()
}

/// A trending tag with its use over the last days, newest day first.
#[must_use]
pub fn trending_tag(name: &str, days: &[DayUse], config: &Config) -> Tag {
    let mut tag = tag(name, config);
    tag.history = days
        .iter()
        .map(|day| TagHistory {
            day: day.day.to_string(),
            uses: day.uses.to_string(),
            accounts: day.accounts.to_string(),
        })
        .collect();
    tag.following = Some(false);
    tag
}

/// Notifications as the grouped list of `GET /api/v2/notifications`: every notification is a
/// group of its own, so clients that merge groups themselves still can.
#[must_use]
pub fn grouped_notifications(notifications: Vec<Notification>) -> GroupedNotifications {
    let mut accounts: Vec<Account> = Vec::new();
    let mut statuses: Vec<Status> = Vec::new();
    let mut groups = Vec::with_capacity(notifications.len());
    for notification in notifications {
        groups.push(NotificationGroup {
            group_key: format!("ungrouped-{}", notification.id),
            notifications_count: 1,
            kind: notification.kind,
            most_recent_notification_id: notification.id,
            page_min_id: notification.id,
            page_max_id: notification.id,
            latest_page_notification_at: notification.created_at,
            sample_account_ids: vec![notification.account.id],
            status_id: notification.status.as_ref().map(|status| status.id),
        });
        if !accounts
            .iter()
            .any(|seen| seen.id == notification.account.id)
        {
            accounts.push(notification.account);
        }
        if let Some(status) = notification.status
            && !statuses.iter().any(|seen| seen.id == status.id)
        {
            statuses.push(status);
        }
    }
    GroupedNotifications {
        accounts,
        statuses,
        notification_groups: groups,
    }
}

/// The content of a sealed message: the server cannot read it, and an OAuth client has no key.
const SEALED_CONTENT: &str = "<p>Encrypted message. Open tuita to read it.</p>";

/// A direct message as the direct status Mastodon clients show in a conversation. Its id is the
/// message id; `sender` and `recipient` are the two people. A sealed message shows
/// a placeholder instead of its text.
#[must_use]
pub fn message_status(
    message: &MessageDto,
    sender: &UserRefDto,
    recipient: &UserRefDto,
    ctx: &Ctx<'_>,
) -> Status {
    let config = ctx.config();
    let account = account_from_ref(sender, ctx);
    let target = account_from_ref(recipient, ctx);
    Status {
        id: message.id,
        uri: format!("{}/messages/{}", account.uri, message.id),
        created_at: message.created_at,
        content: if message.ciphertext.is_some() {
            SEALED_CONTENT.to_owned()
        } else {
            content_html(config, &message.text, &[])
        },
        visibility: StatusVisibility::Direct,
        sensitive: false,
        spoiler_text: String::new(),
        media_attachments: Vec::new(),
        application: None,
        mentions: vec![Mention {
            id: target.id,
            username: target.username,
            url: target.url,
            acct: target.acct,
        }],
        tags: Vec::new(),
        emojis: custom_emojis(config, &message.emojis),
        reblogs_count: 0,
        favourites_count: 0,
        replies_count: 0,
        url: None,
        in_reply_to_id: None,
        in_reply_to_account_id: None,
        reblog: None,
        poll: None,
        card: None,
        language: None,
        text: None,
        edited_at: None,
        favourited: false,
        reblogged: false,
        muted: false,
        bookmarked: false,
        pinned: false,
        account,
    }
}

/// A direct-message conversation. `viewer` is the signed-in account: the newest message's
/// sender or recipient, depending on who wrote it.
#[must_use]
pub fn conversation(
    conversation: &ConversationDto,
    viewer: &UserRefDto,
    ctx: &Ctx<'_>,
) -> Conversation {
    let (sender, recipient) = if conversation.last.mine {
        (viewer, &conversation.user)
    } else {
        (&conversation.user, viewer)
    };
    Conversation {
        id: conversation.user.id,
        unread: conversation.unread > 0,
        accounts: vec![account_from_ref(&conversation.user, ctx)],
        last_status: Some(message_status(&conversation.last, sender, recipient, ctx)),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::dto::{
        AvatarShape, Ink, MediaKind, PollOptionDto, PostCounts, PostViewer, QuoteDto, ReplyPolicy,
        ReplyToDto, UserCounts,
    };
    use serde_json::{Value, json};
    fn t(text: &str) -> OffsetDateTime {
        tuita_core::ap::parse_time(text).expect("valid timestamp")
    }

    fn config() -> Config {
        Config::from_lookup(|var| match var {
            "DATABASE_URL" => Some("postgres://unused".to_owned()),
            "PUBLIC_URL" => Some("https://social.example".to_owned()),
            _ => None,
        })
        .expect("config")
    }

    const ADA: Uuid = Uuid::from_u128(0x0198_aaaa_0000_7000_8000_0000_0000_0001);
    const BOB: Uuid = Uuid::from_u128(0x0198_aaaa_0000_7000_8000_0000_0000_0002);
    const POST: Uuid = Uuid::from_u128(0x0198_aaaa_0000_7000_8000_0000_0000_0010);
    const PARENT: Uuid = Uuid::from_u128(0x0198_aaaa_0000_7000_8000_0000_0000_0011);
    const ENTRY: Uuid = Uuid::from_u128(0x0198_bbbb_0000_7000_8000_0000_0000_0012);

    fn card(id: Uuid, handle: &str, domain: Option<&str>) -> UserRefDto {
        UserRefDto {
            id,
            handle: handle.to_owned(),
            domain: domain.map(str::to_owned),
            display_name: format!("Name of {handle}"),
            avatar_url: Some("/media/0198aaaa-0000-7000-8000-0000000000aa".to_owned()),
            avatar_shape: AvatarShape::Circle,
            ink: Ink::Bone,
            verified: false,
            bot: false,
            emojis: Vec::new(),
        }
    }

    fn user(id: Uuid, handle: &str, domain: Option<&str>) -> UserDto {
        UserDto {
            id,
            handle: handle.to_owned(),
            domain: domain.map(str::to_owned),
            display_name: format!("Name of {handle}"),
            bio: "I write #rust".to_owned(),
            location: "Lisbon".to_owned(),
            website: "https://ada.example/".to_owned(),
            avatar_url: Some("/media/0198aaaa-0000-7000-8000-0000000000aa".to_owned()),
            banner_url: Some("/media/0198aaaa-0000-7000-8000-0000000000bb".to_owned()),
            avatar_shape: AvatarShape::Circle,
            ink: Ink::Bone,
            verified: false,
            staff: None,
            org: false,
            bot: false,
            bot_owner: None,
            affiliation: None,
            pinned_post_id: None,
            created_at: t("2025-01-02T03:04:05Z"),
            counts: UserCounts {
                followers: 7,
                following: 3,
                posts: 12,
            },
            relation: Relation::default(),
            suspended: false,
            url: domain.map(|d| format!("https://{d}/@{handle}")),
            locked: domain.is_some(),
            emojis: Vec::new(),
        }
    }

    fn post(text: &str) -> PostDto {
        PostDto {
            id: POST,
            author: card(ADA, "ada", None),
            text: text.to_owned(),
            rich_text: crate::db::posts::segments(text),
            cw: String::new(),
            sensitive: false,
            created_at: t("2026-11-09T10:00:00.123456Z"),
            edited_at: None,
            reply_policy: ReplyPolicy::Everyone,
            visibility: Visibility::Public,
            url: None,
            client: None,
            counts: PostCounts {
                likes: 3,
                reposts: 2,
                replies: 1,
                quotes: 0,
                views: 9,
            },
            viewer: PostViewer::default(),
            can_reply: true,
            reply_to: None,
            quote: None,
            media: Vec::new(),
            card: None,
            poll: None,
            hidden_reply: false,
            pinned: false,
            entry_id: POST,
            reposted_by: None,
            emojis: Vec::new(),
        }
    }

    fn json_of<T: Serialize>(value: &T) -> Value {
        serde_json::to_value(value).expect("serializes")
    }

    #[test]
    fn a_local_account_has_every_field_clients_read() {
        let config = config();
        let ctx = Ctx::new(&config).with_extra(
            ADA,
            crate::mastodon::ctx::AccountExtra {
                ap_id: None,
                last_status_at: Some(t("2026-11-08T23:00:00Z")),
                discoverable: Some(true),
            },
        );
        let value = json_of(&account(&user(ADA, "ada", None), &ctx));
        assert_eq!(
            value,
            json!({
                "id": ADA.to_string(),
                "username": "ada",
                "acct": "ada",
                "url": "https://social.example/@ada",
                "uri": "https://social.example/users/ada",
                "display_name": "Name of ada",
                "note": "<p>I write <a href=\"https://social.example/tags/rust\" class=\"mention hashtag\" rel=\"tag nofollow noopener noreferrer\">#<span>rust</span></a></p>",
                "avatar": "https://social.example/media/0198aaaa-0000-7000-8000-0000000000aa",
                "avatar_static": "https://social.example/media/0198aaaa-0000-7000-8000-0000000000aa",
                "header": "https://social.example/media/0198aaaa-0000-7000-8000-0000000000bb",
                "header_static": "https://social.example/media/0198aaaa-0000-7000-8000-0000000000bb",
                "locked": false,
                "fields": [
                    {"name": "Website", "value": "<a href=\"https://ada.example/\" target=\"_blank\" rel=\"nofollow noopener noreferrer me\">ada.example</a>", "verified_at": null},
                    {"name": "Location", "value": "Lisbon", "verified_at": null},
                ],
                "emojis": [],
                "bot": false,
                "group": false,
                "discoverable": true,
                "created_at": "2025-01-02T03:04:05.000Z",
                "last_status_at": "2026-11-08",
                "statuses_count": 12,
                "followers_count": 7,
                "following_count": 3,
            })
        );
    }

    #[test]
    fn a_remote_account_is_user_at_domain_with_its_own_urls() {
        let config = config();
        let ctx = Ctx::new(&config).with_extra(
            BOB,
            crate::mastodon::ctx::AccountExtra {
                ap_id: Some("https://far.example/users/bob".to_owned()),
                ..Default::default()
            },
        );
        let value = json_of(&account(&user(BOB, "bob", Some("far.example")), &ctx));
        assert_eq!(value["acct"], "bob@far.example");
        assert_eq!(value["username"], "bob");
        assert_eq!(value["url"], "https://far.example/@bob");
        assert_eq!(value["uri"], "https://far.example/users/bob");
        assert_eq!(value["locked"], true);
        assert_eq!(value["last_status_at"], Value::Null);
    }

    #[test]
    fn an_account_without_images_gets_the_site_icon_and_an_empty_header() {
        let config = config();
        let mut bare = user(ADA, "ada", None);
        bare.avatar_url = None;
        bare.banner_url = None;
        bare.bio = String::new();
        let value = json_of(&account(&bare, &Ctx::new(&config)));
        assert_eq!(value["avatar"], "https://social.example/icon-512.png");
        assert_eq!(value["header"], "");
        assert_eq!(value["note"], "");
    }

    #[test]
    fn a_card_without_a_profile_in_the_context_still_maps() {
        let config = config();
        let value = json_of(&account_from_ref(
            &card(ADA, "ada", None),
            &Ctx::new(&config),
        ));
        assert_eq!(value["acct"], "ada");
        assert_eq!(value["statuses_count"], 0);
        assert_eq!(value["created_at"], iso(uuid_time(ADA)));
    }

    #[test]
    fn a_status_has_html_mentions_tags_counts_and_viewer_flags() {
        let config = config();
        let ctx = Ctx::new(&config).with_mentions(
            POST,
            vec![Mentioned {
                id: BOB,
                handle: "bob".to_owned(),
                domain: Some("far.example".to_owned()),
                ap_id: Some("https://far.example/users/bob".to_owned()),
                url: Some("https://far.example/@bob".to_owned()),
            }],
        );
        let mut post =
            post("hello @bob@far.example #Rust and #rust\nsecond line https://x.example/a");
        post.viewer = PostViewer {
            liked: true,
            reposted: false,
            bookmarked: true,
        };
        post.cw = "spoiler".to_owned();
        post.reply_to = Some(ReplyToDto {
            id: PARENT,
            handle: Some("bob".to_owned()),
        });
        post.pinned = true;
        let ctx = ctx.with_post_author(PARENT, BOB);
        let value = json_of(&status(&post, &ctx));
        assert_eq!(value["id"], POST.to_string());
        assert_eq!(
            value["uri"],
            format!("https://social.example/users/ada/statuses/{POST}")
        );
        assert_eq!(value["url"], format!("https://social.example/@ada/{POST}"));
        assert_eq!(value["created_at"], "2026-11-09T10:00:00.123Z");
        assert_eq!(value["visibility"], "public");
        assert_eq!(value["sensitive"], true);
        assert_eq!(value["spoiler_text"], "spoiler");
        assert_eq!(value["in_reply_to_id"], PARENT.to_string());
        assert_eq!(value["in_reply_to_account_id"], BOB.to_string());
        assert_eq!(value["reblog"], Value::Null);
        assert_eq!(value["favourites_count"], 3);
        assert_eq!(value["reblogs_count"], 2);
        assert_eq!(value["replies_count"], 1);
        assert_eq!(value["favourited"], true);
        assert_eq!(value["reblogged"], false);
        assert_eq!(value["bookmarked"], true);
        assert_eq!(value["pinned"], true);
        let content = value["content"].as_str().expect("html");
        assert!(content.starts_with("<p>hello <span class=\"h-card\"><a href=\"https://far.example/@bob\" class=\"u-url mention\""), "{content}");
        assert!(content.contains("class=\"mention hashtag\""), "{content}");
        assert!(content.contains("<br>second line"), "{content}");
        assert!(
            content.contains("href=\"https://x.example/a\""),
            "{content}"
        );
        assert_eq!(
            value["mentions"],
            json!([{
                "id": BOB.to_string(),
                "username": "bob",
                "url": "https://far.example/@bob",
                "acct": "bob@far.example",
            }])
        );
        assert_eq!(
            value["tags"],
            json!([{"name": "rust", "url": "https://social.example/tags/rust"}])
        );
        for key in ["media_attachments", "emojis"] {
            assert_eq!(value[key], json!([]), "{key}");
        }
        assert_eq!(value["application"], Value::Null);
        assert_eq!(value["poll"], Value::Null);
        assert_eq!(value["card"], Value::Null);
    }

    #[test]
    fn visibility_maps_both_ways() {
        let config = config();
        for (tuita, mastodon) in [
            (Visibility::Public, "public"),
            (Visibility::Unlisted, "unlisted"),
            (Visibility::Followers, "private"),
            (Visibility::Direct, "direct"),
        ] {
            let mut post = post("x");
            post.visibility = tuita;
            let value = json_of(&status(&post, &Ctx::new(&config)));
            assert_eq!(value["visibility"], mastodon);
            assert_eq!(StatusVisibility::from(tuita).to_tuita(), tuita);
        }
    }

    #[test]
    fn media_types_sizes_and_descriptions() {
        let config = config();
        let mut post = post("pics");
        post.media = vec![
            MediaDto {
                id: Uuid::from_u128(1),
                kind: MediaKind::Image,
                url: "/media/a.webp".to_owned(),
                poster_url: None,
                alt: "a cat".to_owned(),
                width: Some(800),
                height: Some(400),
            },
            MediaDto {
                id: Uuid::from_u128(2),
                kind: MediaKind::Video,
                url: "/media/b.mp4".to_owned(),
                poster_url: Some("/media/b.jpg".to_owned()),
                alt: String::new(),
                width: None,
                height: None,
            },
        ];
        let value = json_of(&status(&post, &Ctx::new(&config)));
        let image = &value["media_attachments"][0];
        assert_eq!(image["type"], "image");
        assert_eq!(image["url"], "https://social.example/media/a.webp");
        assert_eq!(image["preview_url"], "https://social.example/media/a.webp");
        assert_eq!(image["description"], "a cat");
        assert_eq!(
            image["meta"]["original"],
            json!({"width": 800, "height": 400, "size": "800x400", "aspect": 2.0})
        );
        assert_eq!(image["meta"]["small"], image["meta"]["original"]);
        assert_eq!(image["remote_url"], Value::Null);
        let video = &value["media_attachments"][1];
        assert_eq!(video["type"], "video");
        assert_eq!(video["preview_url"], "https://social.example/media/b.jpg");
        assert_eq!(video["description"], Value::Null);
        assert_eq!(video["meta"], json!({}));
    }

    #[test]
    fn a_poll_keeps_the_post_id_and_the_viewers_votes() {
        let config = config();
        let mut post = post("which?");
        post.poll = Some(PollDto {
            options: vec![
                PollOptionDto {
                    text: "a".to_owned(),
                    votes: 2,
                },
                PollOptionDto {
                    text: "b".to_owned(),
                    votes: 1,
                },
            ],
            total: 3,
            voters: 3,
            multiple: false,
            ends_at: t("2026-11-10T10:00:00Z"),
            closed: false,
            voted: vec![1],
        });
        let value = json_of(&status(&post, &Ctx::new(&config)));
        assert_eq!(
            value["poll"],
            json!({
                "id": POST.to_string(),
                "expires_at": "2026-11-10T10:00:00.000Z",
                "expired": false,
                "multiple": false,
                "votes_count": 3,
                "voters_count": null,
                "voted": true,
                "own_votes": [1],
                "options": [
                    {"title": "a", "votes_count": 2},
                    {"title": "b", "votes_count": 1},
                ],
                "emojis": [],
            })
        );
        let mut multiple = post.poll.clone().expect("poll");
        multiple.multiple = true;
        multiple.voted = Vec::new();
        let value = json_of(&poll(POST, &multiple, &[]));
        assert_eq!(value["voters_count"], 3);
        assert_eq!(value["voted"], false);
        assert_eq!(value["own_votes"], json!([]));
    }

    #[test]
    fn a_boost_wraps_the_post() {
        let config = config();
        let mut post = post("original");
        post.entry_id = ENTRY;
        post.reposted_by = Some(card(BOB, "bob", None));
        post.viewer.reposted = true;
        let value = json_of(&status(&post, &Ctx::new(&config)));
        assert_eq!(value["id"], ENTRY.to_string());
        assert_eq!(value["account"]["acct"], "bob");
        assert_eq!(value["content"], "");
        assert_eq!(value["url"], Value::Null);
        assert_eq!(value["reblogged"], true);
        assert_eq!(value["created_at"], iso(uuid_time(ENTRY)));
        assert_eq!(
            value["uri"],
            format!("https://social.example/users/bob/statuses/{ENTRY}/activity")
        );
        assert_eq!(value["reblog"]["id"], POST.to_string());
        assert_eq!(value["reblog"]["account"]["acct"], "ada");
        assert_eq!(value["reblog"]["content"], "<p>original</p>");
        assert_eq!(value["reblog"]["reblog"], Value::Null);
    }

    #[test]
    fn a_remote_post_uses_its_page_url() {
        let config = config();
        let mut post = post("from afar");
        post.author = card(BOB, "bob", Some("far.example"));
        post.url = Some("https://far.example/@bob/1".to_owned());
        let value = json_of(&status(&post, &Ctx::new(&config)));
        assert_eq!(value["url"], "https://far.example/@bob/1");
        assert_eq!(value["uri"], "https://far.example/@bob/1");
        assert_eq!(value["account"]["acct"], "bob@far.example");
    }

    #[test]
    fn a_link_card_becomes_a_preview_card() {
        let config = config();
        let mut post = post("see https://x.example");
        post.card = Some(LinkCardDto {
            url: "https://x.example".to_owned(),
            title: "X".to_owned(),
            description: "About X".to_owned(),
            site: "x.example".to_owned(),
            image: Some("/media/card.webp".to_owned()),
            large: true,
        });
        let value = json_of(&status(&post, &Ctx::new(&config)));
        assert_eq!(
            value["card"],
            json!({
                "url": "https://x.example",
                "title": "X",
                "description": "About X",
                "type": "link",
                "author_name": "",
                "author_url": "",
                "provider_name": "x.example",
                "provider_url": "",
                "html": "",
                "width": 0,
                "height": 0,
                "image": "https://social.example/media/card.webp",
                "embed_url": "",
                "blurhash": null,
            })
        );
    }

    #[test]
    fn a_quote_post_keeps_its_own_text() {
        let config = config();
        let mut post = post("agreed");
        post.quote = Some(QuoteDto {
            id: PARENT,
            available: true,
            author: Some(card(BOB, "bob", None)),
            text: "the quoted words".to_owned(),
            rich_text: Vec::new(),
            cw: String::new(),
            sensitive: false,
            created_at: t("2026-11-08T10:00:00Z"),
            emojis: Vec::new(),
        });
        let value = json_of(&status(&post, &Ctx::new(&config)));
        assert_eq!(value["content"], "<p>agreed</p>");
    }

    #[test]
    fn notification_types_follow_mastodon() {
        let config = config();
        let ctx = Ctx::new(&config);
        let actor = card(BOB, "bob", None);
        let at = t("2026-11-09T11:00:00Z");
        for (kind, expected) in [
            (NotificationKindDto::Like, Some("favourite")),
            (NotificationKindDto::Repost, Some("reblog")),
            (NotificationKindDto::Reply, Some("mention")),
            (NotificationKindDto::Mention, Some("mention")),
            (NotificationKindDto::Quote, Some("mention")),
            (NotificationKindDto::Follow, Some("follow")),
            (NotificationKindDto::FollowRequest, Some("follow_request")),
            (NotificationKindDto::Poll, Some("poll")),
            (NotificationKindDto::Affiliation, None),
        ] {
            let made = notification(Uuid::from_u128(5), kind, at, &actor, None, &ctx);
            assert_eq!(made.as_ref().map(|n| n.kind), expected, "{kind:?}");
        }
        let with_status = notification(
            Uuid::from_u128(5),
            NotificationKindDto::Like,
            at,
            &actor,
            Some(&post("liked")),
            &ctx,
        )
        .expect("favourite");
        let value = json_of(&with_status);
        assert_eq!(value["type"], "favourite");
        assert_eq!(value["created_at"], "2026-11-09T11:00:00.000Z");
        assert_eq!(value["account"]["acct"], "bob");
        assert_eq!(value["status"]["id"], POST.to_string());
    }

    #[test]
    fn a_relation_maps_to_a_relationship() {
        let value = json_of(&relationship(
            BOB,
            Relation {
                following: true,
                follows_you: true,
                blocked: false,
                blocks_you: true,
                muted: true,
                follow_pending: false,
            },
        ));
        assert_eq!(
            value,
            json!({
                "id": BOB.to_string(),
                "following": true,
                "showing_reblogs": true,
                "notifying": false,
                "languages": null,
                "followed_by": true,
                "blocking": false,
                "blocked_by": true,
                "muting": true,
                "muting_notifications": false,
                "requested": false,
                "requested_by": false,
                "domain_blocking": false,
                "endorsed": false,
                "note": "",
            })
        );
    }

    #[test]
    fn a_credential_account_flattens_the_account_and_adds_source_and_role() {
        let config = config();
        let me = MeDto {
            user: user(ADA, "ada", None),
            role: crate::dto::Role::Admin,
            settings: crate::dto::SettingsDto {
                dm_policy: crate::dto::DmPolicy::Everyone,
                discoverable: true,
                language: Some("pl".to_owned()),
                show_client: true,
                notif_prefs: crate::dto::NotifPrefsDto {
                    like: true,
                    repost: true,
                    follow: true,
                    reply: true,
                    mention: true,
                    quote: true,
                },
                muted_words: Vec::new(),
            },
            federated_handle: "ada@social.example".to_owned(),
        };
        let value = json_of(&credential_account(&me, 2, &Ctx::new(&config)));
        assert_eq!(value["username"], "ada");
        assert_eq!(value["source"]["note"], "I write #rust");
        assert_eq!(value["source"]["privacy"], "public");
        assert_eq!(value["source"]["language"], "pl");
        assert_eq!(value["source"]["follow_requests_count"], 2);
        assert_eq!(
            value["source"]["fields"][0],
            json!({"name": "Website", "value": "https://ada.example/", "verified_at": null})
        );
        assert_eq!(value["role"]["name"], "Admin");
    }

    #[test]
    fn a_conversation_has_the_other_account_and_a_direct_status() {
        let config = config();
        let ctx = Ctx::new(&config);
        let viewer = card(ADA, "ada", None);
        let other = card(BOB, "bob", None);
        let last = MessageDto {
            id: Uuid::from_u128(20),
            text: "psst".to_owned(),
            ciphertext: None,
            created_at: t("2026-11-09T12:00:00Z"),
            mine: false,
            read_at: None,
            emojis: Vec::new(),
        };
        let value = json_of(&conversation(
            &ConversationDto {
                user: other,
                unread: 2,
                last,
                peer_key: None,
            },
            &viewer,
            &ctx,
        ));
        assert_eq!(value["id"], BOB.to_string());
        assert_eq!(value["unread"], true);
        assert_eq!(value["accounts"][0]["acct"], "bob");
        assert_eq!(value["last_status"]["visibility"], "direct");
        assert_eq!(value["last_status"]["account"]["acct"], "bob");
        assert_eq!(value["last_status"]["mentions"][0]["acct"], "ada");
        assert_eq!(value["last_status"]["content"], "<p>psst</p>");
    }

    #[test]
    fn a_list_and_a_tag() {
        let config = config();
        let value = json_of(&list(&ListDto {
            id: Uuid::from_u128(30),
            name: "Friends".to_owned(),
            description: String::new(),
            private: true,
            created_at: t("2026-11-09T10:00:00Z"),
            owner: card(ADA, "ada", None),
            is_owner: true,
            members: 4,
            has_member: None,
        }));
        assert_eq!(
            value,
            json!({"id": Uuid::from_u128(30).to_string(), "title": "Friends", "replies_policy": "list", "exclusive": false})
        );
        assert_eq!(
            json_of(&tag("rust", &config)),
            json!({"name": "rust", "url": "https://social.example/tags/rust"})
        );
    }

    #[test]
    fn timestamps_are_utc_with_milliseconds() {
        assert_eq!(
            iso(t("2026-11-09T12:00:00.5+02:00")),
            "2026-11-09T10:00:00.500Z"
        );
        assert_eq!(date(t("2026-11-09T23:59:59-05:00")), "2026-11-10");
        assert_eq!(uuid_time(Uuid::nil()), OffsetDateTime::UNIX_EPOCH);
        assert_eq!(
            uuid_time(Uuid::from_u128(0x0198_aaaa_0000_7000_8000_0000_0000_0001)).unix_timestamp(),
            0x0198_aaaa_0000_i64 / 1000
        );
    }

    #[test]
    fn oauth_entities() {
        let app = Application {
            id: Uuid::from_u128(40),
            name: "Phanpy".to_owned(),
            website: None,
            scopes: vec!["read".to_owned(), "write".to_owned()],
            redirect_uris: vec!["https://phanpy.social/".to_owned()],
            redirect_uri: "https://phanpy.social/".to_owned(),
            vapid_key: String::new(),
        };
        let value = json_of(&CredentialApplication {
            application: app,
            client_id: "id".to_owned(),
            client_secret: "secret".to_owned(),
            client_secret_expires_at: 0,
        });
        assert_eq!(value["name"], "Phanpy");
        assert_eq!(value["client_id"], "id");
        assert_eq!(value["client_secret_expires_at"], 0);
        assert_eq!(value["redirect_uris"], json!(["https://phanpy.social/"]));
        assert_eq!(value["website"], Value::Null);
        let token = json_of(&Token {
            access_token: "t".to_owned(),
            token_type: "Bearer",
            scope: "read write".to_owned(),
            created_at: 1_700_000_000,
        });
        assert_eq!(
            token,
            json!({"access_token": "t", "token_type": "Bearer", "scope": "read write", "created_at": 1_700_000_000})
        );
    }
}
