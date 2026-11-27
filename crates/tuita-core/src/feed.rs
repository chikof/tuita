//! The "For you" ranking.
//!
//! 1. Candidates are recent top-level posts: the last [`CANDIDATE_WINDOW`], or the newest
//!    [`CANDIDATE_LIMIT`] of any age when fewer than 60 are that recent (a quiet instance).
//! 2. The score is engagement × affinity × freshness. Likes, reposts, replies, quotes and views
//!    add up to the engagement. Following the author, followed people liking or reposting the
//!    post, and recent interaction with the author boost it. Media and quotes add a little, a
//!    brand-new post gets a boost, and a post the viewer already liked or reposted is cut down.
//! 3. A diversity damper re-picks greedily from the best 600, so each further post by an author
//!    already shown recently counts 0.55 times as much. The author's tally falls by one after
//!    every ten posts.
//!
//! The functions are pure. The server loads [`Candidate`]s and a [`FeedCtx`] with a few queries,
//! calls [`rank`], and stores the ids as the viewer's snapshot for [`SNAPSHOT_TTL`] so later pages
//! continue the same order.
//!
//! ```
//! use std::collections::{HashMap, HashSet};
//! use time::{Duration, OffsetDateTime};
//! use tuita_core::feed::{Candidate, FeedCtx, rank};
//! use uuid::Uuid;
//!
//! let now = OffsetDateTime::UNIX_EPOCH + Duration::days(100);
//! let post = |n: u128, likes: i32, age: Duration| Candidate {
//!     id: Uuid::from_u128(n),
//!     author_id: Uuid::from_u128(n),
//!     created_at: now - age,
//!     views: 0,
//!     likes,
//!     reposts: 0,
//!     replies: 0,
//!     quotes: 0,
//!     followed_likes: 0,
//!     followed_reposts: 0,
//!     has_media: false,
//!     is_quote: false,
//!     engaged: false,
//! };
//! let ctx = FeedCtx {
//!     me: Uuid::from_u128(999),
//!     now,
//!     followed: HashSet::new(),
//!     hidden: HashSet::new(),
//!     affinity: HashMap::new(),
//! };
//! let ranked = rank(&[post(1, 0, Duration::hours(1)), post(2, 50, Duration::hours(1))], &ctx);
//! assert_eq!(ranked, [Uuid::from_u128(2), Uuid::from_u128(1)]);
//! ```

use std::collections::{HashMap, HashSet};

use time::{Duration, OffsetDateTime};
use uuid::Uuid;

/// Posts older than this are not candidates, unless the instance is quiet.
pub const CANDIDATE_WINDOW: Duration = Duration::days(14);
/// How many posts the server loads: the newest top-level posts of any age, by `created_at`
/// descending. [`rank`] applies [`CANDIDATE_WINDOW`] itself.
pub const CANDIDATE_LIMIT: usize = 1500;
/// How far back the server counts a viewer's likes, reposts and replies for [`FeedCtx::affinity`].
pub const AFFINITY_WINDOW: Duration = Duration::days(30);
/// How long a ranked list serves later pages before page one re-ranks.
pub const SNAPSHOT_TTL: Duration = Duration::minutes(20);

/// Fewer candidates than this inside the window means the window is dropped.
const MIN_WINDOWED: usize = 60;
/// How many top-scored posts enter the diversity pass.
const POOL_SIZE: usize = 600;
/// How far down the pool the diversity pass looks for each pick.
const LOOKAHEAD: usize = 40;
/// Weight of a view, and the most views that count.
const VIEW_WEIGHT: f64 = 0.02;
const MAX_VIEWS: i64 = 500;
/// The most followed-people engagement that boosts a post.
const MAX_SOCIAL: i64 = 10;
/// Multiplier for each earlier post by the same author in the recent picks.
const REPEAT_DAMPING: f64 = 0.55;

