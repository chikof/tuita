//! Demo content for development: the body of `tuita seed`.
//!
//! Six accounts that cannot sign in (no password), 40 posts (32 top-level, 8 replies), follows,
//! likes and reposts. It only runs on a database without accounts.

use std::collections::HashMap;

use sqlx::{PgConnection, PgPool};
use time::{Duration, OffsetDateTime};
use tuita_core::text::parse_post;
use uuid::{NoContext, Timestamp, Uuid};

use crate::dto::{AvatarShape, Ink};

struct Demo {
    handle: &'static str,
    name: &'static str,
    bio: &'static str,
    location: &'static str,
    shape: AvatarShape,
    ink: Ink,
    verified: bool,
}

const USERS: [Demo; 6] = [
    Demo {
        handle: "lena",
        name: "Lena Fischer",
        bio: "Motion designer and this instance's host. Black, white and a little ink. \u{2726}",
        location: "",
        shape: AvatarShape::Square,
        ink: Ink::Bone,
        verified: true,
    },
    Demo {
        handle: "ada_builds",
        name: "Ada Moreno",
        bio: "Design engineer. I make pixels behave. #dither",
        location: "Lisbon",
        shape: AvatarShape::Squircle,
        ink: Ink::Phosphor,
        verified: false,
    },
    Demo {
        handle: "kofi",
        name: "Kofi Mensah",
        bio: "Photographer and trail runner. Mostly mountains.",
        location: "Accra",
        shape: AvatarShape::Circle,
        ink: Ink::Amber,
        verified: false,
    },
    Demo {
        handle: "mei_lin",
        name: "Mei Lin",
        bio: "Type nerd. Variable fonts changed my life.",
        location: "Taipei",
        shape: AvatarShape::Octagon,
        ink: Ink::Cyan,
        verified: false,
    },
    Demo {
        handle: "jonas",
        name: "Jonas Berg",
        bio: "Backend person. Postgres enjoyer. Coffee-powered.",
        location: "Oslo",
        shape: AvatarShape::Square,
        ink: Ink::Bone,
        verified: false,
    },
    Demo {
        handle: "priya",
        name: "Priya Raman",
        bio: "Product manager by day, synth noodler by night.",
        location: "Bengaluru",
        shape: AvatarShape::Circle,
        ink: Ink::Rose,
        verified: true,
    },
];

/// Top-level posts, oldest first: (author handle, text).
const POSTS: [(&str, &str); 32] = [
    (
        "lena",
        "Welcome to tuita \u{1f44b} A small social space with a 1-bit dither look. Say hi! #hello",
    ),
    (
        "ada_builds",
        "Hot take: a bayer matrix is the most honest gradient there is. #dither",
    ),
    (
        "kofi",
        "Sunrise run up the ridge this morning. Legs: destroyed. Soul: restored. #trailrunning",
    ),
    (
        "mei_lin",
        "Geist Pixel Square for big numbers is chef\u{2019}s kiss. #typography",
    ),
    (
        "jonas",
        "Shipped a whole backend today with one binary and one Postgres. Boring is beautiful. #rust",
    ),
    (
        "priya",
        "Roadmap meeting went 40 minutes over. We decided to have fewer meetings. \u{1f643}",
    ),
    (
        "ada_builds",
        "Two inks, two papers, zero gradients. Constraints are a feature. #dither",
    ),
    (
        "lena",
        "Tip: pick your ink in Settings > Appearance. Bone, phosphor, amber, cyan or rose.",
    ),
    (
        "kofi",
        "Golden hour in the valley. Sometimes the light does all the work. #photography",
    ),
    (
        "mei_lin",
        "Kerning is just personal space for letters. #typography",
    ),
    (
        "jonas",
        "Index-only scans are the quiet win of the week. #postgres",
    ),
    (
        "priya",
        "Synth patch of the day: slow attack, long release, a little chorus. #synth",
    ),
    (
        "ada_builds",
        "Static texture for chrome, animated fields only for the loud moments. #dither",
    ),
    (
        "kofi",
        "Packed for the weekend: 2 lenses, 1 tent, 0 regrets. #trailrunning",
    ),
    (
        "lena",
        "You can keep up to five accounts signed in on one browser. Look for the account chip.",
    ),
    (
        "mei_lin",
        "Monospace for meta, a pixel face for numbers, a clean sans for everything else. #typography",
    ),
    (
        "jonas",
        "Wrote the migration so the counters update in the same transaction. Sleeping well. #postgres",
    ),
    (
        "priya",
        "The best feature request is a screenshot with a red circle drawn on it.",
    ),
    (
        "ada_builds",
        "Contrast ratio 4.5:1 is not a suggestion. Test it, don\u{2019}t eyeball it. #a11y",
    ),
    (
        "kofi",
        "Fog in the valley, sun on the ridge. Worth the 4 am alarm. #photography",
    ),
    (
        "lena",
        "Following and For you are two different timelines on purpose. Try both.",
    ),
    (
        "mei_lin",
        "Variable fonts: one file, every weight. Hinting is the real boss fight. #typography",
    ),
    (
        "jonas",
        "Reminder: a rate limit is a feature, not an insult. #rust",
    ),
    (
        "priya",
        "Arpeggiator plus a slow filter sweep equals instant focus. #synth",
    ),
    (
        "ada_builds",
        "Reduced motion should still feel designed. Static is a style. #a11y",
    ),
    (
        "kofi",
        "New trail shoes survived their first 30 km. #trailrunning",
    ),
    (
        "lena",
        "Mentions work like you expect: @ada_builds says hi. Notifications arrive live.",
    ),
    (
        "mei_lin",
        "Small caps for labels, tabular figures for counters. #typography",
    ),
    (
        "jonas",
        "Cursor pagination with uuid v7 keys: no offsets, no duplicates. #postgres",
    ),
    (
        "priya",
        "Planning the week on paper first. The screen can wait. #focus",
    ),
    (
        "ada_builds",
        "Photos are never dithered. Everything around them can be. #dither",
    ),
    ("kofi", "Last light on the summit. #photography"),
];

