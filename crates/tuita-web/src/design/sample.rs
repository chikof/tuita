//! Fixtures for the gallery.
//!
//! Every value is a real `tuita_server::dto` type built from fixed inputs. Times are offsets from
//! "now", so relative labels read "2h" on every visit.

use time::{Duration, OffsetDateTime, macros::datetime};
use tuita_server::dto::{
    AccountDto, AdminReportDto, AdminStatsDto, AdminUserDto, AnnouncementDto, AnnouncementTone,
    AvatarShape, ConversationDto, DayStatsDto, DeliveryFailureDto, DmPolicy, DomainBlockDto,
    DomainBlockSeverity, EditVersionDto, FederationDto, FederationQueueDto, Ink, InstanceActorDto,
    LinkCardDto, ListDto, MeDto, MediaDto, MediaKind, MessageDto, NotifPrefsDto,
    NotificationGroupDto, NotificationKindDto, PollDto, PollInput, PollOptionDto, PostCounts,
    PostDto, PostEditsDto, PostViewer, QuoteDto, Relation, ReplyPolicy, ReplyToDto, ReportAction,
    ReportReason, ReportStatus, Role, ScheduledPostDto, ScheduledStatus, SegmentDto, SettingsDto,
    StatsPostsDto, StatsUsersDto, ThreadDto, UserCounts, UserDto, UserRefDto, Visibility,
};
use uuid::Uuid;

use crate::api::enc;

fn now() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}

pub(crate) fn hours_ago(h: i64) -> OffsetDateTime {
    now() - Duration::hours(h)
}

pub(crate) fn minutes_ago(m: i64) -> OffsetDateTime {
    now() - Duration::minutes(m)
}

pub(crate) fn hours_ahead(h: i64) -> OffsetDateTime {
    now() + Duration::hours(h)
}

/// A stable id for a name, so the same fixture always gets the same id.
pub(crate) fn id_of(name: &str) -> Uuid {
    Uuid::from_u128(
        0x7e57_0000_0000_0000_0000_0000_0000_0000
            + name.bytes().fold(0u128, |acc, b| {
                acc.wrapping_mul(131).wrapping_add(u128::from(b))
            }) % 0xffff_ffff_ffff,
    )
}

/// A colourful stand-in for a user photo (photos are never dithered).
#[must_use]
pub(crate) fn photo_url() -> String {
    const SVG: &str = "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 64 64'><defs><linearGradient id='g' x1='0' y1='0' x2='1' y2='1'><stop offset='0' stop-color='#f4a259'/><stop offset='1' stop-color='#5b8e7d'/></linearGradient></defs><rect width='64' height='64' fill='url(#g)'/><circle cx='32' cy='26' r='11' fill='#f2e8cf'/><path d='M10 64c2-18 14-24 22-24s20 6 22 24z' fill='#bc4b51'/></svg>";
    format!("data:image/svg+xml,{}", enc(SVG))
}

/// An account reference with default fields.
pub(crate) fn user_ref(handle: &str, display_name: &str) -> UserRefDto {
    UserRefDto {
        id: id_of(&format!("user-{handle}")),
        handle: handle.to_owned(),
        domain: None,
        display_name: display_name.to_owned(),
        avatar_url: None,
        avatar_shape: AvatarShape::Squircle,
        ink: Ink::Bone,
        verified: false,
        bot: false,
        emojis: Vec::new(),
    }
}

pub(crate) fn ada() -> UserRefDto {
    UserRefDto {
        verified: true,
        ink: Ink::Amber,
        ..user_ref("ada", "Ada Lovelace")
    }
}

pub(crate) fn bob() -> UserRefDto {
    UserRefDto {
        avatar_shape: AvatarShape::Octagon,
        ink: Ink::Cyan,
        ..user_ref("bob", "Bob")
    }
}

/// The signed-in account of the gallery.
pub(crate) fn me() -> UserRefDto {
    UserRefDto {
        avatar_shape: AvatarShape::Circle,
        ink: Ink::Phosphor,
        ..user_ref("sam", "Sam Sample")
    }
}

