//! Notification grouping.
//!
//! Likes and reposts of the same post, and follows, collapse into one row each ("Ana, Ben and 3
//! others liked your post"). Every other kind stays on its own row. The server loads one page of
//! rows, newest first, drops the hidden and deleted ones, and passes them to [`group`].
//!
//! ```
//! use time::OffsetDateTime;
//! use tuita_core::notify::{NotificationKind, NotificationRow, group};
//! use uuid::Uuid;
//!
//! let post = Some(Uuid::from_u128(7));
//! let row = |id: u128, actor: u128| NotificationRow {
//!     id: Uuid::from_u128(id),
//!     kind: NotificationKind::Like,
//!     actor_id: Uuid::from_u128(actor),
//!     post_id: post,
//!     created_at: OffsetDateTime::UNIX_EPOCH,
//!     read: false,
//! };
//! let groups = group(&[row(4, 1), row(3, 2), row(2, 3), row(1, 4)]);
//! assert_eq!(groups.len(), 1);
//! assert_eq!(groups[0].actor_count, 4);
//! assert_eq!(groups[0].actors.len(), 3);
//! ```

use std::collections::{HashMap, HashSet};

use time::OffsetDateTime;
use uuid::Uuid;

/// How many actors a group names; the rest only count.
pub const MAX_ACTORS: usize = 3;

/// What happened.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NotificationKind {
    /// Someone liked a post.
    Like,
    /// Someone reposted a post.
    Repost,
    /// Someone replied to a post.
    Reply,
    /// Someone mentioned the user.
    Mention,
    /// Someone quoted a post.
    Quote,
    /// Someone followed the user.
    Follow,
    /// Someone asked to follow the user.
    FollowRequest,
    /// A poll the user voted in or wrote has ended.
    Poll,
    /// An organization affiliated the user.
    Affiliation,
}

impl NotificationKind {
    /// Whether rows of this kind collapse per post.
    #[must_use]
    pub const fn is_groupable(self) -> bool {
        matches!(self, Self::Like | Self::Repost | Self::Follow)
    }
}

/// One notification as stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotificationRow {
    /// The notification.
    pub id: Uuid,
    /// What happened.
    pub kind: NotificationKind,
    /// Who did it.
    pub actor_id: Uuid,
    /// The post it is about, if any. A follow has none.
    pub post_id: Option<Uuid>,
    /// When it happened.
    pub created_at: OffsetDateTime,
    /// Whether the user has seen it.
    pub read: bool,
}

// 68: UUIDs align to 1, so id, actor and Option<Uuid> take 16+16+17, the timestamp 16 (align 4),
// kind and flag 2: 67 rounded up to 4.
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<NotificationRow>() == 68);

/// One row of the notification list: a single notification, or the collapsed rows of one kind
/// and post.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationGroup {
    /// The newest notification of the group.
    pub id: Uuid,
    /// What happened.
    pub kind: NotificationKind,
    /// The post it is about, if any.
    pub post_id: Option<Uuid>,
    /// When the newest notification happened.
    pub created_at: OffsetDateTime,
    /// Whether every notification of the group has been seen.
    pub read: bool,
    /// The first [`MAX_ACTORS`] distinct actors, newest first.
    pub actors: Vec<Uuid>,
    /// How many distinct actors the group has.
    pub actor_count: u32,
}

/// Groups one page of notifications, given newest first. The groups come out in the order their
/// newest notification appears, which is the order of `rows`.
///
/// An actor counts once per group even when it appears again after the first
/// [`MAX_ACTORS`], not only while it is among the named ones.
#[must_use]
pub fn group(rows: &[NotificationRow]) -> Vec<NotificationGroup> {
    let mut groups: Vec<NotificationGroup> = Vec::new();
    let mut by_key: HashMap<(NotificationKind, Option<Uuid>), usize> = HashMap::new();
    let mut counted: HashSet<(usize, Uuid)> = HashSet::new();
    for row in rows {
        let index = match by_key.get(&(row.kind, row.post_id)) {
            Some(&index) if row.kind.is_groupable() => index,
            _ => {
                groups.push(NotificationGroup {
                    id: row.id,
                    kind: row.kind,
                    post_id: row.post_id,
                    created_at: row.created_at,
                    read: true,
                    actors: Vec::new(),
                    actor_count: 0,
                });
                let index = groups.len() - 1;
                if row.kind.is_groupable() {
                    by_key.insert((row.kind, row.post_id), index);
                }
                index
            }
        };
        let group = &mut groups[index];
        if counted.insert((index, row.actor_id)) {
            group.actor_count += 1;
            if group.actors.len() < MAX_ACTORS {
                group.actors.push(row.actor_id);
            }
        }
        group.read &= row.read;
    }
    groups
}

