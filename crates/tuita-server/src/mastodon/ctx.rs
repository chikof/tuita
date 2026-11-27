//! What the mappers in [`super::dto`] need besides the tuita value they convert.
//!
//! A [`PostDto`] holds a small author card, text segments and a reply id. A Mastodon `Status`
//! also needs the author's full profile, the accounts the text mentions (with their profile
//! URLs), and the account that wrote the post being replied to. [`Ctx`] holds those, loaded in
//! a handful of batched queries for a whole page of posts, so a handler does
//! `Ctx::for_posts(&state, &posts)` once and then maps every post with [`super::dto::status`].
//!
//! A mapper never queries. A value missing from the context degrades to what the value itself
//! says (a profile with zero counts, a mention left as plain text), so a context built by hand
//! in a test is a valid one.

use std::collections::{HashMap, HashSet};

use time::OffsetDateTime;
use uuid::Uuid;

use crate::config::Config;
use crate::db::ap::{Mentioned, resolve_mentions};
use crate::db::emoji;
use crate::db::mastodon as queries;
use crate::db::users::user_dto;
use crate::dto::{PostDto, Relation, UserDto};
use crate::error::AppError;
use crate::state::AppState;

/// Facts about an account that [`UserDto`] does not carry.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AccountExtra {
    /// A remote account's actor id (the Mastodon `uri`); `None` for local accounts.
    pub ap_id: Option<String>,
    /// When the account last posted something public or unlisted.
    pub last_status_at: Option<OffsetDateTime>,
    /// Whether the account is listed in directories; `None` when unknown.
    pub discoverable: Option<bool>,
}

/// The context of one mapping run. See the module docs.
#[derive(Clone, Debug)]
pub struct Ctx<'a> {
    config: &'a Config,
    users: HashMap<Uuid, UserDto>,
    extras: HashMap<Uuid, AccountExtra>,
    mentions: HashMap<Uuid, Vec<Mentioned>>,
    reply_authors: HashMap<Uuid, Uuid>,
}

impl<'a> Ctx<'a> {
    /// An empty context: every account is built from the card the mapper got.
    #[must_use]
    pub fn new(config: &'a Config) -> Self {
        Self {
            config,
            users: HashMap::new(),
            extras: HashMap::new(),
            mentions: HashMap::new(),
            reply_authors: HashMap::new(),
        }
    }

    /// Adds the full profile of an account.
    #[must_use]
    pub fn with_user(mut self, user: UserDto) -> Self {
        self.users.insert(user.id, user);
        self
    }

    /// Adds the extra facts of account `id`.
    #[must_use]
    pub fn with_extra(mut self, id: Uuid, extra: AccountExtra) -> Self {
        self.extras.insert(id, extra);
        self
    }

    /// Adds the accounts that the text of post `post` mentions.
    #[must_use]
    pub fn with_mentions(mut self, post: Uuid, mentions: Vec<Mentioned>) -> Self {
        self.mentions.insert(post, mentions);
        self
    }

    /// Adds the author of post `post`, for `in_reply_to_account_id`.
    #[must_use]
    pub fn with_post_author(mut self, post: Uuid, author: Uuid) -> Self {
        self.reply_authors.insert(post, author);
        self
    }

    /// The instance configuration (`PUBLIC_URL` and the federation domain).
    #[must_use]
    pub fn config(&self) -> &Config {
        self.config
    }

    /// The full profile of account `id`, if the context has it.
    #[must_use]
    pub fn user(&self, id: Uuid) -> Option<&UserDto> {
        self.users.get(&id)
    }

    /// The extra facts of account `id`; empty when the context has none.
    #[must_use]
    pub fn extra(&self, id: Uuid) -> AccountExtra {
        self.extras.get(&id).cloned().unwrap_or_default()
    }

    /// The accounts post `post` mentions, in the order its text names them.
    #[must_use]
    pub fn mentions(&self, post: Uuid) -> &[Mentioned] {
        self.mentions.get(&post).map_or(&[], Vec::as_slice)
    }

    /// The author of post `post`, if the context has it.
    #[must_use]
    pub fn post_author(&self, post: Uuid) -> Option<Uuid> {
        self.reply_authors.get(&post).copied()
    }

    /// The context for a page of posts: profiles of every author, booster and quoted author,
    /// the accounts the posts mention, and the authors of the posts they reply to.
    ///
    /// # Errors
    ///
    /// Database failures.
    pub async fn for_posts(state: &'a AppState, posts: &[PostDto]) -> Result<Self, AppError> {
        Self::load(state, &posts.iter().collect::<Vec<_>>(), &[]).await
    }

    /// The context for a page of accounts that are not authors of posts (followers, search
    /// results): profiles of `users` and their extra facts.
    ///
    /// # Errors
    ///
    /// Database failures.
    pub async fn for_users(state: &'a AppState, users: &[Uuid]) -> Result<Self, AppError> {
        Self::load(state, &[], users).await
    }

    /// The context for `posts` plus accounts `users` that appear beside them (the actors of
    /// notifications).
    ///
    /// # Errors
    ///
    /// Database failures.
    pub async fn load(
        state: &'a AppState,
        posts: &[&PostDto],
        users: &[Uuid],
    ) -> Result<Self, AppError> {
        let db = state.db();
        let config = state.config();
        let mut ids: HashSet<Uuid> = users.iter().copied().collect();
        let mut parents: HashSet<Uuid> = HashSet::new();
        for post in posts {
            ids.insert(post.author.id);
            if let Some(booster) = &post.reposted_by {
                ids.insert(booster.id);
            }
            if let Some(author) = post.quote.as_ref().and_then(|quote| quote.author.as_ref()) {
                ids.insert(author.id);
            }
            if let Some(reply) = &post.reply_to {
                parents.insert(reply.id);
            }
        }
        let ids: Vec<Uuid> = ids.into_iter().collect();
        let rows = queries::account_rows(db, &ids).await?;
        let last = queries::last_status_times(db, &ids).await?;
        let mut ctx = Self::new(config);
        let mut cards = Vec::with_capacity(rows.len());
        for row in rows {
            ctx.extras.insert(
                row.id,
                AccountExtra {
                    ap_id: row.ap_id.clone(),
                    last_status_at: last.get(&row.id).copied(),
                    discoverable: row.discoverable,
                },
            );
            cards.push(user_dto(&row, Relation::default()));
        }
        emoji::hydrate(db, &mut cards).await?;
        ctx.users = cards.into_iter().map(|user| (user.id, user)).collect();
        let parents: Vec<Uuid> = parents.into_iter().collect();
        ctx.reply_authors = queries::post_authors(db, &parents).await?;
        let texts: Vec<&str> = posts.iter().map(|post| post.text.as_str()).collect();
        let resolved = resolve_mentions(db, config.domain(), &texts).await?;
        for (post, mentions) in posts.iter().zip(resolved) {
            ctx.mentions.insert(post.id, mentions);
        }
        Ok(ctx)
    }
}