/// Replies: (index into [`POSTS`] of the parent, author handle, text).
const REPLIES: [(usize, &str, &str); 8] = [
    (1, "mei_lin", "Agreed! A bayer matrix never lies."),
    (
        1,
        "jonas",
        "As long as it does not shift the layout, I am in.",
    ),
    (0, "kofi", "Hello! Love the look already."),
    (0, "priya", "Hi from Bengaluru \u{1f44b}"),
    (
        4,
        "ada_builds",
        "One binary, one database. That is the dream.",
    ),
    (8, "mei_lin", "That light is unreal."),
    (
        18,
        "kofi",
        "Testing contrast on a mountain is hard, but I tried.",
    ),
    (26, "ada_builds", "Right back at you @lena!"),
];

/// What [`run`] created, or that it did nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Created this many accounts and posts.
    Seeded {
        /// Accounts created.
        users: usize,
        /// Posts created, replies included.
        posts: usize,
    },
    /// The database already has accounts; nothing changed.
    AlreadyPopulated,
}

/// Fills an empty database with the demo content.
///
/// # Errors
///
/// Database failures; the transaction is rolled back, so nothing is half-seeded.
pub async fn run(db: &PgPool) -> Result<Outcome, sqlx::Error> {
    let mut tx = db.begin().await?;
    // Two concurrent seeds would both see an empty table; serialise them.
    sqlx::query("select pg_advisory_xact_lock(0x7475697461000002)")
        .execute(&mut *tx)
        .await?;
    let existing: i64 = sqlx::query_scalar(
        "select count(*) from users where handle <> 'tuita' or domain is not null",
    )
    .fetch_one(&mut *tx)
    .await?;
    if existing > 0 {
        return Ok(Outcome::AlreadyPopulated);
    }
    let now = OffsetDateTime::now_utc();
    let users = insert_users(&mut tx, now).await?;
    insert_follows(&mut tx, &users, now).await?;

    let mut post_ids: Vec<Uuid> = Vec::with_capacity(POSTS.len());
    let mut post_times: Vec<OffsetDateTime> = Vec::with_capacity(POSTS.len());
    for (n, (handle, text)) in POSTS.iter().enumerate() {
        let remaining = i32::try_from(POSTS.len() - n).unwrap_or(1);
        let at = now - Duration::minutes(i64::from(remaining) * 162);
        let id = insert_post(&mut tx, &users, handle, text, None, at).await?;
        post_ids.push(id);
        post_times.push(at);
    }
    for (n, (parent, handle, text)) in REPLIES.iter().enumerate() {
        let at = post_times[*parent] + Duration::minutes(20 + 7 * i64::try_from(n).unwrap_or(0));
        insert_post(&mut tx, &users, handle, text, Some(post_ids[*parent]), at).await?;
    }
    insert_reactions(&mut tx, &users, &post_ids, now).await?;
    recount(&mut tx).await?;
    tx.commit().await?;
    Ok(Outcome::Seeded {
        users: USERS.len(),
        posts: POSTS.len() + REPLIES.len(),
    })
}

/// A v7 uuid carrying `at`, so ids sort like the backdated `created_at`.
fn id_at(at: OffsetDateTime) -> Uuid {
    let secs = u64::try_from(at.unix_timestamp()).unwrap_or(0);
    Uuid::new_v7(Timestamp::from_unix(NoContext, secs, at.nanosecond()))
}

