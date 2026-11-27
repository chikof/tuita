//! Request and response bodies of the JSON API.
//!
//! Conventions: camelCase fields, timestamps as RFC 3339 strings, ids as UUID strings.
//! Optional values serialize as `null`. In request bodies an absent optional field and `null`
//! mean the same.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// Declares a string-valued enum that is stored as text in Postgres.
macro_rules! text_enum {
    (
        $(#[$meta:meta])*
        $name:ident { $($(#[$vmeta:meta])* $variant:ident => $text:literal),+ $(,)? }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub enum $name {
            $($(#[$vmeta])* #[serde(rename = $text)] $variant,)+
        }

        impl $name {
            /// The value stored in the database and sent over the wire.
            #[must_use]
            pub const fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $text,)+ }
            }

            /// Parses the stored text; `None` when it is not one of the variants.
            #[must_use]
            pub fn parse(text: &str) -> Option<Self> {
                match text { $($text => Some(Self::$variant),)+ _ => None }
            }
        }
    };
}

/// Body of every error response: `{"error": "<message>"}`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorBody {
    /// Human-readable message, safe to show to the user.
    pub error: String,
}

/// Body of actions that only succeed or fail: `{"ok": true}`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkDto {
    /// Always true; failures are errors.
    pub ok: bool,
}

/// One page of a list. `next` is the opaque value to send back as `before` (or `cursor` for
/// the For you timeline); `null` means the end of the list.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Page<T> {
    /// The entries, in display order.
    pub items: Vec<T>,
    /// Cursor of the next page, or `null` when this is the last.
    pub next: Option<String>,
}

text_enum! {
    /// Shape of the pixel-grid avatar.
    AvatarShape {
        /// Square.
        Square => "square",
        /// Circle.
        Circle => "circle",
        /// Square with rounded corners.
        Squircle => "squircle",
        /// Octagon.
        Octagon => "octagon",
    }
}

text_enum! {
    /// The ink tone the user picked for the UI.
    Ink {
        /// Warm white ink.
        Bone => "bone",
        /// Green phosphor.
        Phosphor => "phosphor",
        /// Amber.
        Amber => "amber",
        /// Cyan.
        Cyan => "cyan",
        /// Rose.
        Rose => "rose",
    }
}

text_enum! {
    /// An account's role on the instance.
    Role {
        /// A regular account.
        User => "user",
        /// Can delete posts and handle reports.
        Moderator => "moderator",
        /// Runs the instance.
        Admin => "admin",
    }
}

text_enum! {
    /// Who may reply to a post.
    ReplyPolicy {
        /// Anyone.
        Everyone => "everyone",
        /// Accounts the author follows.
        Following => "following",
        /// Accounts the post mentions.
        Mentioned => "mentioned",
    }
}

text_enum! {
    /// Who may see a post.
    Visibility {
        /// Everyone; timelines, search and tags list it.
        Public => "public",
        /// Everyone with the link, but no public timeline lists it.
        Unlisted => "unlisted",
        /// Followers of the author.
        Followers => "followers",
        /// Only the people the post mentions.
        Direct => "direct",
    }
}

text_enum! {
    /// Who may send direct messages.
    DmPolicy {
        /// Anyone.
        Everyone => "everyone",
        /// Accounts the user follows.
        Following => "following",
        /// Nobody.
        None => "none",
    }
}

text_enum! {
    /// Who signs up.
    SignupsMode {
        /// Anyone may sign up.
        Open => "open",
        /// Only with an invite code.
        Invite => "invite",
        /// Nobody.
        Closed => "closed",
    }
}

text_enum! {
    /// What a notification is about.
    NotificationKindDto {
        /// Someone liked a post.
        Like => "like",
        /// Someone reposted a post.
        Repost => "repost",
        /// Someone replied.
        Reply => "reply",
        /// Someone mentioned the user.
        Mention => "mention",
        /// Someone quoted a post.
        Quote => "quote",
        /// Someone followed the user.
        Follow => "follow",
        /// Someone asked to follow the user.
        FollowRequest => "follow_request",
        /// A poll ended.
        Poll => "poll",
        /// An organization affiliated the user.
        Affiliation => "affiliation",
    }
}

text_enum! {
    /// Which tab of a profile.
    ProfileTab {
        /// Posts and reposts, pinned post first.
        Posts => "posts",
        /// Replies.
        Replies => "replies",
        /// Posts with media.
        Media => "media",
        /// Posts the user liked.
        Likes => "likes",
    }
}

text_enum! {
    /// Which kind of search result.
    SearchType {
        /// Posts.
        Posts => "posts",
        /// People.
        People => "people",
        /// Posts with media.
        Media => "media",
    }
}

text_enum! {
    /// Which timeline.
    Feed {
        /// Posts and reposts of people the user follows.
        Following => "following",
        /// Ranked recent posts.
        Foryou => "foryou",
    }
}

text_enum! {
    /// Which notifications to list.
    NotificationFilter {
        /// Everything.
        All => "all",
        /// Replies, mentions and quotes.
        Mentions => "mentions",
    }
}

text_enum! {
    /// What a `read` stream event cleared.
    ReadKind {
        /// Notifications were marked read.
        Notifications => "notifications",
        /// Messages were marked read.
        Messages => "messages",
    }
}
text_enum! {
    /// What an upload holds.
    MediaKind {
        /// A still or animated picture.
        Image => "image",
        /// A video.
        Video => "video",
    }
}

text_enum! {
    /// Where a scheduled post stands.
    ScheduledStatus {
        /// Waiting for its time.
        Pending => "pending",
        /// Published; `postId` names the post.
        Published => "published",
        /// Publishing failed; `error` says why.
        Failed => "failed",
    }
}

text_enum! {
    /// Why an account or post is reported.
    ReportReason {
        /// Unwanted ads or floods.
        Spam => "spam",
        /// Harassment.
        Harassment => "harassment",
        /// Hate.
        Hate => "hate",
        /// Violence.
        Violence => "violence",
        /// False information.
        Misinformation => "misinformation",
        /// Pretending to be someone else.
        Impersonation => "impersonation",
        /// Something else; say what in the note.
        Other => "other",
    }
}

// ------------------------------------------------------------------ custom emotes

/// A custom emote: the image that `:shortcode:` stands for.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmojiDto {
    /// The name between the colons, in the case the emote was created with.
    pub shortcode: String,
    /// Site-relative URL of the image, `/media/{id}.{ext}`.
    pub url: String,
    /// The group the picker files it under.
    pub category: Option<String>,
}

/// `PATCH /admin/emojis/{shortcode}`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchEmojiRequest {
    /// The new category, up to 32 characters; `null` or blank removes it.
    #[serde(default)]
    pub category: Option<String>,
}

// ------------------------------------------------------------------ users

/// Follower, following and post totals of an account.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserCounts {
    /// Accounts following this one.
    pub followers: i32,
    /// Accounts this one follows.
    pub following: i32,
    /// Posts that are not deleted.
    pub posts: i32,
}

/// How the viewer and an account relate.
#[allow(
    clippy::struct_excessive_bools,
    reason = "wire format: one JSON flag per relation"
)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Relation {
    /// The viewer follows the account.
    pub following: bool,
    /// The account follows the viewer.
    pub follows_you: bool,
    /// The viewer blocked the account.
    pub blocked: bool,
    /// The account blocked the viewer.
    pub blocks_you: bool,
    /// The viewer muted the account.
    pub muted: bool,
    /// The viewer asked to follow the account and waits for the answer (remote accounts).
    pub follow_pending: bool,
}