/// One post as the candidate query returns it. Counts are the post's counters; the server folds
/// federated counts in with `greatest(...)` (M4).
#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    /// The post.
    pub id: Uuid,
    /// Who wrote it.
    pub author_id: Uuid,
    /// When it was posted.
    pub created_at: OffsetDateTime,
    /// How often it was opened. Only the first 500 count.
    pub views: i64,
    /// Likes.
    pub likes: i32,
    /// Reposts.
    pub reposts: i32,
    /// Replies that are not deleted.
    pub replies: i32,
    /// Quote posts that are not deleted.
    pub quotes: i32,
    /// How many people the viewer follows liked it.
    pub followed_likes: i32,
    /// How many people the viewer follows reposted it. Each counts double.
    pub followed_reposts: i32,
    /// Whether it has an image or a video.
    pub has_media: bool,
    /// Whether it quotes another post.
    pub is_quote: bool,
    /// Whether the viewer already liked or reposted it.
    pub engaged: bool,
}

// 88: two UUIDs (32), the timestamp (16), views (8), six i32 counters (24), three bools and padding.
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<Candidate>() == 88);

/// Who is looking, and what they relate to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedCtx {
    /// The viewer.
    pub me: Uuid,
    /// The current time, from the caller's clock.
    pub now: OffsetDateTime,
    /// The accounts the viewer follows.
    pub followed: HashSet<Uuid>,
    /// Accounts whose posts the viewer never sees: blocked and muted.
    pub hidden: HashSet<Uuid>,
    /// Per author, how many posts of theirs the viewer liked or reposted, or replied to, in the
    /// last [`AFFINITY_WINDOW`].
    pub affinity: HashMap<Uuid, u32>,
}

/// The score of one candidate: higher shows earlier. It ignores the window, the hidden accounts
/// and the diversity damper, which [`rank`] applies.
#[must_use]
pub fn score(candidate: &Candidate, ctx: &FeedCtx) -> f32 {
    score_f64(candidate, ctx) as f32
}

// Ranks in f64 so near ties order stably.
fn score_f64(c: &Candidate, ctx: &FeedCtx) -> f64 {
    let age_hours = ((ctx.now - c.created_at).as_seconds_f64() / 3600.0).max(0.0);
    let engagement = 1.0
        + f64::from(c.likes)
        + 2.0 * f64::from(c.reposts)
        + 2.5 * f64::from(c.replies)
        + 1.5 * f64::from(c.quotes)
        + c.views.min(MAX_VIEWS) as f64 * VIEW_WEIGHT;
    let social = (i64::from(c.followed_likes) + 2 * i64::from(c.followed_reposts)).min(MAX_SOCIAL);
    let affinity = ctx.affinity.get(&c.author_id).copied().unwrap_or(0);

    let mut boost = 1.0;
    if c.author_id == ctx.me {
        boost *= 0.7; // the viewer has seen their own posts
    } else if ctx.followed.contains(&c.author_id) {
        boost *= 2.2;
    }
    boost *= 1.0 + social as f64 * 0.35;
    boost *= 1.0 + f64::from(affinity).ln_1p() * 0.45;
    if c.has_media {
        boost *= 1.1;
    }
    if c.is_quote {
        boost *= 1.05;
    }
    if c.engaged {
        boost *= 0.45;
    }
    if age_hours < 1.0 {
        boost *= 1.4; // give brand-new posts a chance
    }
    let freshness = (age_hours + 2.0).powf(-1.35);
    engagement.ln_1p() * 3.0 * boost * freshness + boost * freshness
}