#[cfg(test)]
mod test {
    use super::*;
    use NotificationKind::{Follow, Like, Mention, Reply, Repost};

    fn at(secs: i64) -> OffsetDateTime {
        OffsetDateTime::UNIX_EPOCH + time::Duration::seconds(secs)
    }

    /// Rows get descending ids and times, as a newest-first page has.
    fn page(rows: &[(NotificationKind, u128, Option<u128>, bool)]) -> Vec<NotificationRow> {
        let count = rows.len() as u128;
        rows.iter()
            .enumerate()
            .map(|(i, &(kind, actor, post, read))| NotificationRow {
                id: Uuid::from_u128(count - i as u128),
                kind,
                actor_id: Uuid::from_u128(actor),
                post_id: post.map(Uuid::from_u128),
                created_at: at(1000 - i as i64),
                read,
            })
            .collect()
    }

    fn ids(values: &[u128]) -> Vec<Uuid> {
        values.iter().map(|v| Uuid::from_u128(*v)).collect()
    }

    #[test]
    fn collapses_likes_of_one_post() {
        let groups = group(&page(&[
            (Like, 1, Some(10), true),
            (Like, 2, Some(10), true),
            (Like, 3, Some(10), true),
            (Like, 4, Some(10), true),
            (Like, 5, Some(10), true),
        ]));
        assert_eq!(groups.len(), 1);
        assert_eq!(
            groups[0].id,
            Uuid::from_u128(5),
            "the newest row names the group"
        );
        assert_eq!(groups[0].created_at, at(1000));
        assert_eq!(groups[0].actors, ids(&[1, 2, 3]));
        assert_eq!(groups[0].actor_count, 5);
        assert!(groups[0].read);
    }

    #[test]
    fn separates_by_kind_and_post() {
        let groups = group(&page(&[
            (Like, 1, Some(10), false),
            (Like, 2, Some(11), false),
            (Repost, 3, Some(10), false),
            (Like, 4, Some(10), false),
        ]));
        let shape: Vec<_> = groups
            .iter()
            .map(|g| (g.kind, g.post_id, g.actor_count))
            .collect();
        assert_eq!(
            shape,
            [
                (Like, Some(Uuid::from_u128(10)), 2),
                (Like, Some(Uuid::from_u128(11)), 1),
                (Repost, Some(Uuid::from_u128(10)), 1),
            ]
        );
    }

    #[test]
    fn follows_collapse_without_a_post() {
        let groups = group(&page(&[
            (Follow, 1, None, false),
            (Reply, 2, Some(9), false),
            (Follow, 3, None, true),
        ]));
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].kind, Follow);
        assert_eq!(groups[0].actors, ids(&[1, 3]));
        assert!(!groups[0].read, "one unread row makes the group unread");
    }

    #[test]
    fn replies_and_mentions_stay_single() {
        let groups = group(&page(&[
            (Reply, 1, Some(10), false),
            (Reply, 2, Some(10), true),
            (Mention, 1, Some(10), true),
        ]));
        assert_eq!(groups.len(), 3);
        assert!(
            groups
                .iter()
                .all(|g| g.actor_count == 1 && g.actors.len() == 1)
        );
        assert_eq!(
            groups.iter().map(|g| g.read).collect::<Vec<_>>(),
            [false, true, true]
        );
    }

    #[test]
    fn counts_an_actor_once_even_past_the_named_ones() {
        let rows: Vec<_> = [1, 2, 3, 4, 4, 1, 4]
            .iter()
            .map(|a| (Follow, *a, None, false))
            .collect();
        let groups = group(&page(&rows));
        assert_eq!(groups[0].actor_count, 4);
        assert_eq!(groups[0].actors, ids(&[1, 2, 3]));
    }

    #[test]
    fn empty_page_has_no_groups() {
        assert!(group(&[]).is_empty());
    }
}