/// A profile card: everything the profile header and people lists show.
#[allow(
    clippy::struct_excessive_bools,
    reason = "wire format: one JSON flag per fact"
)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDto {
    /// The account id.
    pub id: Uuid,
    /// The handle without `@`.
    pub handle: String,
    /// The home domain; `null` for accounts of this instance.
    pub domain: Option<String>,
    /// The name shown next to the handle.
    pub display_name: String,
    /// Short biography.
    pub bio: String,
    /// Free-text location.
    pub location: String,
    /// Website URL, or empty.
    pub website: String,
    /// URL of the avatar image; `null` shows the generated avatar.
    pub avatar_url: Option<String>,
    /// URL of the banner image; `null` shows the generated banner.
    pub banner_url: Option<String>,
    /// The avatar's shape.
    pub avatar_shape: AvatarShape,
    /// The ink the account picked.
    pub ink: Ink,
    /// Carries the verified badge.
    pub verified: bool,
    /// `admin` or `moderator` for staff, else `null`.
    pub staff: Option<Role>,
    /// An organization account.
    pub org: bool,
    /// An automated account.
    pub bot: bool,
    /// The handle of the account that runs a bot; `null` when none is named.
    pub bot_owner: Option<String>,
    /// The organization the account is affiliated with, once the account accepted it.
    pub affiliation: Option<AffiliationBadgeDto>,
    /// The pinned post.
    pub pinned_post_id: Option<Uuid>,
    /// Sign-up time.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// Totals.
    pub counts: UserCounts,
    /// Relation to the viewer.
    pub relation: Relation,
    /// Suspended. Only staff ever see a suspended account.
    pub suspended: bool,
    /// The account's profile page on its own instance (`http` or `https`); `null` for local
    /// accounts.
    pub url: Option<String>,
    /// Follows need the account's approval (remote accounts only).
    pub locked: bool,
    /// The custom emotes the display name, bio and location mention.
    pub emojis: Vec<EmojiDto>,
}

/// The small author card shown on posts, notifications and account lists.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserRefDto {
    /// The account id.
    pub id: Uuid,
    /// The handle without `@`.
    pub handle: String,
    /// The home domain; `null` for accounts of this instance.
    pub domain: Option<String>,
    /// The name shown next to the handle.
    pub display_name: String,
    /// URL of the avatar image; `null` shows the generated avatar.
    pub avatar_url: Option<String>,
    /// The avatar's shape.
    pub avatar_shape: AvatarShape,
    /// The ink the account picked.
    pub ink: Ink,
    /// Carries the verified badge.
    pub verified: bool,
    /// An automated account.
    pub bot: bool,
    /// The custom emotes the display name mentions.
    pub emojis: Vec<EmojiDto>,
}

/// Which notification kinds the user wants. All default to on.
#[allow(
    clippy::struct_excessive_bools,
    reason = "wire format: one JSON flag per notification kind"
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotifPrefsDto {
    /// Likes.
    pub like: bool,
    /// Reposts.
    pub repost: bool,
    /// Follows.
    pub follow: bool,
    /// Replies.
    pub reply: bool,
    /// Mentions.
    pub mention: bool,
    /// Quotes.
    pub quote: bool,
}

/// Private settings only the account owner sees.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsDto {
    /// Who may send direct messages.
    pub dm_policy: DmPolicy,
    /// Shown in suggestions and people search.
    pub discoverable: bool,
    /// Preferred interface language.
    pub language: Option<String>,
    /// Show the posting device on posts.
    pub show_client: bool,
    /// Notification kinds that are on.
    pub notif_prefs: NotifPrefsDto,
    /// Words hidden from timelines.
    pub muted_words: Vec<String>,
}

/// The signed-in account with its private settings.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeDto {
    /// The profile card.
    pub user: UserDto,
    /// The account's role.
    pub role: Role,
    /// Private settings.
    pub settings: SettingsDto,
    /// `handle@domain`, how remote servers address the account.
    pub federated_handle: String,
}

/// An account signed in on this browser, for the account switcher.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountDto {
    /// The account.
    pub user: UserRefDto,
    /// The account that is active in this browser.
    pub active: bool,
}

/// `GET /accounts`: the accounts signed in on this browser.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountsDto {
    /// The signed-in accounts, active first.
    pub accounts: Vec<AccountDto>,
    /// How many accounts one browser may hold.
    pub max: u8,
}

/// `GET /users/{handle}/followed_by_known`: followers of an account whom the viewer follows.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnownFollowersDto {
    /// The newest few.
    pub users: Vec<UserRefDto>,
    /// How many there are in total.
    pub count: i64,
}

// ------------------------------------------------------------------ auth and account

/// `POST /auth/signup`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignupRequest {
    /// The wanted handle, 3 to 15 letters, digits or underscores.
    pub handle: String,
    /// The display name, 1 to 50 characters.
    pub display_name: String,
    /// The password, 8 to 200 characters.
    pub password: String,
    /// The user accepted the terms and privacy policy. Must be true.
    pub accept_terms: bool,
    /// Keep the accounts already signed in on this browser.
    #[serde(default)]
    pub add: bool,
    /// Preferred interface language.
    #[serde(default)]
    pub language: Option<String>,
    /// The invite code; required while the instance takes sign-ups by invite only.
    #[serde(default)]
    pub invite: Option<String>,
}

/// `POST /auth/login`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    /// The handle, with or without `@`.
    pub handle: String,
    /// The password.
    pub password: String,
    /// Keep the accounts already signed in on this browser.
    #[serde(default)]
    pub add: bool,
}

/// Answer of signup, login and account switching.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    /// The account that is now active.
    pub user: MeDto,
}

/// `GET /me`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeResponse {
    /// The active account, or `null` when signed out.
    pub user: Option<MeDto>,
}

/// `POST /auth/logout`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogoutRequest {
    /// Sign out every account on this browser, not just the active one.
    #[serde(default)]
    pub all: bool,
}

/// Answer of logout.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogoutResponse {
    /// The account that became active, or `null` when none is left.
    pub switched_to: Option<MeDto>,
}

/// `POST /accounts/switch`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwitchRequest {
    /// The account to make active.
    pub id: Uuid,
}

/// `PATCH /me`: only the fields present change.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchMeRequest {
    /// Display name, 1 to 50 characters.
    #[serde(default)]
    pub display_name: Option<String>,
    /// Bio, up to 160 characters.
    #[serde(default)]
    pub bio: Option<String>,
    /// Location, up to 30 characters.
    #[serde(default)]
    pub location: Option<String>,
    /// Website, up to 100 characters; `""` clears it.
    #[serde(default)]
    pub website: Option<String>,
    /// Avatar shape.
    #[serde(default)]
    pub avatar_shape: Option<AvatarShape>,
    /// Ink tone.
    #[serde(default)]
    pub ink: Option<Ink>,
    /// Who may send direct messages.
    #[serde(default)]
    pub dm_policy: Option<DmPolicy>,
    /// Show up in suggestions and people search.
    #[serde(default)]
    pub discoverable: Option<bool>,
    /// Show the posting device.
    #[serde(default)]
    pub show_client: Option<bool>,
    /// Interface language; `""` clears it.
    #[serde(default)]
    pub language: Option<String>,
    /// Muted words, up to 100 of up to 40 characters.
    #[serde(default)]
    pub muted_words: Option<Vec<String>>,
    /// Notification kinds that are on.
    #[serde(default)]
    pub notif_prefs: Option<NotifPrefsDto>,
}

/// `POST /me/password`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasswordRequest {
    /// The current password.
    pub current: String,
    /// The new password, 8 to 200 characters.
    pub next: String,
}

/// A login session of the signed-in account.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionDto {
    /// The session id. The token itself is never sent.
    pub id: Uuid,
    /// This is the session making the request.
    pub current: bool,
    /// Sign-in time.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// Last request, at most an hour stale.
    #[serde(with = "time::serde::rfc3339")]
    pub last_used_at: OffsetDateTime,
    /// When the session ends.
    #[serde(with = "time::serde::rfc3339")]
    pub expires_at: OffsetDateTime,
    /// The browser's `User-Agent`.
    pub user_agent: String,
    /// The last client address.
    pub ip: Option<String>,
    /// Two-letter country, when a trusted proxy reports it.
    pub country: Option<String>,
    /// The device family, like `linux` or `iphone`.
    pub client: Option<String>,
}