/// Ranks `candidates` for the viewer and returns the post ids, best first.
///
/// `candidates` are the newest top-level posts, newest first (see [`CANDIDATE_LIMIT`]). Posts by
/// hidden accounts are dropped.
#[must_use]
pub fn rank(candidates: &[Candidate], ctx: &FeedCtx) -> Vec<Uuid> {
    let in_window = |c: &Candidate| ctx.now - c.created_at < CANDIDATE_WINDOW;
    let windowed = candidates.iter().filter(|c| in_window(c)).count() >= MIN_WINDOWED;

    let mut scored: Vec<Scored> = candidates
        .iter()
        .filter(|c| (!windowed || in_window(c)) && !ctx.hidden.contains(&c.author_id))
        .map(|c| Scored {
            id: c.id,
            author: c.author_id,
            score: score_f64(c, ctx),
        })
        .collect();
    scored.sort_by(|a, b| b.score.total_cmp(&a.score));
    scored.truncate(POOL_SIZE);
    diversify(scored)
}

struct Scored {
    id: Uuid,
    author: Uuid,
    score: f64,
}

/// Re-picks greedily from the front of the pool, damping each author by how often they were
/// picked recently. Authors are forgotten gradually, so they can come back further down.
fn diversify(mut pool: Vec<Scored>) -> Vec<Uuid> {
    let mut out = Vec::with_capacity(pool.len());
    let mut recent: HashMap<Uuid, u32> = HashMap::new();
    while !pool.is_empty() {
        let mut best = 0;
        let mut best_score = -1.0;
        for (i, post) in pool.iter().take(LOOKAHEAD).enumerate() {
            let seen = recent.get(&post.author).copied().unwrap_or(0);
            let damped = post.score * REPEAT_DAMPING.powf(f64::from(seen));
            if damped > best_score {
                best_score = damped;
                best = i;
            }
        }
        let pick = pool.remove(best);
        out.push(pick.id);
        *recent.entry(pick.author).or_insert(0) += 1;
        if out.len() % 10 == 0 {
            for tally in recent.values_mut() {
                *tally = tally.saturating_sub(1);
            }
        }
    }
    out
}

#[cfg(test)]
mod reference;

#[cfg(test)]
mod test {
    use super::*;

    /// xorshift32, the same generator as the script that recorded the expected values.
    struct Rng(u32);