async fn insert_users(
    tx: &mut PgConnection,
    now: OffsetDateTime,
) -> Result<HashMap<&'static str, Uuid>, sqlx::Error> {
    let mut ids = HashMap::new();
    for (n, user) in USERS.iter().enumerate() {
        let at = now - Duration::days(90 - i64::try_from(n).unwrap_or(0));
        let id = id_at(at);
        sqlx::query(
            "insert into users (id, handle, display_name, bio, location, avatar_shape, ink, verified, created_at)
             values ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(id)
        .bind(user.handle)
        .bind(user.name)
        .bind(user.bio)
        .bind(user.location)
        .bind(user.shape.as_str())
        .bind(user.ink.as_str())
        .bind(user.verified)
        .bind(at)
        .execute(&mut *tx)
        .await?;
        ids.insert(user.handle, id);
    }
    Ok(ids)
}

async fn insert_follows(
    tx: &mut PgConnection,
    users: &HashMap<&'static str, Uuid>,
    now: OffsetDateTime,
) -> Result<(), sqlx::Error> {
    const FOLLOWS: [(&str, &[&str]); 6] = [
        ("lena", &["ada_builds", "mei_lin", "priya"]),
        ("ada_builds", &["lena", "mei_lin", "jonas"]),
        ("kofi", &["lena", "priya", "ada_builds"]),
        ("mei_lin", &["lena", "ada_builds"]),
        ("jonas", &["lena", "ada_builds", "priya"]),
        ("priya", &["lena", "jonas", "kofi"]),
    ];
    for (follower, followees) in FOLLOWS {
        for followee in followees {
            sqlx::query(
                "insert into follows (follower_id, followee_id, created_at) values ($1, $2, $3)",
            )
            .bind(users[follower])
            .bind(users[followee])
            .bind(now - Duration::days(30))
            .execute(&mut *tx)
            .await?;
        }
    }
    Ok(())
}

async fn insert_post(
    tx: &mut PgConnection,
    users: &HashMap<&'static str, Uuid>,
    handle: &str,
    text: &str,
    reply_to: Option<Uuid>,
    at: OffsetDateTime,
) -> Result<Uuid, sqlx::Error> {
    let id = id_at(at);
    sqlx::query(
        "insert into posts (id, author_id, text, reply_to_id, root_id, created_at)
         values ($1, $2, $3, $4, $4, $5)",
    )
    .bind(id)
    .bind(users[handle])
    .bind(text)
    .bind(reply_to)
    .bind(at)
    .execute(&mut *tx)
    .await?;
    let parsed = parse_post(text);
    for tag in parsed.hashtags() {
        sqlx::query("insert into post_tags (post_id, tag) values ($1, $2)")
            .bind(id)
            .bind(tag)
            .execute(&mut *tx)
            .await?;
    }
    for (mention, _) in parsed.mentions() {
        if let Some(user) = users.get(mention.to_lowercase().as_str()) {
            sqlx::query("insert into mentions (post_id, user_id) values ($1, $2)")
                .bind(id)
                .bind(user)
                .execute(&mut *tx)
                .await?;
        }
    }
    Ok(id)
}

/// A deterministic scatter of likes and reposts.
async fn insert_reactions(
    tx: &mut PgConnection,
    users: &HashMap<&'static str, Uuid>,
    posts: &[Uuid],
    now: OffsetDateTime,
) -> Result<(), sqlx::Error> {
    for (i, post) in posts.iter().enumerate() {
        for (j, user) in USERS.iter().enumerate() {
            let user = users[user.handle];
            if (i * 7 + j * 3) % 5 < 2 {
                sqlx::query(
                    "insert into likes (user_id, post_id, created_at) values ($1, $2, $3)
                     on conflict do nothing",
                )
                .bind(user)
                .bind(post)
                .bind(now)
                .execute(&mut *tx)
                .await?;
            }
            if (i * 5 + j * 11) % 13 == 0 {
                sqlx::query(
                    "insert into reposts (id, user_id, post_id, created_at) values ($1, $2, $3, $4)
                     on conflict do nothing",
                )
                .bind(id_at(
                    now - Duration::minutes(i64::try_from(i).unwrap_or(0) + 1),
                ))
                .bind(user)
                .bind(post)
                .bind(now - Duration::minutes(i64::try_from(i).unwrap_or(0) + 1))
                .execute(&mut *tx)
                .await?;
            }
        }
    }
    Ok(())
}

/// Sets every counter from the rows, so the seed need not track them while inserting.
async fn recount(tx: &mut PgConnection) -> Result<(), sqlx::Error> {
    sqlx::query(
        "update users u set
            followers_count = (select count(*) from follows where followee_id = u.id),
            following_count = (select count(*) from follows where follower_id = u.id),
            posts_count = (select count(*) from posts where author_id = u.id and deleted_at is null)",
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "update posts p set
            like_count = (select count(*) from likes where post_id = p.id),
            repost_count = (select count(*) from reposts where post_id = p.id),
            reply_count = (select count(*) from posts r where r.reply_to_id = p.id and r.deleted_at is null),
            quote_count = (select count(*) from posts q where q.quote_of_id = p.id and q.deleted_at is null)",
    )
    .execute(&mut *tx)
    .await?;
    Ok(())
}