/// `DELETE /me/sessions`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RevokedDto {
    /// How many other sessions ended.
    pub revoked: i64,
}

// ------------------------------------------------------------------ instance

/// Sign-up and post totals of the instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceStats {
    /// Local accounts.
    pub users: i64,
    /// Local posts that are not deleted.
    pub posts: i64,
}

/// `GET /instance`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceDto {
    /// Instance name.
    pub name: String,
    /// Instance description.
    pub description: String,
    /// The federation domain.
    pub domain: String,
    /// The server version.
    pub version: String,
    /// Who may sign up.
    pub signups: SignupsMode,
    /// Whether sign-ups are open right now (open, or no admin exists yet).
    pub registrations_open: bool,
    /// Contact address or URL, or empty.
    pub contact: String,
    /// House rules, one sentence each.
    pub rules: Vec<String>,
    /// Minimum age.
    pub min_age: u8,
    /// Where users can get the running source (AGPL); `null` when the operator set none.
    pub source_url: Option<String>,
    /// Largest video upload in MiB.
    pub max_video_mb: u16,
    /// The interface language to suggest before sign-in: one of `en pl de es fr pt uk it nl tr cs ja`.
    pub detected_language: String,
    /// Totals.
    pub stats: InstanceStats,
}

/// `GET /badges` and the `badges` stream event.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BadgesDto {
    /// Unread notifications.
    pub notifications: i64,
    /// Unread direct messages.
    pub messages: i64,
    /// Open reports; only staff get this.
    pub reports: Option<i64>,
}

// ------------------------------------------------------------------ posts

/// One piece of a post's text.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SegmentDto {
    /// Plain text.
    Text {
        /// The text.
        text: String,
    },
    /// `@handle` or `@handle@domain`.
    Mention {
        /// The handle without `@`.
        handle: String,
        /// The domain for remote mentions.
        domain: Option<String>,
    },
    /// `#tag`.
    Hashtag {
        /// The lower-case tag.
        tag: String,
        /// The tag as typed, without `#`.
        text: String,
    },
    /// An `http://` or `https://` URL.
    Link {
        /// The URL as typed.
        url: String,
    },
}

/// Totals of a post.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostCounts {
    /// Likes.
    pub likes: i32,
    /// Reposts.
    pub reposts: i32,
    /// Replies that are not deleted.
    pub replies: i32,
    /// Quote posts that are not deleted.
    pub quotes: i32,
    /// Times the post was opened.
    pub views: i64,
}

/// What the viewer did with a post.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostViewer {
    /// The viewer liked it.
    pub liked: bool,
    /// The viewer reposted it.
    pub reposted: bool,
    /// The viewer bookmarked it.
    pub bookmarked: bool,
}

/// The post a reply answers: enough for "Replying to @handle".
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplyToDto {
    /// The parent post.
    pub id: Uuid,
    /// The parent's author, or `null` when the author is unavailable.
    pub handle: Option<String>,
}

/// The embedded post of a quote post.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteDto {
    /// The quoted post.
    pub id: Uuid,
    /// False when the post was deleted or its author is hidden from the viewer; the other
    /// fields are then empty.
    pub available: bool,
    /// The quoted post's author.
    pub author: Option<UserRefDto>,
    /// The quoted text.
    pub text: String,
    /// Segments of the quoted text.
    pub rich_text: Vec<SegmentDto>,
    /// The quoted content warning.
    pub cw: String,
    /// The quoted post is marked sensitive.
    pub sensitive: bool,
    /// When the quoted post was written.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// The custom emotes the quoted text and content warning mention.
    pub emojis: Vec<EmojiDto>,
}

/// An image or video attached to a post, or an upload not yet attached.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaDto {
    /// The media id.
    pub id: Uuid,
    /// `image` or `video`.
    pub kind: MediaKind,
    /// The file URL, `/media/{id}.{ext}`. Never dithered, never resized by the server beyond
    /// the stored 2048 px limit.
    pub url: String,
    /// A video's poster frame.
    pub poster_url: Option<String>,
    /// The description for screen readers.
    pub alt: String,
    /// Width in pixels.
    pub width: Option<i32>,
    /// Height in pixels.
    pub height: Option<i32>,
}

text_enum! {
    /// What an upload is for. Decides how an image is cropped and sized.
    MediaPurpose {
        /// A post attachment: long edge at most 2048 px.
        Post => "post",
        /// An avatar: cropped, stored at 400×400.
        Avatar => "avatar",
        /// A banner: cropped, stored at 1500×500.
        Banner => "banner",
    }
}

/// The `crop` field of `POST /media` as JSON, for avatars and banners. The image is rotated
/// clockwise by `rotate` degrees first, then `x`, `y`, `w`, `h` (pixels of the rotated image)
/// pick the area that is scaled to the stored size.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CropRect {
    /// Left edge.
    pub x: u32,
    /// Top edge.
    pub y: u32,
    /// Width.
    pub w: u32,
    /// Height.
    pub h: u32,
    /// Clockwise rotation: 0, 90, 180 or 270.
    #[serde(default)]
    pub rotate: u16,
}

/// `PATCH /media/{id}`: describe an upload before it joins a post.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchMediaRequest {
    /// The description, up to 1000 characters.
    pub alt: String,
}

/// A link preview of the first link in a post. `null` until the unfurl worker finished it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkCardDto {
    /// The linked URL.
    pub url: String,
    /// The page title.
    pub title: String,
    /// The page description.
    pub description: String,
    /// The site name.
    pub site: String,
    /// The preview image URL.
    pub image: Option<String>,
    /// Show the large layout.
    pub large: bool,
}

/// A post as shown in timelines, threads and lists.
#[allow(
    clippy::struct_excessive_bools,
    reason = "wire format: one JSON flag per fact"
)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostDto {
    /// The post id.
    pub id: Uuid,
    /// The author.
    pub author: UserRefDto,
    /// The raw text.
    pub text: String,
    /// The text split into text, mentions, hashtags and links.
    pub rich_text: Vec<SegmentDto>,
    /// The content warning; empty when none.
    pub cw: String,
    /// Media is marked sensitive.
    pub sensitive: bool,
    /// When the post was written.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// When the post was last edited.
    #[serde(with = "time::serde::rfc3339::option")]
    pub edited_at: Option<OffsetDateTime>,
    /// Who may reply.
    pub reply_policy: ReplyPolicy,
    /// Who sees it. Local posts are `public`.
    pub visibility: Visibility,
    /// The page of a remote post on its own instance (`http` or `https`); `null` for local posts.
    pub url: Option<String>,
    /// The posting device family, when the author shows it.
    pub client: Option<String>,
    /// Totals.
    pub counts: PostCounts,
    /// What the viewer did with it.
    pub viewer: PostViewer,
    /// The viewer may reply.
    pub can_reply: bool,
    /// The post this one answers.
    pub reply_to: Option<ReplyToDto>,
    /// The post this one quotes, embedded.
    pub quote: Option<QuoteDto>,
    /// Attached media.
    pub media: Vec<MediaDto>,
    /// Link preview.
    pub card: Option<LinkCardDto>,
    /// The poll, when the post has one.
    pub poll: Option<PollDto>,
    /// The author hid this reply from the thread.
    pub hidden_reply: bool,
    /// Shown pinned on a profile.
    pub pinned: bool,
    /// Id of this entry in its list: `id`, or the repost row's id for a repost entry. Cursors
    /// and `since` use it.
    pub entry_id: Uuid,
    /// Set on a timeline entry that is someone's repost of this post.
    pub reposted_by: Option<UserRefDto>,
    /// The custom emotes the text, content warning and poll options mention.
    pub emojis: Vec<EmojiDto>,
}