    impl Rng {
        fn next(&mut self, bound: u32) -> u32 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 17;
            self.0 ^= self.0 << 5;
            self.0 % bound
        }
    }

    fn now() -> OffsetDateTime {
        OffsetDateTime::UNIX_EPOCH + Duration::days(20_000)
    }

    fn uuid(n: u32) -> Uuid {
        Uuid::from_u128(u128::from(n))
    }

    /// The fixture of `reference`: `count` posts by 12 authors, `max_age_days` old at most, newest
    /// first. A post's id is its index in the fixture.
    fn fixture(seed: u32, count: u32, max_age_days: u32) -> (Vec<Candidate>, FeedCtx) {
        let mut rng = Rng(seed);
        let mut posts: Vec<(u32, Candidate)> = (0..count)
            .map(|i| {
                let age = rng.next(max_age_days * 86_400);
                let candidate = Candidate {
                    id: uuid(i),
                    author_id: uuid(1000 + rng.next(12)),
                    created_at: now() - Duration::seconds(i64::from(age)),
                    likes: rng.next(50) as i32,
                    reposts: rng.next(10) as i32,
                    replies: rng.next(12) as i32,
                    quotes: rng.next(5) as i32,
                    views: i64::from(rng.next(800)),
                    followed_likes: rng.next(6) as i32,
                    followed_reposts: rng.next(4) as i32,
                    has_media: rng.next(3) == 0,
                    is_quote: rng.next(5) == 0,
                    engaged: rng.next(6) == 0,
                };
                (age, candidate)
            })
            .collect();
        posts.sort_by_key(|(age, _)| *age);
        let ctx = FeedCtx {
            me: uuid(1000),
            now: now(),
            followed: (0..12)
                .filter(|n| n % 3 == 0)
                .map(|n| uuid(1000 + n))
                .collect(),
            hidden: HashSet::from([uuid(1005)]),
            affinity: (0..12).map(|n| (uuid(1000 + n), n * 2)).collect(),
        };
        (posts.into_iter().map(|(_, c)| c).collect(), ctx)
    }

    fn indexes(ids: &[Uuid]) -> Vec<u128> {
        ids.iter().map(Uuid::as_u128).collect()
    }

    #[test]
    fn ranks_reference_values() {
        let (posts, ctx) = fixture(2024, 150, 20);
        assert_eq!(indexes(&rank(&posts, &ctx)), reference::BUSY);
    }

    #[test]
    fn quiet_instance_looks_further_back() {
        let (posts, ctx) = fixture(77, 150, 400);
        let in_window = posts
            .iter()
            .filter(|c| ctx.now - c.created_at < CANDIDATE_WINDOW)
            .count();
        assert!(
            in_window < MIN_WINDOWED,
            "fixture should be quiet, has {in_window} recent"
        );
        assert_eq!(indexes(&rank(&posts, &ctx)), reference::QUIET);
    }

    #[test]
    fn scores_reference_values() {
        let (posts, ctx) = fixture(2024, 150, 20);
        for (index, expected) in reference::SCORES {
            let post = posts
                .iter()
                .find(|c| c.id == uuid(*index))
                .expect("fixture post");
            let got = f64::from(score(post, &ctx));
            assert!(
                (got - expected).abs() <= expected * 1e-6,
                "post {index}: {got} vs {expected}"
            );
        }
    }

    #[test]
    fn window_keeps_old_posts_out_when_busy() {
        let (posts, ctx) = fixture(2024, 150, 20);
        let ranked = rank(&posts, &ctx);
        let old = |id: &Uuid| {
            let post = posts
                .iter()
                .find(|c| c.id == *id)
                .expect("ranked post exists");
            ctx.now - post.created_at >= CANDIDATE_WINDOW
        };
        assert!(!ranked.iter().any(old));
    }

    #[test]
    fn drops_hidden_authors() {
        let (posts, ctx) = fixture(2024, 150, 20);
        let ranked = rank(&posts, &ctx);
        let hidden = |id: &Uuid| {
            posts
                .iter()
                .any(|c| c.id == *id && ctx.hidden.contains(&c.author_id))
        };
        assert!(!ranked.iter().any(hidden));
        assert!(
            posts.iter().any(|c| ctx.hidden.contains(&c.author_id)),
            "fixture has hidden posts"
        );
    }

    #[test]
    fn empty_input_ranks_nothing() {
        let (_, ctx) = fixture(1, 0, 1);
        assert!(rank(&[], &ctx).is_empty());
    }

    #[test]
    fn viewer_prefers_followed_and_unengaged_posts() {
        let (posts, ctx) = fixture(5, 1, 1);
        let base = Candidate {
            engaged: false,
            has_media: false,
            is_quote: false,
            ..posts[0].clone()
        };
        let followed_author = *ctx
            .followed
            .iter()
            .find(|a| **a != ctx.me)
            .expect("followed author");
        let other = Candidate {
            author_id: uuid(9999),
            ..base.clone()
        };
        let followed = Candidate {
            author_id: followed_author,
            ..base.clone()
        };
        let engaged = Candidate {
            engaged: true,
            ..other.clone()
        };
        assert!(score(&followed, &ctx) > score(&other, &ctx));
        assert!(score(&engaged, &ctx) < score(&other, &ctx));
    }

    #[test]
    fn damper_spreads_one_author() {
        let (mut posts, ctx) = fixture(3, 30, 3);
        for post in &mut posts {
            post.author_id = uuid(2000 + (post.id.as_u128() % 2) as u32);
        }
        let ranked = rank(&posts, &ctx);
        let first: Vec<_> = ranked.iter().take(6).map(|id| id.as_u128() % 2).collect();
        assert!(first.contains(&0) && first.contains(&1), "{first:?}");
    }
}
