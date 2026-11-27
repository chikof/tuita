//! Database access, one module per area. Handlers call these; they never write SQL.
//!
//! Conventions every module follows:
//! - Functions take `&PgPool`, or `&mut PgConnection` when they must run in the caller's
//!   transaction (anything that changes a counter).
//! - `$1` is the viewer's user id in every query that filters by visibility.
//! - Ids are `Uuid::now_v7()`; counters change in the same transaction as the row they count.

pub mod accounts;
pub mod admin_reads;
pub mod affiliations;
pub mod announcements;
pub mod ap;
pub mod audit;
pub mod compose;
pub mod deliveries;
pub mod emoji;
pub mod federation;
pub mod graph;
pub mod inbound;
pub mod invites;
pub mod lists;
pub mod live;
pub mod mastodon;
pub mod mastodon_accounts;
pub mod mastodon_activity;
pub mod mastodon_feeds;
pub mod mastodon_misc;
pub mod media;
pub mod message_keys;
pub mod messages;
pub mod moderation;
pub mod notifications;
pub mod oauth;
pub mod passkeys;
pub mod polls;
pub mod post_actions;
pub mod posts;
pub mod remote;
pub mod reports;
pub mod scheduled;
pub mod search;
pub mod sessions;
pub mod settings;
pub mod timeline;
pub mod tokens;
pub mod users;
pub mod views;

/// SQL that is true when the author behind users alias `author` is visible to viewer `viewer`
/// (a bind parameter such as `$1`): not suspended, not deleted, no block either way, not muted.
#[must_use]
pub fn visible_author(author: &str, viewer: &str) -> String {
    format!(
        "(not {} and not {})",
        blocked_author(author, viewer),
        muted_author(author, viewer)
    )
}

/// SQL that is true when `domain` is the blocked domain `block` or a subdomain of it. Both are
/// SQL expressions of type `text`; a `domain_blocks` row blocks the whole name below it. The
/// suffix is compared with `right`, not `like`, so no character of `block` acts as a wildcard.
#[must_use]
pub fn domain_in_block(domain: &str, block: &str) -> String {
    format!("({domain} = {block} or right({domain}, length({block}) + 1) = '.' || {block})")
}

/// SQL that is true when the author behind users alias `author` is on an instance with a
/// `suspend` domain block (on its domain or a parent of it).
#[must_use]
pub fn suspended_domain(author: &str) -> String {
    format!(
        "({author}.domain is not null and exists (select 1 from domain_blocks bd \
            where bd.severity = 'suspend' and {}))",
        domain_in_block(&format!("{author}.domain"), "bd.domain")
    )
}

/// SQL that is true when the author behind users alias `author` is gone or blocked: suspended,
/// deleted, from an instance with a `suspend` domain block, or a block either way with `viewer`.
#[must_use]
pub fn blocked_author(author: &str, viewer: &str) -> String {
    format!(
        "({author}.suspended_at is not null or {author}.deleted_at is not null \
         or {} \
         or exists (select 1 from blocks vb where (vb.user_id = {viewer} and vb.target_id = {author}.id) \
                    or (vb.user_id = {author}.id and vb.target_id = {viewer})))",
        suspended_domain(author)
    )
}

/// SQL that is true when `viewer` muted the author behind users alias `author`.
#[must_use]
pub fn muted_author(author: &str, viewer: &str) -> String {
    format!(
        "exists (select 1 from mutes vm where vm.user_id = {viewer} and vm.target_id = {author}.id)"
    )
}

/// SQL that is true when the author behind users alias `author` is on an instance with a
/// `silence` domain block (on its domain or a parent of it). Public timelines, tag lists and
/// search leave such authors out.
#[must_use]
pub fn silenced_author(author: &str) -> String {
    format!(
        "({author}.domain is not null and exists (select 1 from domain_blocks sd \
            where sd.severity = 'silence' and {}))",
        domain_in_block(&format!("{author}.domain"), "sd.domain")
    )
}

/// SQL that is true when `viewer` (a bind parameter such as `$1`) may open the post behind posts
/// alias `post`: public and unlisted posts, the viewer's own, posts that mention the viewer, and
/// followers-only posts of authors the viewer follows. Direct posts reach only their author and
/// the people they mention.
#[must_use]
pub fn visible_post(post: &str, viewer: &str) -> String {
    format!(
        "({post}.visibility in ('public', 'unlisted') or {post}.author_id = {viewer} \
         or exists (select 1 from mentions vmn where vmn.post_id = {post}.id and vmn.user_id = {viewer}) \
         or ({post}.visibility = 'followers' and exists (select 1 from follows vf \
              where vf.follower_id = {viewer} and vf.followee_id = {post}.author_id \
              and vf.state = 'accepted')))"
    )
}

/// SQL for the posts a timeline may list: everything but direct posts, which no timeline shows.
/// Combine with [`visible_post`], which keeps followers-only posts from strangers out.
#[must_use]
pub fn listable_post(post: &str) -> String {
    format!("{post}.visibility <> 'direct'")
}

/// SQL for the posts public timelines, tag lists and search may list: public posts of authors on
/// no silenced instance. `author` is the users alias of the post's author.
#[must_use]
pub fn discoverable_post(post: &str, author: &str) -> String {
    format!(
        "({post}.visibility = 'public' and not {})",
        silenced_author(author)
    )
}

/// SQL that is true when one of the viewer's muted words is in the post behind posts alias
/// `post`: a phrase of its content warning or text, or a `#tag` that `post_tags` holds. `viewer`
/// is a bind parameter such as `$1`.
///
/// Muted words are phrases, not whole words: `cat` hides `concatenate`.
#[must_use]
pub fn word_muted(post: &str, viewer: &str) -> String {
    format!(
        "exists (select 1 from users mv, unnest(coalesce(mv.muted_words, '{{}}')) w
                  where mv.id = {viewer} and w <> ''
                    and (position(lower(w) in lower({post}.cw || ' ' || {post}.text)) > 0
                         or (left(w, 1) = '#' and exists (select 1 from post_tags mt
                              where mt.post_id = {post}.id and mt.tag = lower(substr(w, 2))))))"
    )
}

/// SQL that is true when the viewer may see the post behind posts alias `post` as far as muted
/// words go: their own posts always, others' unless [`word_muted`].
#[must_use]
pub fn word_visible(post: &str, viewer: &str) -> String {
    format!(
        "({post}.author_id = {viewer} or not {})",
        word_muted(post, viewer)
    )
}

pub mod deletion;
pub mod export;