/// `POST /posts`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePostRequest {
    /// The text, up to 500 characters (each link counts 23).
    #[serde(default)]
    pub text: String,
    /// Content warning, up to 100 characters.
    #[serde(default)]
    pub cw: String,
    /// Mark the post sensitive.
    #[serde(default)]
    pub sensitive: bool,
    /// The post to reply to.
    #[serde(default)]
    pub reply_to: Option<Uuid>,
    /// The post to quote.
    #[serde(default)]
    pub quote_of: Option<Uuid>,
    /// Who may reply; defaults to everyone.
    #[serde(default)]
    pub reply_policy: Option<ReplyPolicy>,
    /// Uploaded media: up to 4 images, or exactly 1 video. Not together with a poll.
    #[serde(default)]
    pub media_ids: Vec<Uuid>,
    /// A poll. Needs text; not together with media.
    #[serde(default)]
    pub poll: Option<PollInput>,
}

/// `GET /posts/{id}`: a post with its conversation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadDto {
    /// The post.
    pub post: PostDto,
    /// The posts above it, oldest first.
    pub ancestors: Vec<PostDto>,
    /// Direct replies, best first.
    pub replies: Vec<PostDto>,
    /// Replies the author hid.
    pub hidden_replies: Vec<PostDto>,
}

// ------------------------------------------------------------------ polls, edits, scheduling

/// One answer of a poll with its tally.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PollOptionDto {
    /// The answer text.
    pub text: String,
    /// Votes for it.
    pub votes: i32,
}

/// A poll as shown on a post.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PollDto {
    /// The answers, in order.
    pub options: Vec<PollOptionDto>,
    /// All votes. With a multiple-choice poll this exceeds `voters`.
    pub total: i32,
    /// Accounts that voted.
    pub voters: i32,
    /// A voter may pick several answers.
    pub multiple: bool,
    /// When voting ends.
    #[serde(with = "time::serde::rfc3339")]
    pub ends_at: OffsetDateTime,
    /// Voting has ended.
    pub closed: bool,
    /// Indexes of the answers the viewer picked; empty when they did not vote.
    pub voted: Vec<u8>,
}

/// A poll in a post request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PollInput {
    /// 2 to 4 different answers of at most 25 characters. Blank ones are dropped.
    pub options: Vec<String>,
    /// A voter may pick several answers.
    #[serde(default)]
    pub multiple: bool,
    /// How long the poll runs, 1 to 168 hours.
    pub duration_hours: u32,
}

/// `POST /posts/{id}/vote`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoteRequest {
    /// Indexes of the picked answers; exactly one unless the poll allows several.
    pub choices: Vec<u8>,
}

/// `PATCH /posts/{id}`: only the fields present change.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditPostRequest {
    /// The new text.
    #[serde(default)]
    pub text: Option<String>,
    /// The new content warning.
    #[serde(default)]
    pub cw: Option<String>,
}

/// One version of a post's text.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditVersionDto {
    /// The text.
    pub text: String,
    /// The content warning.
    pub cw: String,
    /// When this version was written.
    #[serde(with = "time::serde::rfc3339")]
    pub at: OffsetDateTime,
}

/// `GET /posts/{id}/edits`: the current text and the earlier versions, newest first.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostEditsDto {
    /// The text now.
    pub current: EditVersionDto,
    /// Earlier versions, newest first.
    pub edits: Vec<EditVersionDto>,
}

/// `POST /posts/thread`: a chain of posts; each one replies to the one before.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateThreadRequest {
    /// 2 to 25 posts. Only the first may set `replyTo` or `quoteOf`; each later one answers
    /// its predecessor.
    pub posts: Vec<CreatePostRequest>,
}

/// Answer of `POST /posts/thread`: the created posts in order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadPostsDto {
    /// The posts, first to last.
    pub posts: Vec<PostDto>,
}

/// `POST /scheduled`: a post request plus when to publish it. Replies cannot be scheduled.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleRequest {
    /// Between 1 minute and 90 days from now.
    #[serde(with = "time::serde::rfc3339")]
    pub publish_at: OffsetDateTime,
    /// The post.
    #[serde(flatten)]
    pub post: CreatePostRequest,
}

/// `PATCH /scheduled/{id}`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RescheduleRequest {
    /// Between 1 minute and 90 days from now.
    #[serde(with = "time::serde::rfc3339")]
    pub publish_at: OffsetDateTime,
}

/// A post waiting to be published, or one that was.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledPostDto {
    /// The schedule entry id.
    pub id: Uuid,
    /// When it publishes.
    #[serde(with = "time::serde::rfc3339")]
    pub publish_at: OffsetDateTime,
    /// Where it stands.
    pub status: ScheduledStatus,
    /// The published post, once there is one.
    pub post_id: Option<Uuid>,
    /// Why publishing failed.
    pub error: Option<String>,
    /// When it was scheduled.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// The text.
    pub text: String,
    /// The content warning.
    pub cw: String,
    /// Media is marked sensitive.
    pub sensitive: bool,
    /// Who may reply.
    pub reply_policy: ReplyPolicy,
    /// The attached media.
    pub media: Vec<MediaDto>,
    /// The poll, with its duration counted from publishing.
    pub poll: Option<PollInput>,
    /// The post it quotes.
    pub quote_of: Option<Uuid>,
}

/// `POST /views`: posts that were on screen for a moment.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewsRequest {
    /// At most 50 post ids; more are ignored.
    pub ids: Vec<Uuid>,
}

// ------------------------------------------------------------------ messages

/// One direct message.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageDto {
    /// The message id; ids sort by time.
    pub id: Uuid,
    /// The text. Empty for a sealed message (`ciphertext` is set): the server never holds that text.
    pub text: String,
    /// The sealed message, `v1.<sender key>.<recipient key>.<salt>.<iv>.<ciphertext>`; `null` for a
    /// plaintext message (from a remote account, or sent before encryption).
    pub ciphertext: Option<String>,
    /// When it was sent.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// The viewer sent it.
    pub mine: bool,
    /// When the recipient read it.
    #[serde(with = "time::serde::rfc3339::option")]
    pub read_at: Option<OffsetDateTime>,
    /// The custom emotes the text mentions.
    pub emojis: Vec<EmojiDto>,
}

/// One row of the conversation list.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationDto {
    /// The other person.
    pub user: UserRefDto,
    /// Messages from them the viewer has not read.
    pub unread: i64,
    /// The newest message.
    pub last: MessageDto,
    /// The other person's current message public key; `null` for a remote account and for a local
    /// account that has not turned encryption on.
    pub peer_key: Option<String>,
}

/// `GET /conversations/{handle}`: messages of one conversation, oldest first.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationViewDto {
    /// The other person.
    pub user: UserDto,
    /// One page of messages, oldest first.
    pub messages: Vec<MessageDto>,
    /// Cursor for the older page, sent back as `before`; `null` at the start of the conversation.
    pub next: Option<String>,
    /// Why the viewer cannot send a message to this person now; `null` when they can.
    pub blocked_reason: Option<String>,
    /// How a message to this person travels, see [`MessageMode`].
    pub mode: MessageMode,
    /// The other person's current message public key; `null` for a remote account and for a local
    /// account that has not turned encryption on.
    pub peer_key: Option<String>,
}

/// `POST /conversations/{handle}`. Exactly one of `text` and `ciphertext`, as the conversation's
/// [`MessageMode`] says: `ciphertext` for `sealed`, `text` for `plain`.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageRequest {
    /// 1 to 2000 characters, in `plain` mode.
    #[serde(default)]
    pub text: Option<String>,
    /// A sealed message, in `sealed` mode.
    #[serde(default)]
    pub ciphertext: Option<String>,
}

text_enum! {
    /// How a message to one person travels.
    MessageMode {
        /// Both sides have a message key: the browser seals the text and the server stores only
        /// ciphertext.
        Sealed => "sealed",
        /// No encryption is possible: the person is on another server, or (only when the instance
        /// allows plaintext) has no message key. A client that has seen a key for a local person
        /// must never fall back to this.
        Plain => "plain",
        /// A message cannot be sent yet: one side has not turned encryption on and the instance
        /// requires it.
        Unavailable => "unavailable",
    }
}

