//! Reading an account's own data: the data export, and the posts of the public RSS feed.

use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

use super::media;
use super::users::{UserRow, me_dto};
use crate::dto::{ExportDto, ExportEdgeDto, ExportMessageDto, ExportPostDto, MediaKind};
use crate::error::AppError;

/// A post of an account's feed.
#[derive(Clone, Debug, FromRow)]
pub struct FeedPost {
    /// The post id.
    pub id: Uuid,
    /// The text.
    pub text: String,
    /// The content warning.
    pub cw: String,
    /// When it was written.
    pub created_at: OffsetDateTime,
    /// The pictures as `(url, alt)`: images, and the poster of a video. Filled by [`feed_posts`].
    #[sqlx(skip)]
    pub media: Vec<(String, String)>,
}

/// The `limit` newest top-level posts of `author` that are not deleted, with their media.
///
/// # Errors
///
/// Database failures.
pub async fn feed_posts(db: &PgPool, author: Uuid, limit: i64) -> Result<Vec<FeedPost>, AppError> {
    let mut posts: Vec<FeedPost> = sqlx::query_as(
        "select id, text, cw, created_at from posts
          where author_id = $1 and deleted_at is null and reply_to_id is null
            and visibility = 'public'
          order by id desc limit $2",
    )
    .bind(author)
    .bind(limit)
    .fetch_all(db)
    .await?;
    let ids: Vec<Uuid> = posts.iter().map(|post| post.id).collect();
    let mut attached = media::for_posts(db, &ids).await?;
    for post in &mut posts {
        post.media = attached
            .remove(&post.id)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|media| {
                let url = match media.kind {
                    MediaKind::Image => Some(media.url),
                    MediaKind::Video => media.poster_url,
                };
                url.map(|url| (url, media.alt))
            })
            .collect();
    }
    Ok(posts)
}

#[derive(FromRow)]
struct PostRow {
    id: Uuid,
    text: String,
    cw: String,
    reply_to_id: Option<Uuid>,
    quote_of_id: Option<Uuid>,
    created_at: OffsetDateTime,
}

#[derive(FromRow)]
struct MessageRow {
    handle: String,
    domain: Option<String>,
    sent: bool,
    text: String,
    ciphertext: Option<String>,
    created_at: OffsetDateTime,
}

async fn edges(db: &PgPool, table: &str, user: Uuid) -> Result<Vec<ExportEdgeDto>, AppError> {
    let sql = format!(
        "select post_id, created_at from {table} where user_id = $1 order by created_at, post_id"
    );
    let rows: Vec<(Uuid, OffsetDateTime)> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(user)
        .fetch_all(db)
        .await?;
    Ok(rows
        .into_iter()
        .map(|(post_id, created_at)| ExportEdgeDto {
            post_id,
            created_at,
        })
        .collect())
}

/// The handles on the `other` side of `table` rows whose `own` column is `user`; remote handles
/// carry their domain. `filter` is an extra condition on the rows `e`.
async fn handles(
    db: &PgPool,
    table: &str,
    own: &str,
    other: &str,
    filter: &str,
    user: Uuid,
) -> Result<Vec<String>, AppError> {
    let sql = format!(
        "select u.handle, u.domain from {table} e join users u on u.id = e.{other}
          where e.{own} = $1 and {filter} order by e.created_at, e.{other}"
    );
    let rows: Vec<(String, Option<String>)> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(user)
        .fetch_all(db)
        .await?;
    Ok(rows
        .into_iter()
        .map(|(handle, domain)| match domain {
            Some(domain) => format!("{handle}@{domain}"),
            None => handle,
        })
        .collect())
}

/// Everything `user` holds: profile, posts, likes, reposts, bookmarks, graph and messages.
///
/// # Errors
///
/// Database failures.
pub async fn export(db: &PgPool, user: &UserRow, domain: &str) -> Result<ExportDto, AppError> {
    let posts: Vec<PostRow> = sqlx::query_as(
        "select id, text, cw, reply_to_id, quote_of_id, created_at from posts
          where author_id = $1 and deleted_at is null order by id",
    )
    .bind(user.id)
    .fetch_all(db)
    .await?;
    let ids: Vec<Uuid> = posts.iter().map(|post| post.id).collect();
    let mut attached = media::for_posts(db, &ids).await?;
    let posts = posts
        .into_iter()
        .map(|post| ExportPostDto {
            media: attached
                .remove(&post.id)
                .unwrap_or_default()
                .into_iter()
                .map(|media| media.url)
                .collect(),
            id: post.id,
            text: post.text,
            cw: post.cw,
            reply_to: post.reply_to_id,
            quote_of: post.quote_of_id,
            created_at: post.created_at,
        })
        .collect();

    let accepted = "e.state = 'accepted'";
    let rows: Vec<MessageRow> = sqlx::query_as(
        "select o.handle, o.domain, m.sender_id = $1 as sent, m.text, m.ciphertext, m.created_at
           from messages m
           join users o on o.id = case when m.sender_id = $1 then m.recipient_id else m.sender_id end
          where (m.sender_id = $1 or m.recipient_id = $1) and m.deleted_at is null
          order by m.id",
    )
    .bind(user.id)
    .fetch_all(db)
    .await?;
    let messages = rows
        .into_iter()
        .map(|row| ExportMessageDto {
            with: match row.domain {
                Some(domain) => format!("{}@{domain}", row.handle),
                None => row.handle,
            },
            sent: row.sent,
            text: row.text,
            ciphertext: row.ciphertext,
            created_at: row.created_at,
        })
        .collect();

    Ok(ExportDto {
        exported_at: OffsetDateTime::now_utc(),
        instance: domain.to_owned(),
        profile: me_dto(db, user, domain).await?,
        posts,
        likes: edges(db, "likes", user.id).await?,
        reposts: edges(db, "reposts", user.id).await?,
        bookmarks: edges(db, "bookmarks", user.id).await?,
        following: handles(
            db,
            "follows",
            "follower_id",
            "followee_id",
            accepted,
            user.id,
        )
        .await?,
        followers: handles(
            db,
            "follows",
            "followee_id",
            "follower_id",
            accepted,
            user.id,
        )
        .await?,
        blocked: handles(db, "blocks", "user_id", "target_id", "true", user.id).await?,
        muted: handles(db, "mutes", "user_id", "target_id", "true", user.id).await?,
        messages,
    })
}