/// A full profile for `r`.
pub(crate) fn user_of(r: &UserRefDto) -> UserDto {
    UserDto {
        id: r.id,
        handle: r.handle.clone(),
        domain: r.domain.clone(),
        display_name: r.display_name.clone(),
        bio: "Builds tiny things in two inks. #dither @bob https://example.com".to_owned(),
        location: "Gdańsk".to_owned(),
        website: "https://example.com".to_owned(),
        avatar_url: r.avatar_url.clone(),
        banner_url: None,
        avatar_shape: r.avatar_shape,
        ink: r.ink,
        verified: r.verified,
        staff: None,
        org: false,
        bot: r.bot,
        bot_owner: None,
        affiliation: None,
        pinned_post_id: None,
        created_at: datetime!(2026-01-02 00:00 UTC),
        counts: UserCounts {
            followers: 1240,
            following: 48,
            posts: 320,
        },
        relation: Relation {
            following: false,
            follows_you: true,
            blocked: false,
            blocks_you: false,
            muted: false,
            follow_pending: false,
        },
        suspended: false,
        url: None,
        locked: false,
        emojis: r.emojis.clone(),
    }
}

/// A post by `author` with default fields: public, two hours old, nothing attached.
pub(crate) fn post_of(id: &str, author: UserRefDto, text: &str) -> PostDto {
    let post_id = id_of(id);
    PostDto {
        id: post_id,
        author,
        text: text.to_owned(),
        rich_text: vec![SegmentDto::Text {
            text: text.to_owned(),
        }],
        cw: String::new(),
        sensitive: false,
        created_at: hours_ago(2),
        edited_at: None,
        reply_policy: ReplyPolicy::Everyone,
        visibility: Visibility::Public,
        url: None,
        client: Some("linux".to_owned()),
        counts: PostCounts {
            likes: 12,
            reposts: 3,
            replies: 2,
            quotes: 1,
            views: 480,
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
        entry_id: post_id,
        reposted_by: None,
        emojis: Vec::new(),
    }
}

pub(crate) fn plain() -> PostDto {
    PostDto {
        rich_text: vec![
            SegmentDto::Text {
                text: "Two inks, one pixel grid. ".to_owned(),
            },
            SegmentDto::Hashtag {
                tag: "dither".to_owned(),
                text: "dither".to_owned(),
            },
            SegmentDto::Text {
                text: " with ".to_owned(),
            },
            SegmentDto::Mention {
                handle: "bob".to_owned(),
                domain: None,
            },
        ],
        ..post_of("sample-1", ada(), "Two inks, one pixel grid.")
    }
}

pub(crate) fn liked() -> PostDto {
    PostDto {
        viewer: PostViewer {
            liked: true,
            reposted: true,
            bookmarked: true,
        },
        ..post_of("sample-2", bob(), "Already liked and bookmarked.")
    }
}

pub(crate) fn warned() -> PostDto {
    PostDto {
        cw: "Spoilers".to_owned(),
        ..post_of("sample-3", bob(), "The ending, spoiled.")
    }
}

pub(crate) fn reposted() -> PostDto {
    PostDto {
        reposted_by: Some(bob()),
        ..post_of("sample-4", ada(), "Reposted by someone else.")
    }
}

pub(crate) fn pinned() -> PostDto {
    PostDto {
        pinned: true,
        ..post_of("sample-5", ada(), "Pinned to the profile.")
    }
}

pub(crate) fn reply() -> PostDto {
    PostDto {
        reply_to: Some(ReplyToDto {
            id: plain().id,
            handle: Some("ada".to_owned()),
        }),
        ..post_of("sample-6", bob(), "A reply.")
    }
}

pub(crate) fn quoting() -> PostDto {
    let quoted = plain();
    PostDto {
        quote: Some(QuoteDto {
            id: quoted.id,
            available: true,
            author: Some(ada()),
            text: quoted.text,
            rich_text: quoted.rich_text,
            cw: String::new(),
            sensitive: false,
            created_at: quoted.created_at,
            emojis: Vec::new(),
        }),
        ..post_of("sample-7", bob(), "Quoting that.")
    }
}

pub(crate) fn limited() -> PostDto {
    PostDto {
        can_reply: false,
        reply_policy: ReplyPolicy::Following,
        ..post_of("sample-8", ada(), "Replies are limited here.")
    }
}

pub(crate) fn thread() -> ThreadDto {
    ThreadDto {
        post: plain(),
        ancestors: Vec::new(),
        replies: vec![reply()],
        hidden_replies: Vec::new(),
    }
}

pub(crate) fn notifications() -> Vec<NotificationGroupDto> {
    vec![
        NotificationGroupDto {
            id: id_of("n1"),
            kind: NotificationKindDto::Like,
            read: false,
            created_at: hours_ago(1),
            actors: vec![ada(), bob()],
            actor_count: 5,
            post: Some(plain()),
        },
        NotificationGroupDto {
            id: id_of("n2"),
            kind: NotificationKindDto::Follow,
            read: true,
            created_at: hours_ago(5),
            actors: vec![bob()],
            actor_count: 1,
            post: None,
        },
        NotificationGroupDto {
            id: id_of("n3"),
            kind: NotificationKindDto::Reply,
            read: true,
            created_at: hours_ago(8),
            actors: vec![bob()],
            actor_count: 1,
            post: Some(reply()),
        },
    ]
}

pub(crate) fn sample_me() -> MeDto {
    MeDto {
        user: UserDto {
            relation: Relation {
                follows_you: false,
                ..user_of(&me()).relation
            },
            ..user_of(&me())
        },
        role: Role::User,
        federated_handle: "sam@tuita.example".to_owned(),
        settings: SettingsDto {
            dm_policy: DmPolicy::Everyone,
            discoverable: true,
            language: None,
            show_client: true,
            notif_prefs: NotifPrefsDto {
                like: true,
                repost: true,
                follow: true,
                reply: true,
                mention: true,
                quote: true,
            },
            muted_words: Vec::new(),
        },
    }
}

pub(crate) fn accounts() -> Vec<AccountDto> {
    vec![
        AccountDto {
            user: me(),
            active: true,
        },
        AccountDto {
            user: ada(),
            active: false,
        },
    ]
}

/// An SVG landscape standing in for a photo; it is a real, colourful image and is never dithered.
pub(crate) fn photo(id: &str, width: i32, height: i32, hue: i32, alt: &str) -> MediaDto {
    let (w, h) = (f64::from(width), f64::from(height));
    let svg = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 {width} {height}'><defs><linearGradient id='s' x1='0' y1='0' x2='0' y2='1'><stop offset='0' stop-color='hsl({hue} 80% 70%)'/><stop offset='1' stop-color='hsl({} 85% 45%)'/></linearGradient></defs><rect width='{width}' height='{height}' fill='url(#s)'/><circle cx='{}' cy='{}' r='{}' fill='hsl({} 90% 85%)'/><path d='M0 {height}V{}L{} {}L{} {}L{} {}L{width} {}V{height}Z' fill='hsl({} 45% 25%)'/></svg>",
        (hue + 40) % 360,
        w * 0.72,
        h * 0.28,
        h * 0.12,
        (hue + 180) % 360,
        h * 0.7,
        w * 0.25,
        h * 0.5,
        w * 0.45,
        h * 0.68,
        w * 0.7,
        h * 0.42,
        h * 0.72,
        (hue + 200) % 360,
    );
    MediaDto {
        id: id_of(id),
        kind: MediaKind::Image,
        url: format!("data:image/svg+xml,{}", enc(&svg)),
        poster_url: None,
        alt: alt.to_owned(),
        width: Some(width),
        height: Some(height),
    }
}

pub(crate) fn photos() -> Vec<MediaDto> {
    vec![
        photo("ph1", 1200, 800, 20, "A sunset over orange hills"),
        photo("ph2", 900, 900, 160, "Green mountains in fog"),
        photo("ph3", 1000, 700, 210, "A blue valley"),
        photo("ph4", 800, 1000, 300, "Purple dunes at dusk"),
    ]
}

/// A post by Ada with the first `n` photos.
pub(crate) fn with_media(n: usize) -> PostDto {
    PostDto {
        media: photos().into_iter().take(n).collect(),
        ..post_of(&format!("media-{n}"), ada(), "Photos, not dithered.")
    }
}

pub(crate) fn link_card() -> LinkCardDto {
    LinkCardDto {
        url: "https://www.example.com/articles/two-inks".to_owned(),
        title: "Two inks, one pixel grid: building a dithered social app".to_owned(),
        description: "How a 1-bit look scales from a toggle to a whole design system.".to_owned(),
        site: "Example Weekly".to_owned(),
        image: photos().into_iter().next().map(|p| p.url),
        large: true,
    }
}

fn poll_of(multiple: bool, closed: bool, voted: Vec<u8>) -> PollDto {
    PollDto {
        options: [("Bayer 4×4", 14), ("Blue noise", 9), ("Floyd–Steinberg", 5)]
            .into_iter()
            .map(|(text, votes)| PollOptionDto {
                text: text.to_owned(),
                votes,
            })
            .collect(),
        total: 28,
        voters: 28,
        multiple,
        ends_at: if closed {
            hours_ago(1)
        } else {
            hours_ahead(20)
        },
        closed,
        voted,
    }
}

pub(crate) fn poll_open() -> PostDto {
    PostDto {
        poll: Some(poll_of(false, false, Vec::new())),
        ..post_of("poll-open", ada(), "Which dither is best?")
    }
}

pub(crate) fn poll_multi() -> PostDto {
    PostDto {
        poll: Some(poll_of(true, false, Vec::new())),
        ..post_of("poll-multi", bob(), "Pick any that apply.")
    }
}

pub(crate) fn poll_voted() -> PostDto {
    PostDto {
        poll: Some(poll_of(false, false, vec![1])),
        ..post_of("poll-voted", ada(), "You voted on this one.")
    }
}

pub(crate) fn poll_closed() -> PostDto {
    PostDto {
        poll: Some(poll_of(false, true, Vec::new())),
        ..post_of("poll-closed", bob(), "This poll ended.")
    }
}

fn message(id: &str, text: &str, mine: bool, ago: i64, read_ago: Option<i64>) -> MessageDto {
    MessageDto {
        id: id_of(id),
        text: text.to_owned(),
        ciphertext: None,
        created_at: minutes_ago(ago),
        mine,
        read_at: read_ago.map(minutes_ago),
        emojis: Vec::new(),
    }
}

pub(crate) fn chat() -> Vec<MessageDto> {
    vec![
        message("c1", "Did you see the new dither engine?", false, 90, None),
        message(
            "c2",
            "Yes! The Bayer masks look great at 2 px.",
            true,
            80,
            Some(79),
        ),
        message("c3", "Docs: https://example.com/dither", true, 79, Some(78)),
        message("c4", "Nice. Photos stay untouched, right?", false, 10, None),
        message("c5", "Always.", true, 5, Some(4)),
    ]
}

/// A sealed conversation: the messages of `chat()` (1, 2, 4, 5) as ciphertext under new ids.
pub(crate) fn sealed_chat() -> Vec<MessageDto> {
    let chat = chat();
    [(0, "s1"), (1, "s2"), (3, "s3"), (4, "s4")]
        .into_iter()
        .map(|(index, id)| MessageDto {
            id: id_of(id),
            text: String::new(),
            ciphertext: Some("v1.sample.sample.sample.sample.sample".to_owned()),
            ..chat[index].clone()
        })
        .collect()
}

pub(crate) const SAFETY_OLD: &str =
    "3471 0928 5516 7740 1893 2206 6159 4012 8847 3305 7126 9581 0264 4738 1950";
pub(crate) const SAFETY_NEW: &str =
    "8120 4457 0693 2988 1574 3361 9042 7715 2086 5439 6602 1327 8851 0476 3194";

pub(crate) fn conversations() -> Vec<ConversationDto> {
    let chat = chat();
    vec![
        ConversationDto {
            user: ada(),
            unread: 2,
            last: chat[3].clone(),
            peer_key: None,
        },
        ConversationDto {
            user: bob(),
            unread: 0,
            last: chat[4].clone(),
            peer_key: None,
        },
    ]
}

pub(crate) fn sample_list() -> ListDto {
    ListDto {
        id: id_of("list-1"),
        name: "Pixel people".to_owned(),
        description: "Artists who work in 1-bit.".to_owned(),
        private: false,
        created_at: datetime!(2026-02-01 00:00 UTC),
        owner: me(),
        is_owner: true,
        members: 12,
        has_member: None,
    }
}

pub(crate) fn private_list() -> ListDto {
    ListDto {
        id: id_of("list-2"),
        name: "Close friends".to_owned(),
        description: String::new(),
        private: true,
        members: 1,
        ..sample_list()
    }
}

pub(crate) fn scheduled() -> Vec<ScheduledPostDto> {
    vec![
        ScheduledPostDto {
            id: id_of("sc1"),
            publish_at: hours_ahead(26),
            status: ScheduledStatus::Pending,
            post_id: None,
            error: None,
            created_at: minutes_ago(30),
            text: "Tomorrow morning: a new palette.".to_owned(),
            cw: String::new(),
            sensitive: false,
            reply_policy: ReplyPolicy::Everyone,
            media: photos().into_iter().skip(1).take(1).collect(),
            poll: None,
            quote_of: None,
        },
        ScheduledPostDto {
            id: id_of("sc2"),
            publish_at: minutes_ago(120),
            status: ScheduledStatus::Failed,
            post_id: None,
            error: Some("You can only attach up to 4 images.".to_owned()),
            created_at: minutes_ago(600),
            text: "This one failed.".to_owned(),
            cw: "Heads up".to_owned(),
            sensitive: false,
            reply_policy: ReplyPolicy::Everyone,
            media: Vec::new(),
            poll: Some(PollInput {
                options: vec!["a".to_owned(), "b".to_owned()],
                multiple: false,
                duration_hours: 24,
            }),
            quote_of: None,
        },
    ]
}

pub(crate) fn history() -> PostEditsDto {
    PostEditsDto {
        current: EditVersionDto {
            text: "Two inks, one pixel grid. Now with polls.".to_owned(),
            cw: String::new(),
            at: minutes_ago(5),
        },
        edits: vec![
            EditVersionDto {
                text: "Two inks, one pixel grid.".to_owned(),
                cw: "Design".to_owned(),
                at: minutes_ago(20),
            },
            EditVersionDto {
                text: "Two inks.".to_owned(),
                cw: String::new(),
                at: minutes_ago(40),
            },
        ],
    }
}

/// A remote account on `fedi.example`.
pub(crate) fn mara() -> UserRefDto {
    UserRefDto {
        domain: Some("fedi.example".to_owned()),
        avatar_shape: AvatarShape::Circle,
        ink: Ink::Rose,
        ..user_ref("mara", "Mara Okafor")
    }
}

pub(crate) fn remote_post() -> PostDto {
    PostDto {
        url: Some("https://fedi.example/@mara/112233445566".to_owned()),
        ..post_of(
            "remote-1",
            mara(),
            "Hello from another instance. Federated, not cloned.",
        )
    }
}

pub(crate) fn unlisted_post() -> PostDto {
    PostDto {
        visibility: Visibility::Unlisted,
        ..post_of(
            "unlisted-1",
            ada(),
            "Unlisted: out of the public timelines.",
        )
    }
}

pub(crate) fn followers_post() -> PostDto {
    PostDto {
        visibility: Visibility::Followers,
        url: Some("https://fedi.example/@mara/112233445577".to_owned()),
        ..post_of("followers-1", mara(), "Followers only, shared from afar.")
    }
}

pub(crate) fn direct_post() -> PostDto {
    PostDto {
        visibility: Visibility::Direct,
        url: Some("https://fedi.example/@mara/112233445588".to_owned()),
        ..post_of(
            "direct-1",
            mara(),
            "@sam a direct note, nobody else sees it.",
        )
    }
}

fn remote_base(url: bool) -> UserDto {
    UserDto {
        location: String::new(),
        website: String::new(),
        url: url.then(|| "https://fedi.example/@mara".to_owned()),
        relation: Relation {
            following: false,
            follows_you: false,
            blocked: false,
            blocks_you: false,
            muted: false,
            follow_pending: false,
        },
        ..user_of(&mara())
    }
}

pub(crate) fn remote_user() -> UserDto {
    UserDto {
        bio: "Writes about small networks. @ada is a friend.".to_owned(),
        counts: UserCounts {
            followers: 318,
            following: 204,
            posts: 1520,
        },
        ..remote_base(true)
    }
}

pub(crate) fn remote_locked() -> UserDto {
    UserDto {
        locked: true,
        ..remote_base(true)
    }
}

pub(crate) fn remote_pending() -> UserDto {
    UserDto {
        locked: true,
        relation: Relation {
            follow_pending: true,
            ..remote_base(true).relation
        },
        ..remote_base(true)
    }
}

pub(crate) fn remote_followed() -> UserDto {
    UserDto {
        relation: Relation {
            following: true,
            ..remote_base(true).relation
        },
        ..remote_base(true)
    }
}

pub(crate) fn remote_notifications() -> Vec<NotificationGroupDto> {
    vec![
        NotificationGroupDto {
            id: id_of("rn1"),
            kind: NotificationKindDto::Follow,
            read: false,
            created_at: hours_ago(2),
            actors: vec![mara()],
            actor_count: 1,
            post: None,
        },
        NotificationGroupDto {
            id: id_of("rn2"),
            kind: NotificationKindDto::Like,
            read: true,
            created_at: hours_ago(6),
            actors: vec![mara(), bob()],
            actor_count: 2,
            post: Some(plain()),
        },
    ]
}

// ---- admin ------------------------------------------------------------------------------

const POSTS: [i32; 14] = [42, 58, 51, 77, 64, 12, 9, 48, 66, 81, 95, 72, 18, 33];
const SIGNUPS: [i32; 14] = [3, 5, 2, 8, 6, 1, 0, 4, 7, 9, 12, 6, 2, 5];
const REPORTS: [i32; 14] = [0, 1, 0, 2, 1, 0, 0, 0, 3, 1, 0, 2, 0, 1];
const LIKES: [i32; 14] = [
    120, 150, 98, 210, 180, 40, 22, 130, 190, 240, 310, 205, 60, 90,
];

fn day(ago: i64) -> String {
    (now() - Duration::days(ago)).date().to_string()
}

/// A believable week: quiet nights, busy evenings, calm weekends (Monday first).
fn heat(weekday: i32) -> Vec<i32> {
    (0..24)
        .map(|hour| {
            let daytime = (f64::from(hour - 6) / 18.0 * std::f64::consts::PI)
                .sin()
                .max(0.0);
            let weekend = if weekday >= 5 { 0.55 } else { 1.0 };
            (daytime * 30.0 * weekend).round() as i32 + (hour * 7 + weekday * 3) % 5
        })
        .collect()
}

pub(crate) fn stats() -> AdminStatsDto {
    AdminStatsDto {
        users: StatsUsersDto {
            local: 12480,
            demo: 4,
            suspended: 7,
            active7d: 3120,
        },
        posts: StatsPostsDto {
            total: 183_204,
            last24h: 1_240,
        },
        reports_open: 3,
        days: (0..14)
            .map(|i| DayStatsDto {
                day: day(13 - i as i64),
                posts: POSTS[i],
                signups: SIGNUPS[i],
                likes: LIKES[i],
                reports: REPORTS[i],
            })
            .collect(),
        heatmap: (0..7).map(heat).collect(),
    }
}

pub(crate) fn quiet_stats() -> AdminStatsDto {
    AdminStatsDto {
        users: StatsUsersDto {
            local: 1,
            demo: 0,
            suspended: 0,
            active7d: 1,
        },
        posts: StatsPostsDto {
            total: 0,
            last24h: 0,
        },
        reports_open: 0,
        days: (0..14)
            .map(|i| DayStatsDto {
                day: day(13 - i),
                posts: 0,
                signups: 0,
                likes: 0,
                reports: 0,
            })
            .collect(),
        heatmap: vec![vec![0; 24]; 7],
    }
}

pub(crate) fn announcements() -> Vec<AnnouncementDto> {
    [
        (
            "ann-info",
            "Maintenance on Saturday at 02:00 UTC. The site may be slow for ten minutes.",
            AnnouncementTone::Info,
            "/terms",
        ),
        (
            "ann-success",
            "Passkeys are here: sign in with your fingerprint or screen lock.",
            AnnouncementTone::Success,
            "",
        ),
        (
            "ann-warning",
            "A sign-in problem is being fixed. Some people see an error on the first try.",
            AnnouncementTone::Warning,
            "https://example.org/status",
        ),
    ]
    .into_iter()
    .map(|(id, text, tone, link)| AnnouncementDto {
        id: id_of(id),
        text: text.to_owned(),
        tone,
        link: link.to_owned(),
        created_at: now(),
    })
    .collect()
}

pub(crate) fn admin_user(r: &UserRefDto) -> AdminUserDto {
    AdminUserDto {
        user: user_of(r),
        role: Role::User,
        deleted: false,
        open_reports: 0,
        last_active_at: Some(hours_ago(3)),
    }
}

pub(crate) fn report_open() -> AdminReportDto {
    AdminReportDto {
        id: id_of("report-1"),
        reason: ReportReason::Spam,
        note: "The same link in a dozen replies today.".to_owned(),
        status: ReportStatus::Open,
        resolution: None,
        created_at: hours_ago(2),
        resolved_at: None,
        reporter: Some(bob()),
        target: ada(),
        target_suspended: false,
        resolved_by: None,
        post: Some(plain()),
        same_target_open: 2,
    }
}

pub(crate) fn report_account() -> AdminReportDto {
    AdminReportDto {
        id: id_of("report-2"),
        reason: ReportReason::Impersonation,
        note: String::new(),
        post: None,
        same_target_open: 1,
        ..report_open()
    }
}

pub(crate) fn report_resolved() -> AdminReportDto {
    AdminReportDto {
        id: id_of("report-3"),
        reason: ReportReason::Harassment,
        status: ReportStatus::Resolved,
        resolution: Some(ReportAction::DeleteAndSuspend),
        resolved_at: Some(hours_ago(1)),
        resolved_by: Some(me()),
        target_suspended: true,
        same_target_open: 1,
        ..report_open()
    }
}

const FINGERPRINT: &str = "3f9ac2d17b04e86a5c1f2907bd38e4a6710cab52f8d93e1647b0a5d2c8e91f34";

pub(crate) fn federation_actor() -> InstanceActorDto {
    InstanceActorDto {
        id: "https://tuita.example/actor".to_owned(),
        key_id: "https://tuita.example/actor#main-key".to_owned(),
        fingerprint: FINGERPRINT.to_owned(),
        public_key_pem: "-----BEGIN PUBLIC KEY-----\n-----END PUBLIC KEY-----\n".to_owned(),
    }
}

pub(crate) fn queue_empty() -> FederationQueueDto {
    FederationQueueDto {
        count: 0,
        due: 0,
        oldest: None,
        failing: Vec::new(),
    }
}

pub(crate) fn queue_busy() -> FederationQueueDto {
    FederationQueueDto {
        count: 14,
        due: 3,
        oldest: Some(hours_ago(5)),
        failing: vec![
            DeliveryFailureDto {
                domain: "down.example".to_owned(),
                count: 9,
                attempts: 4,
                last_error: Some("connection refused (os error 111)".to_owned()),
                next_attempt_at: now() + Duration::minutes(20),
            },
            DeliveryFailureDto {
                domain: "slow.social:8443".to_owned(),
                count: 1,
                attempts: 1,
                last_error: None,
                next_attempt_at: now() + Duration::minutes(1),
            },
        ],
    }
}

pub(crate) fn domain_block_silence() -> DomainBlockDto {
    DomainBlockDto {
        domain: "noisy.example".to_owned(),
        severity: DomainBlockSeverity::Silence,
        reason: "Floods the public timeline".to_owned(),
        accounts: 12,
        created_at: datetime!(2026-10-01 09:30 UTC),
    }
}

pub(crate) fn domain_block_suspend() -> DomainBlockDto {
    DomainBlockDto {
        domain: "spam.example".to_owned(),
        severity: DomainBlockSeverity::Suspend,
        reason: String::new(),
        accounts: 1,
        created_at: datetime!(2026-10-20 18:05 UTC),
    }
}

pub(crate) fn federation() -> FederationDto {
    FederationDto {
        domain: "tuita.example".to_owned(),
        queue: queue_busy(),
        actor: federation_actor(),
        blocks: vec![domain_block_silence(), domain_block_suspend()],
        known_domains: 38,
        remote_accounts: 412,
        remote_posts: 5280,
    }
}

pub(crate) fn federation_empty() -> FederationDto {
    FederationDto {
        domain: "tuita.example".to_owned(),
        queue: queue_empty(),
        actor: federation_actor(),
        blocks: Vec::new(),
        known_domains: 0,
        remote_accounts: 0,
        remote_posts: 0,
    }
}

/// Two photos behind the sensitive cover (the second post of the media grid demo).
pub(crate) fn media_sensitive() -> PostDto {
    let id = id_of("media-sensitive");
    PostDto {
        id,
        entry_id: id,
        sensitive: true,
        ..with_media(2)
    }
}

/// What the lightbox demo opens: four photos on a post nobody can reply to.
pub(crate) fn lightbox_thread() -> ThreadDto {
    ThreadDto {
        post: PostDto {
            can_reply: false,
            ..with_media(4)
        },
        ancestors: Vec::new(),
        replies: Vec::new(),
        hidden_replies: Vec::new(),
    }
}