/// A message key: the public half and the passphrase-sealed private half.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageKeyDto {
    /// Base64url of the 65-byte uncompressed P-256 point.
    pub public_key: String,
    /// The private key sealed under the person's passphrase, opaque to the server.
    pub backup: String,
    /// When the key was made.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

/// `GET /me/message-key`.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MyMessageKeyDto {
    /// The account's key; `null` until it turns encryption on.
    pub key: Option<MessageKeyDto>,
}

/// `PUT /me/message-key`.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetMessageKeyRequest {
    /// Base64url of the 65-byte uncompressed P-256 point.
    pub public_key: String,
    /// The private key sealed under the passphrase.
    pub backup: String,
    /// Replace an existing key: earlier messages sealed to it stay unreadable.
    #[serde(default)]
    pub replace: bool,
}

// ------------------------------------------------------------------ lists

/// A list of accounts whose posts form a timeline.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListDto {
    /// The list id.
    pub id: Uuid,
    /// The name.
    pub name: String,
    /// What it is for.
    pub description: String,
    /// Only the owner may open it.
    pub private: bool,
    /// When it was created.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// The owner.
    pub owner: UserRefDto,
    /// The viewer owns it.
    pub is_owner: bool,
    /// Accounts in it.
    pub members: i64,
    /// Whether the account named by `GET /lists?member=` is in it; absent otherwise.
    pub has_member: Option<bool>,
}

/// `POST /lists`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListRequest {
    /// 1 to 40 characters.
    pub name: String,
    /// Up to 160 characters.
    #[serde(default)]
    pub description: String,
    /// Only the owner may open it.
    #[serde(default)]
    pub private: bool,
}

/// `PATCH /lists/{id}`: only the fields present change.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchListRequest {
    /// The new name.
    #[serde(default)]
    pub name: Option<String>,
    /// The new description.
    #[serde(default)]
    pub description: Option<String>,
    /// The new visibility.
    #[serde(default)]
    pub private: Option<bool>,
}

// ------------------------------------------------------------------ reports, export, deletion

/// `POST /reports`: name a post, or an account by `handle`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportRequest {
    /// The reported post; its author is the target.
    #[serde(default)]
    pub post_id: Option<Uuid>,
    /// The reported account when no post is named.
    #[serde(default)]
    pub handle: Option<String>,
    /// Why.
    pub reason: ReportReason,
    /// Details, up to 1000 characters.
    #[serde(default)]
    pub note: String,
}

/// Answer of `POST /reports`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportAckDto {
    /// Always true.
    pub ok: bool,
    /// The viewer already has an open report on the same target; nothing was added.
    pub duplicate: bool,
}

/// `DELETE /me`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteAccountRequest {
    /// The account password.
    pub password: String,
}

/// A post in the data export.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportPostDto {
    /// The post id.
    pub id: Uuid,
    /// The text.
    pub text: String,
    /// The content warning.
    pub cw: String,
    /// The post it answers.
    pub reply_to: Option<Uuid>,
    /// The post it quotes.
    pub quote_of: Option<Uuid>,
    /// The attached media URLs.
    pub media: Vec<String>,
    /// When it was written.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

/// A like, repost or bookmark in the data export.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportEdgeDto {
    /// The post.
    pub post_id: Uuid,
    /// When.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

/// A direct message in the data export.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportMessageDto {
    /// The other person's handle.
    pub with: String,
    /// The account sent it.
    pub sent: bool,
    /// The text. Empty for a sealed message: the export is made on the server, which cannot open it.
    pub text: String,
    /// The sealed message, as in [`MessageDto::ciphertext`]; `null` for a plaintext message.
    pub ciphertext: Option<String>,
    /// When it was sent.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

/// `GET /me/export`: everything the account holds, as a JSON download.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportDto {
    /// When the export was made.
    #[serde(with = "time::serde::rfc3339")]
    pub exported_at: OffsetDateTime,
    /// The instance domain.
    pub instance: String,
    /// The account and its settings.
    pub profile: MeDto,
    /// Posts that are not deleted.
    pub posts: Vec<ExportPostDto>,
    /// Liked posts.
    pub likes: Vec<ExportEdgeDto>,
    /// Reposted posts.
    pub reposts: Vec<ExportEdgeDto>,
    /// Bookmarked posts.
    pub bookmarks: Vec<ExportEdgeDto>,
    /// Handles the account follows.
    pub following: Vec<String>,
    /// Handles that follow the account.
    pub followers: Vec<String>,
    /// Handles the account blocked.
    pub blocked: Vec<String>,
    /// Handles the account muted.
    pub muted: Vec<String>,
    /// Messages sent and received that are not deleted.
    pub messages: Vec<ExportMessageDto>,
}

// ------------------------------------------------------------------ discovery

/// A hashtag and how many posts use it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrendDto {
    /// The lower-case tag without `#`.
    pub tag: String,
    /// Posts that use it.
    pub posts: i64,
}

/// `GET /timelines/{feed}/new`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewCountDto {
    /// Posts newer than `since`, at most 200.
    pub count: i64,
}

/// `GET /search`. Posts and media searches fill `posts`; people searches fill `users`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchDto {
    /// Matching accounts. A posts search shows a few above the posts on its first page.
    pub users: Vec<UserDto>,
    /// Matching posts.
    pub posts: Vec<PostDto>,
    /// Cursor of the next page.
    pub next: Option<String>,
}

// ------------------------------------------------------------------ notifications

/// One row of the notification list: one notification, or several collapsed into one.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationGroupDto {
    /// The newest notification of the group.
    pub id: Uuid,
    /// What happened.
    pub kind: NotificationKindDto,
    /// Every notification of the group was seen.
    pub read: bool,
    /// When the newest one happened.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// The first three actors, newest first.
    pub actors: Vec<UserRefDto>,
    /// How many distinct actors the group has.
    pub actor_count: u32,
    /// The post it is about.
    pub post: Option<PostDto>,
}

/// A short post excerpt for toasts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostPreviewDto {
    /// The post.
    pub id: Uuid,
    /// The first 280 characters.
    pub text: String,
    /// The content warning.
    pub cw: String,
}

/// The `notification` stream event: a toast's content.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationEventDto {
    /// The notification.
    pub id: Uuid,
    /// What happened.
    pub kind: NotificationKindDto,
    /// Who did it.
    pub actor: UserRefDto,
    /// The post it is about.
    pub post: Option<PostPreviewDto>,
    /// When it happened.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

/// The `message` stream event: a direct message arrived. Sent from M2 on.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageEventDto {
    /// The sender.
    pub from: UserRefDto,
    /// The first 280 characters of a plaintext message; empty when `encrypted`.
    pub text: String,
    /// The message is sealed: the preview is empty and only the browser can read it.
    pub encrypted: bool,
    /// When it was sent.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

/// The `read` stream event: another tab cleared notifications or messages.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadEventDto {
    /// What was cleared.
    pub kind: ReadKind,
}

/// The `card` stream event: a link preview finished loading. Sent from M2 on.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardEventDto {
    /// The post the preview belongs to.
    pub post_id: Uuid,
    /// The preview.
    pub card: LinkCardDto,
}

/// The `update` stream event of the Mastodon websocket: a timeline entry a recipient should see.
/// It names the entry; the websocket loads the post as the recipient sees it. Never sent on SSE.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostEventDto {
    /// The post.
    pub post_id: Uuid,
    /// The timeline entry: the post id, or the repost id for a boost.
    pub entry_id: Uuid,
    /// Who boosted it; `None` for the post itself.
    pub reposted_by: Option<Uuid>,
}

/// One server-sent event on `GET /stream`, as `{type, data}`. The SSE `event:` field repeats
/// `type`; the `data:` field is this whole object as JSON.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum StreamEvent {
    /// The unread counters changed. Sent first on every connection.
    Badges(BadgesDto),
    /// Someone did something to the user.
    Notification(Box<NotificationEventDto>),
    /// A direct message arrived.
    Message(Box<MessageEventDto>),
    /// Notifications or messages were marked read elsewhere.
    Read(ReadEventDto),
    /// A link preview is ready.
    Card(Box<CardEventDto>),
    /// A post or boost entered a followed timeline. Only the Mastodon websocket reads it; the SSE
    /// stream drops it and the web client never sees it.
    Update(Box<PostEventDto>),
    /// A timeline entry (a post id, or a repost id) was removed. Mastodon websocket only.
    Delete(Uuid),
}

