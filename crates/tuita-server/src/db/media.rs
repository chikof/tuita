//! Media rows: how posts claim uploads, and how uploads become [`MediaDto`]s.
//!
//! The files and the upload routes live in [`crate::media`]; this module is the SQL the post
//! area shares with them. A file's `path` column is its key in the media store:
//! `{id}.{ext}`.

use std::collections::HashMap;

use sqlx::{FromRow, PgConnection, PgPool};
use uuid::Uuid;

use crate::dto::{MediaDto, MediaKind};
use crate::error::AppError;

/// Most images a post may carry.
pub const MAX_IMAGES: usize = 4;

/// The public URL of a stored file, from its `path` column.
#[must_use]
pub fn url(path: &str) -> String {
    format!("/media/{path}")
}

/// The columns [`dto`] needs, plus the poster's file name.
#[derive(Clone, Debug, FromRow)]
pub struct MediaRow {
    /// Media id.
    pub id: Uuid,
    /// `image` or `video`.
    pub kind: String,
    /// Object key in the media store.
    pub path: String,
    /// Description.
    pub alt: String,
    /// Width in pixels.
    pub width: Option<i32>,
    /// Height in pixels.
    pub height: Option<i32>,
    /// File name of the poster image of a video.
    pub poster_path: Option<String>,
}

/// Selects [`MediaRow`] from `media m left join media pp on pp.id = m.poster_id`.
pub const ROW_COLS: &str = "m.id, m.kind, m.path, m.alt, m.width, m.height, pp.path as poster_path";

/// Builds the wire form of `row`.
#[must_use]
pub fn dto(row: &MediaRow) -> MediaDto {
    MediaDto {
        id: row.id,
        kind: MediaKind::parse(&row.kind).unwrap_or(MediaKind::Image),
        url: url(&row.path),
        poster_url: row.poster_path.as_deref().map(url),
        alt: row.alt.clone(),
        width: row.width,
        height: row.height,
    }
}

/// One upload by id, whoever owns it and wherever it is attached.
///
/// # Errors
///
/// Database failures.
pub async fn find(db: &PgPool, id: Uuid) -> Result<Option<MediaDto>, AppError> {
    let sql = format!(
        "select {ROW_COLS} from media m left join media pp on pp.id = m.poster_id where m.id = $1"
    );
    let row: Option<MediaRow> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(id)
        .fetch_optional(db)
        .await?;
    Ok(row.as_ref().map(dto))
}

/// The media of each of `posts`, in attachment order. Posts without media are absent.
///
/// # Errors
///
/// Database failures.
pub async fn for_posts(
    db: &PgPool,
    posts: &[Uuid],
) -> Result<HashMap<Uuid, Vec<MediaDto>>, AppError> {
    if posts.is_empty() {
        return Ok(HashMap::new());
    }
    let sql = format!(
        "select pm.post_id, {ROW_COLS}
           from post_media pm join media m on m.id = pm.media_id
           left join media pp on pp.id = m.poster_id
          where pm.post_id = any($1) order by pm.post_id, pm.position"
    );
    #[derive(FromRow)]
    struct Attached {
        post_id: Uuid,
        #[sqlx(flatten)]
        media: MediaRow,
    }
    let rows: Vec<Attached> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(posts)
        .fetch_all(db)
        .await?;
    let mut out: HashMap<Uuid, Vec<MediaDto>> = HashMap::new();
    for row in rows {
        out.entry(row.post_id).or_default().push(dto(&row.media));
    }
    Ok(out)
}

/// Checks that `ids` are uploads of `owner` that no post holds yet and that fit together
/// (up to [`MAX_IMAGES`] images, or one video), and locks them for the transaction.
/// Returns them in the order given.
///
/// # Errors
///
/// `400` for a missing, foreign, already attached or badly combined upload.
pub async fn claim(
    conn: &mut PgConnection,
    owner: Uuid,
    ids: &[Uuid],
) -> Result<Vec<(Uuid, MediaKind)>, AppError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let mut seen = ids.to_vec();
    seen.sort_unstable();
    seen.dedup();
    if seen.len() != ids.len() {
        return Err(AppError::bad_request("Attach each upload only once."));
    }
    let rows: Vec<(Uuid, String, Option<Uuid>)> = sqlx::query_as(
        "select id, kind, post_id from media
          where id = any($1) and owner_id = $2
            and not exists (select 1 from post_media pm where pm.media_id = media.id)
          for update",
    )
    .bind(ids)
    .bind(owner)
    .fetch_all(&mut *conn)
    .await?;
    let mut claimed = Vec::with_capacity(ids.len());
    for id in ids {
        let Some((_, kind, post_id)) = rows.iter().find(|(row, _, _)| row == id) else {
            return Err(AppError::bad_request(
                "That upload wasn’t found, or it is already attached to a post. Please upload it again.",
            ));
        };
        if post_id.is_some() {
            return Err(AppError::bad_request(
                "That upload is already attached to a post.",
            ));
        }
        claimed.push((*id, MediaKind::parse(kind).unwrap_or(MediaKind::Image)));
    }
    let videos = claimed
        .iter()
        .filter(|(_, kind)| *kind == MediaKind::Video)
        .count();
    if videos > 0 && claimed.len() > 1 {
        return Err(AppError::bad_request(
            "A post can have a video or images, but only one video and not both.",
        ));
    }
    if claimed.len() > MAX_IMAGES {
        return Err(AppError::bad_request(format!(
            "You can attach up to {MAX_IMAGES} images."
        )));
    }
    Ok(claimed)
}

/// Attaches claimed uploads to `post` in the order given.
///
/// # Errors
///
/// Database failures.
pub async fn attach(conn: &mut PgConnection, post: Uuid, ids: &[Uuid]) -> Result<(), AppError> {
    for (position, id) in ids.iter().enumerate() {
        sqlx::query("insert into post_media (post_id, media_id, position) values ($1, $2, $3)")
            .bind(post)
            .bind(id)
            .bind(i16::try_from(position).unwrap_or(i16::MAX))
            .execute(&mut *conn)
            .await?;
    }
    sqlx::query("update media set post_id = $1 where id = any($2)")
        .bind(post)
        .bind(ids)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

/// The uploads `ids`, keyed by id. Unknown ids are absent.
///
/// # Errors
///
/// Database failures.
pub async fn by_ids(db: &PgPool, ids: &[Uuid]) -> Result<HashMap<Uuid, MediaDto>, AppError> {
    if ids.is_empty() {
        return Ok(HashMap::new());
    }
    let sql = format!(
        "select {ROW_COLS} from media m left join media pp on pp.id = m.poster_id where m.id = any($1)"
    );
    let rows: Vec<MediaRow> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(ids)
        .fetch_all(db)
        .await?;
    Ok(rows.iter().map(|row| (row.id, dto(row))).collect())
}