impl StreamEvent {
    /// The SSE `event:` name, equal to the `type` field of the JSON.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Badges(_) => "badges",
            Self::Notification(_) => "notification",
            Self::Message(_) => "message",
            Self::Read(_) => "read",
            Self::Card(_) => "card",
            Self::Update(_) => "update",
            Self::Delete(_) => "delete",
        }
    }
}
// 32: the tag plus the largest inline payload (`BadgesDto`, 24 bytes); the rest are boxed.
#[cfg(target_pointer_width = "64")]
const _: () = assert!(std::mem::size_of::<StreamEvent>() == 32);

// ------------------------------------------------------------------ M3: admin, trust, bots, passkeys

text_enum! {
    /// The colour of an announcement banner.
    AnnouncementTone {
        /// Neutral.
        Info => "info",
        /// Good news.
        Success => "success",
        /// Something to watch out for.
        Warning => "warning",
    }
}

text_enum! {
    /// Whether a report still needs a decision.
    ReportStatus {
        /// Waiting for staff.
        Open => "open",
        /// Decided.
        Resolved => "resolved",
    }
}

text_enum! {
    /// What staff did about a report.
    ReportAction {
        /// Nothing; the report was unfounded.
        Dismiss => "dismiss",
        /// The reported post is deleted.
        DeletePost => "delete_post",
        /// The reported account is suspended.
        SuspendUser => "suspend_user",
        /// Both.
        DeleteAndSuspend => "delete_and_suspend",
    }
}

text_enum! {
    /// Which accounts the admin user list shows.
    UserFilter {
        /// Local accounts that can sign in.
        Local => "local",
        /// Accounts of other instances.
        Remote => "remote",
        /// Admins and moderators.
        Staff => "staff",
        /// Verified accounts.
        Verified => "verified",
        /// Suspended accounts.
        Suspended => "suspended",
        /// Every account that is not deleted.
        All => "all",
    }
}

text_enum! {
    /// The profile field that tripped the word filter.
    FilterField {
        /// The handle.
        Handle => "handle",
        /// The display name.
        Name => "name",
        /// The location.
        Location => "location",
        /// The bio.
        Bio => "bio",
    }
}

text_enum! {
    /// What an API token may do. `messages` covers `/conversations`, `read` every other `GET`,
    /// `write` every other request.
    TokenScope {
        /// Read requests.
        Read => "read",
        /// Everything that changes something, except messages.
        Write => "write",
        /// Conversations, read and write.
        Messages => "messages",
    }
}

text_enum! {
    /// Where an affiliation stands.
    AffiliationStatus {
        /// The organization invited the account; the account has not answered.
        Pending => "pending",
        /// The account accepted.
        Active => "active",
    }
}

// ---- admin: users, reports, audit

/// An account as staff see it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminUserDto {
    /// The profile card. `relation` is always empty.
    pub user: UserDto,
    /// The account's role.
    pub role: Role,
    /// The account was deleted (its row stays, anonymised).
    pub deleted: bool,
    /// Reports about the account that are still open.
    pub open_reports: i32,
    /// The last time one of the account's sessions was used.
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_active_at: Option<OffsetDateTime>,
}

/// One report on the account detail page.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserReportDto {
    /// The report id.
    pub id: Uuid,
    /// Why.
    pub reason: ReportReason,
    /// The reporter's details.
    pub note: String,
    /// Open or resolved.
    pub status: ReportStatus,
    /// What staff did.
    pub resolution: Option<ReportAction>,
    /// Handle of the reporter; `null` when that account is gone.
    pub reporter: Option<String>,
    /// When it was filed.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

/// One audit log row.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEntryDto {
    /// The row id; rows sort by it.
    pub id: Uuid,
    /// Handle of the staff member; `null` for the command line, the system, or a deleted account.
    pub actor: Option<String>,
    /// What happened, like `user.suspend`.
    pub action: String,
    /// What it happened to: `@handle`, `#<id>`, a code, or empty.
    pub target: String,
    /// A sentence of detail, or empty.
    pub detail: String,
    /// When.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

/// `GET /admin/users/{ref}`: everything a moderator needs about one account.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminUserDetailDto {
    /// The account.
    pub user: AdminUserDto,
    /// The invite code the account signed up with.
    pub invited_with: Option<String>,
    /// Registered passkeys.
    pub passkeys: i32,
    /// Live sessions.
    pub sessions: i32,
    /// API tokens.
    pub tokens: i32,
    /// Accounts this one follows.
    pub following: i32,
    /// The five newest posts.
    pub posts: Vec<PostDto>,
    /// The twenty newest reports about the account.
    pub reports: Vec<UserReportDto>,
    /// The twenty newest audit rows about the account, newest first.
    pub history: Vec<AuditEntryDto>,
}

/// `PATCH /admin/users/{ref}`: only the fields present change.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchAdminUserRequest {
    /// Give or take the verified badge.
    #[serde(default)]
    pub verified: Option<bool>,
    /// Suspend or lift the suspension. A suspension ends the account's sessions.
    #[serde(default)]
    pub suspended: Option<bool>,
    /// The new role. Admins only.
    #[serde(default)]
    pub role: Option<Role>,
}

/// `PATCH /admin/users/{ref}/org`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetOrgRequest {
    /// Mark the account as an organization, or not.
    pub org: bool,
}

/// `POST /admin/users/{ref}/rename`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameUserRequest {
    /// A new handle.
    #[serde(default)]
    pub handle: Option<String>,
    /// Set the display name to the handle.
    #[serde(default)]
    pub reset_name: bool,
    /// Clear the bio.
    #[serde(default)]
    pub reset_bio: bool,
}

/// `DELETE /admin/users/{ref}`; the body is optional.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteUserRequest {
    /// Why, up to 200 characters, kept in the audit log.
    #[serde(default)]
    pub reason: Option<String>,
}

/// `DELETE /admin/users/{ref}/posts`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurgedDto {
    /// Posts deleted.
    pub deleted: i32,
}

/// A report in the staff queue.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminReportDto {
    /// The report id.
    pub id: Uuid,
    /// Why.
    pub reason: ReportReason,
    /// The reporter's details.
    pub note: String,
    /// Open or resolved.
    pub status: ReportStatus,
    /// What staff did.
    pub resolution: Option<ReportAction>,
    /// When it was filed.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// When it was decided.
    #[serde(with = "time::serde::rfc3339::option")]
    pub resolved_at: Option<OffsetDateTime>,
    /// Who filed it; `null` when that account is gone.
    pub reporter: Option<UserRefDto>,
    /// The reported account.
    pub target: UserRefDto,
    /// The reported account is suspended.
    pub target_suspended: bool,
    /// Who decided it.
    pub resolved_by: Option<UserRefDto>,
    /// The reported post; `null` for an account report or once the post is deleted.
    pub post: Option<PostDto>,
    /// Open reports about the same account, this one included.
    pub same_target_open: i32,
}

/// `POST /admin/reports/{id}/resolve`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveReportRequest {
    /// What to do.
    pub action: ReportAction,
}

/// An account whose profile trips the word filter.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlaggedUserDto {
    /// The account.
    pub user: AdminUserDto,
    /// The first field that matched.
    pub field: FilterField,
    /// Why, as in `blocked words`.
    pub reason: String,
}

/// `GET /admin/filter/scan`: at most 200 accounts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterScanDto {
    /// The flagged accounts, newest first.
    pub flagged: Vec<FlaggedUserDto>,
}

// ---- admin: stats

/// Account totals.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsUsersDto {
    /// Local accounts that can sign in and are not deleted.
    pub local: i32,
    /// Local accounts without a password (demo accounts and accounts made by the command line).
    pub demo: i32,
    /// Suspended accounts.
    pub suspended: i32,
    /// Accounts with a session used in the last 7 days.
    pub active7d: i32,
}

/// Post totals.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsPostsDto {
    /// Posts that are not deleted.
    pub total: i32,
    /// Of those, written in the last 24 hours.
    pub last24h: i32,
}

/// What happened on one UTC day.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DayStatsDto {
    /// The day, `YYYY-MM-DD`, UTC.
    pub day: String,
    /// Accounts created.
    pub signups: i32,
    /// Posts written, deleted ones included.
    pub posts: i32,
    /// Likes given.
    pub likes: i32,
    /// Reports filed.
    pub reports: i32,
}

/// `GET /admin/stats`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminStatsDto {
    /// Account totals.
    pub users: StatsUsersDto,
    /// Post totals.
    pub posts: StatsPostsDto,
    /// Reports waiting for staff.
    pub reports_open: i32,
    /// The last 14 UTC days, oldest first, today last.
    pub days: Vec<DayStatsDto>,
    /// Posts and likes of the last 28 days: 7 rows (Monday to Sunday) of 24 columns (hour of the
    /// UTC day).
    pub heatmap: Vec<Vec<i32>>,
}

// ---- admin: instance settings

/// The admin-editable word filter.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentFilterDto {
    /// Blocked words, one per line or comma separated, up to 5000 characters.
    pub blocked: String,
    /// Allowed words that a blocked word would otherwise catch, up to 5000 characters.
    pub allowed: String,
    /// Reserved handles, up to 5000 characters.
    pub reserved: String,
}

/// `GET` and `PATCH /admin/settings`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminSettingsDto {
    /// Instance name, 1 to 40 characters.
    pub name: String,
    /// Description, up to 300 characters.
    pub description: String,
    /// Contact email, up to 120 characters, or empty.
    pub contact: String,
    /// House rules, one sentence each; together up to 3000 characters.
    pub rules: Vec<String>,
    /// Minimum age, 13 to 21.
    pub min_age: i32,
    /// Who may sign up.
    pub signups: SignupsMode,
    /// The word filter.
    pub filter: ContentFilterDto,
    /// The federation domain, from `PUBLIC_URL`; read only.
    pub domain: String,
}

/// `PATCH /admin/settings`: only the fields present change.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchSettingsRequest {
    /// Instance name.
    #[serde(default)]
    pub name: Option<String>,
    /// Description.
    #[serde(default)]
    pub description: Option<String>,
    /// Contact email.
    #[serde(default)]
    pub contact: Option<String>,
    /// House rules.
    #[serde(default)]
    pub rules: Option<Vec<String>>,
    /// Minimum age.
    #[serde(default)]
    pub min_age: Option<i32>,
    /// Who may sign up.
    #[serde(default)]
    pub signups: Option<SignupsMode>,
    /// The word filter; replaces all three of its lists.
    #[serde(default)]
    pub filter: Option<ContentFilterDto>,
}

// ---- announcements and invites

/// An announcement everyone sees.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnouncementDto {
    /// The announcement id.
    pub id: Uuid,
    /// The text, up to 280 characters.
    pub text: String,
    /// The banner colour.
    pub tone: AnnouncementTone,
    /// A link to follow, or empty.
    pub link: String,
    /// When it went up.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

/// An announcement in the admin list.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminAnnouncementDto {
    /// The announcement id.
    pub id: Uuid,
    /// The text.
    pub text: String,
    /// The banner colour.
    pub tone: AnnouncementTone,
    /// A link to follow, or empty.
    pub link: String,
    /// Shown to everyone until ended or expired.
    pub active: bool,
    /// When it went up.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// When it stops showing by itself.
    #[serde(with = "time::serde::rfc3339::option")]
    pub expires_at: Option<OffsetDateTime>,
}

/// `POST /admin/announcements`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAnnouncementRequest {
    /// The text, 1 to 280 characters.
    pub text: String,
    /// The banner colour; `info` when absent.
    #[serde(default)]
    pub tone: Option<AnnouncementTone>,
    /// A link, up to 200 characters, starting with `https://`, `http://` or `/`.
    #[serde(default)]
    pub link: Option<String>,
    /// Days until it expires, at most 365; absent or 0 means it stays until ended.
    #[serde(default)]
    pub days: Option<i32>,
}

/// An invite code in the admin list.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InviteDto {
    /// The code, like `AB3DE-FGH2J`.
    pub code: String,
    /// A note for the admin, up to 80 characters.
    pub note: String,
    /// How many accounts may sign up with it.
    pub max_uses: i32,
    /// How many did.
    pub uses: i32,
    /// When it stops working.
    #[serde(with = "time::serde::rfc3339::option")]
    pub expires_at: Option<OffsetDateTime>,
    /// An admin revoked it.
    pub revoked: bool,
    /// When it was made.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// Handles of the first twenty accounts that signed up with it.
    pub used_by: Vec<String>,
}

/// `POST /admin/invites`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInviteRequest {
    /// How many accounts may use it, 1 to 1000; 1 when absent.
    #[serde(default)]
    pub max_uses: Option<i32>,
    /// Days until it expires, at most 365; absent or 0 means it never expires.
    #[serde(default)]
    pub days: Option<i32>,
    /// A note, up to 80 characters.
    #[serde(default)]
    pub note: Option<String>,
}

// ---- API tokens and bots

/// An API token as listed. The secret is never part of it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenDto {
    /// The token id.
    pub id: Uuid,
    /// The name the owner gave it, 1 to 40 characters.
    pub name: String,
    /// What it may do.
    pub scopes: Vec<TokenScope>,
    /// The first 12 characters (`tuita_` and 6 more), to tell tokens apart.
    pub prefix: String,
    /// When it was made.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// The last use, at most a minute stale.
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_used_at: Option<OffsetDateTime>,
}

/// `POST /tokens`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTokenRequest {
    /// A name, 1 to 40 characters.
    pub name: String,
    /// What the token may do; `["read"]` when absent. At least one scope.
    #[serde(default)]
    pub scopes: Option<Vec<TokenScope>>,
}

/// The answer of `POST /tokens`: the only time the secret is sent.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatedTokenDto {
    /// The secret, `tuita_` and 43 URL-safe characters. Shown once; only its hash is stored.
    pub token: String,
    /// The token as it will be listed.
    pub info: TokenDto,
}

/// `PATCH /me/bot`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchBotRequest {
    /// Mark the account as automated, or not.
    pub bot: bool,
    /// The handle of the account that runs the bot; only used when `bot` is true.
    #[serde(default)]
    pub owner: Option<String>,
}

// ---- OAuth consent (the web client's authorization page; the OAuth endpoints themselves are Mastodon-shaped)

/// `GET /oauth/authorization`: what the consent card shows for an authorization request that
/// passed every check.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthConsentDto {
    /// The name the app gave when it registered. Anyone can register any name.
    pub app_name: String,
    /// The app's website, when it gave one.
    pub website: Option<String>,
    /// What the app would be allowed to do: `read`, `write`, or both.
    pub scopes: Vec<TokenScope>,
    /// The registered redirect URI the answer goes to.
    pub redirect_uri: String,
    /// Where the answer goes, for the user to see: the host of an `https` or loopback `http`
    /// URI, `scheme://host` of an app's own scheme. Empty for the out-of-band URI.
    pub redirect_host: String,
    /// The code is shown to the user instead of sent to a URI.
    pub oob: bool,
    /// The request carries a PKCE challenge.
    pub pkce: bool,
}

/// `POST /oauth/authorize`: the user's answer. The fields repeat the authorization request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthAuthorizeRequest {
    /// The app's public client id.
    pub client_id: String,
    /// The redirect URI; the app's only one when absent.
    #[serde(default)]
    pub redirect_uri: Option<String>,
    /// The scopes asked for, separated by spaces; `read` when absent.
    #[serde(default)]
    pub scope: Option<String>,
    /// Echoed to the redirect.
    #[serde(default)]
    pub state: Option<String>,
    /// `code`; the only type there is.
    #[serde(default)]
    pub response_type: Option<String>,
    /// The PKCE challenge, `S256` of the verifier.
    #[serde(default)]
    pub code_challenge: Option<String>,
    /// `S256`; the only method there is.
    #[serde(default)]
    pub code_challenge_method: Option<String>,
    /// The user allowed the app (true) or denied it (false).
    pub approve: bool,
}

/// The answer of `POST /oauth/authorize`: where to send the user next.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthAuthorizeDto {
    /// The URL to send the browser to, with `code` (and `state`), or with `error=access_denied`
    /// when the user denied. `null` for the out-of-band URI.
    pub redirect: Option<String>,
    /// The code to show the user, for the out-of-band URI when the user approved. `null` otherwise.
    pub code: Option<String>,
}

// ---- affiliations

/// The organization badge on a profile.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AffiliationBadgeDto {
    /// The organization's handle.
    pub handle: String,
    /// The organization's display name.
    pub display_name: String,
    /// The organization's avatar image.
    pub avatar_url: Option<String>,
    /// The organization's avatar shape.
    pub avatar_shape: AvatarShape,
    /// The label the organization gave the account, like `Engineer`; may be empty.
    pub label: String,
}

/// One affiliation as its two sides see it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AffiliationDto {
    /// The affiliation id.
    pub id: Uuid,
    /// Pending or active.
    pub status: AffiliationStatus,
    /// The label, up to 30 characters.
    pub label: String,
    /// When the organization invited the account.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// The other side: the member for an organization, the organization for a member.
    pub user: UserRefDto,
}

/// `GET /affiliations`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AffiliationsDto {
    /// The viewer is an organization.
    pub org: bool,
    /// For an organization: active members first, then pending invitations.
    pub members: Vec<AffiliationDto>,
    /// The organization the viewer is affiliated with.
    pub current: Option<AffiliationDto>,
    /// Invitations waiting for the viewer's answer.
    pub invites: Vec<AffiliationDto>,
}

/// `POST /affiliations`: an organization invites an account.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AffiliateRequest {
    /// The account's handle.
    pub handle: String,
    /// A label, up to 30 characters.
    #[serde(default)]
    pub label: Option<String>,
}

/// `PATCH /affiliations/{id}`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchAffiliationRequest {
    /// The new label, up to 30 characters; empty clears it.
    pub label: String,
}

/// An entry of `GET /orgs/{handle}/affiliates`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AffiliateDto {
    /// The affiliated account.
    pub user: UserDto,
    /// Its label.
    pub label: String,
}

// ---- passkeys

/// A registered passkey.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasskeyDto {
    /// The passkey id (not the credential id).
    pub id: Uuid,
    /// The name, 1 to 60 characters; `Passkey` unless the user chose one.
    pub name: String,
    /// When it was registered.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// The last sign-in with it.
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_used_at: Option<OffsetDateTime>,
}

/// `GET /passkeys`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasskeysDto {
    /// The account's passkeys, newest first.
    pub passkeys: Vec<PasskeyDto>,
    /// The relying party id (the host of `PUBLIC_URL`); `null` when it is an IP address, which
    /// WebAuthn does not allow.
    pub rp_id: Option<String>,
}

/// A WebAuthn challenge. `publicKey` is the JSON form of `PublicKeyCredentialCreationOptions`
/// (registration) or `PublicKeyCredentialRequestOptions` (sign-in), the input of
/// `PublicKeyCredential.parseCreationOptionsFromJSON` and `parseRequestOptionsFromJSON`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasskeyOptionsDto {
    /// Send back with the credential. A challenge works once and for 5 minutes.
    pub challenge_id: Uuid,
    /// The options.
    pub public_key: serde_json::Value,
}

/// `POST /passkeys/register`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasskeyRegisterRequest {
    /// From the options.
    pub challenge_id: Uuid,
    /// The `toJSON()` of the `PublicKeyCredential` that `navigator.credentials.create` returned.
    pub credential: serde_json::Value,
    /// A name, 1 to 60 characters; `Passkey` when absent.
    #[serde(default)]
    pub name: Option<String>,
}

/// `POST /auth/passkey`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasskeyLoginRequest {
    /// From the options.
    pub challenge_id: Uuid,
    /// The `toJSON()` of the `PublicKeyCredential` that `navigator.credentials.get` returned.
    pub credential: serde_json::Value,
    /// Keep the accounts already signed in on this browser.
    #[serde(default)]
    pub add: bool,
}

/// `PATCH /passkeys/{id}`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenamePasskeyRequest {
    /// The new name, 1 to 60 characters.
    pub name: String,
}

// ------------------------------------------------------------------ federation

text_enum! {
    /// What a domain block does.
    DomainBlockSeverity {
        /// The instance's accounts stay, but their posts leave public timelines and search.
        Silence => "silence",
        /// Nothing is accepted from the instance and nothing is sent to it.
        Suspend => "suspend",
    }
}

/// A blocked instance.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainBlockDto {
    /// The domain, lower case, with a `:port` when it is not the default.
    pub domain: String,
    /// What the block does.
    pub severity: DomainBlockSeverity,
    /// Why; up to 200 characters.
    pub reason: String,
    /// Accounts of the instance this server knows.
    pub accounts: i32,
    /// When the block was made.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

/// `POST /admin/federation/blocks`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDomainBlockRequest {
    /// The domain, or a URL on it. Replaces an earlier block of the same domain.
    pub domain: String,
    /// What the block does.
    pub severity: DomainBlockSeverity,
    /// Why, up to 200 characters.
    #[serde(default)]
    pub reason: String,
}

/// `PATCH /admin/federation/blocks/{domain}`: only the fields present change.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchDomainBlockRequest {
    /// The new severity.
    #[serde(default)]
    pub severity: Option<DomainBlockSeverity>,
    /// The new reason, up to 200 characters.
    #[serde(default)]
    pub reason: Option<String>,
}

/// An instance that deliveries keep failing to.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryFailureDto {
    /// The inbox's domain.
    pub domain: String,
    /// Deliveries waiting for it.
    pub count: i32,
    /// The most attempts any of them has had (a delivery is dropped after 12).
    pub attempts: i32,
    /// What the latest failed attempt said, up to 300 characters.
    pub last_error: Option<String>,
    /// When the next attempt is due.
    #[serde(with = "time::serde::rfc3339")]
    pub next_attempt_at: OffsetDateTime,
}

/// The delivery queue.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FederationQueueDto {
    /// Deliveries waiting, due or not.
    pub count: i32,
    /// Deliveries due now.
    pub due: i32,
    /// When the oldest waiting delivery was queued; `null` when the queue is empty.
    #[serde(with = "time::serde::rfc3339::option")]
    pub oldest: Option<OffsetDateTime>,
    /// Instances with at least one failed attempt, most waiting first (at most 50).
    pub failing: Vec<DeliveryFailureDto>,
}

/// The instance actor, which signs fetches.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceActorDto {
    /// The actor's URL.
    pub id: String,
    /// The key id other servers see in a signature.
    pub key_id: String,
    /// SHA-256 of the public key's DER form, lower-case hex.
    pub fingerprint: String,
    /// The public key, PEM.
    pub public_key_pem: String,
}

/// `GET /admin/federation`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FederationDto {
    /// This instance's federation domain.
    pub domain: String,
    /// The delivery queue.
    pub queue: FederationQueueDto,
    /// The instance actor.
    pub actor: InstanceActorDto,
    /// Blocked instances, by domain.
    pub blocks: Vec<DomainBlockDto>,
    /// Instances this server knows an account of.
    pub known_domains: i32,
    /// Remote accounts this server knows.
    pub remote_accounts: i32,
    /// Remote posts this server stores.
    pub remote_posts: i32,
}

/// `POST /admin/federation/retry`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetryDto {
    /// Deliveries set to run now.
    pub requeued: i32,
}
