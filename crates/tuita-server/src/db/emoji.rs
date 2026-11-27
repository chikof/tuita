//! Custom emotes: the `custom_emojis` table, and the lookup that fills the `emojis` array of every
//! DTO that carries text.
//!
//! An emote belongs to an origin: local (`domain` null, uploaded by staff) or one remote domain
//! (a copy of an image that instance sent). A local author's text uses local emotes only, a
//! remote author's text only that domain's. [`hydrate`] resolves a whole response with one query.

use std::collections::{HashMap, HashSet};

use sqlx::{FromRow, PgConnection, PgPool};
use tuita_core::emoji::{MAX_LEN, MIN_LEN, is_valid_shortcode, shortcodes};
use uuid::Uuid;

use super::audit::{self, Entry};
use super::media;
use crate::dto::{EmojiDto, PostDto, QuoteDto, UserDto, UserRefDto};
use crate::error::AppError;

/// Most local emotes an instance holds.
pub const MAX_LOCAL: i64 = 1000;
/// Longest category name in characters.
pub const MAX_CATEGORY_CHARS: usize = 32;

#[derive(FromRow)]
struct Row {
    shortcode: String,
    domain: Option<String>,
    path: String,
    category: Option<String>,
}

impl Row {
    fn dto(self) -> EmojiDto {
        EmojiDto {
            shortcode: self.shortcode,
            url: media::url(&self.path),
            category: self.category,
        }
    }
}

const SELECT: &str = "select e.shortcode, e.domain, m.path, e.category
    from custom_emojis e join media m on m.id = e.media_id";

fn origin(domain: Option<&str>) -> String {
    domain.map_or_else(String::new, str::to_ascii_lowercase)
}

/// The shortcodes and origins a response mentions, collected before the one lookup.
#[derive(Debug, Default)]
pub struct Wanted {
    shortcodes: HashSet<String>,
    origins: HashSet<String>,
}

impl Wanted {
    /// Notes the tokens of `text`, written by an author of `domain` (`None` is local).
    pub fn add(&mut self, domain: Option<&str>, text: &str) {
        let mut any = false;
        for code in shortcodes(text) {
            self.shortcodes.insert(code.to_ascii_lowercase());
            any = true;
        }
        if any {
            self.origins.insert(origin(domain));
        }
    }
}

/// The emotes found for a [`Wanted`], ready to be matched against each text.
#[derive(Debug, Default)]
pub struct Catalog {
    by_key: HashMap<(String, String), EmojiDto>,
}

impl Catalog {
    /// One query for everything `wanted` names; none when it names nothing.
    ///
    /// # Errors
    ///
    /// Database failures.
    pub async fn load(db: &PgPool, wanted: Wanted) -> Result<Self, AppError> {
        if wanted.shortcodes.is_empty() {
            return Ok(Self::default());
        }
        let codes: Vec<String> = wanted.shortcodes.into_iter().collect();
        let origins: Vec<String> = wanted.origins.into_iter().collect();
        let sql = format!(
            "{SELECT} where lower(e.shortcode) = any($1) and coalesce(e.domain, '') = any($2)"
        );
        let rows: Vec<Row> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
            .bind(&codes)
            .bind(&origins)
            .fetch_all(db)
            .await?;
        let by_key = rows
            .into_iter()
            .map(|row| {
                let key = (
                    origin(row.domain.as_deref()),
                    row.shortcode.to_ascii_lowercase(),
                );
                (key, row.dto())
            })
            .collect();
        Ok(Self { by_key })
    }

    /// The distinct emotes the `texts` of one author of `domain` mention, in order of appearance.
    #[must_use]
    pub fn mentioned<'a>(
        &self,
        domain: Option<&str>,
        texts: impl IntoIterator<Item = &'a str>,
    ) -> Vec<EmojiDto> {
        if self.by_key.is_empty() {
            return Vec::new();
        }
        let origin = origin(domain);
        let mut out: Vec<EmojiDto> = Vec::new();
        for text in texts {
            for code in shortcodes(text) {
                let key = (origin.clone(), code.to_ascii_lowercase());
                if let Some(emoji) = self.by_key.get(&key)
                    && !out.contains(emoji)
                {
                    out.push(emoji.clone());
                }
            }
        }
        out
    }
}

/// A DTO with an `emojis` array that [`hydrate`] fills.
pub trait Emojified {
    /// Adds every text that may hold a `:shortcode:`, nested DTOs included.
    fn want(&self, wanted: &mut Wanted);

    /// Sets `emojis` on this DTO and the DTOs inside it.
    fn attach(&mut self, catalog: &Catalog);
}

impl<T: Emojified> Emojified for &mut T {
    fn want(&self, wanted: &mut Wanted) {
        (**self).want(wanted);
    }

    fn attach(&mut self, catalog: &Catalog) {
        (**self).attach(catalog);
    }
}

impl Emojified for UserRefDto {
    fn want(&self, wanted: &mut Wanted) {
        wanted.add(self.domain.as_deref(), &self.display_name);
    }

    fn attach(&mut self, catalog: &Catalog) {
        self.emojis = catalog.mentioned(self.domain.as_deref(), [self.display_name.as_str()]);
    }
}

impl Emojified for UserDto {
    fn want(&self, wanted: &mut Wanted) {
        let domain = self.domain.as_deref();
        wanted.add(domain, &self.display_name);
        wanted.add(domain, &self.bio);
        wanted.add(domain, &self.location);
    }

    fn attach(&mut self, catalog: &Catalog) {
        self.emojis = catalog.mentioned(
            self.domain.as_deref(),
            [
                self.display_name.as_str(),
                self.bio.as_str(),
                self.location.as_str(),
            ],
        );
    }
}

impl Emojified for QuoteDto {
    fn want(&self, wanted: &mut Wanted) {
        let Some(author) = &self.author else {
            return;
        };
        author.want(wanted);
        wanted.add(author.domain.as_deref(), &self.text);
        wanted.add(author.domain.as_deref(), &self.cw);
    }

    fn attach(&mut self, catalog: &Catalog) {
        let Some(author) = &mut self.author else {
            return;
        };
        author.attach(catalog);
        self.emojis = catalog.mentioned(
            author.domain.as_deref(),
            [self.text.as_str(), self.cw.as_str()],
        );
    }
}

impl Emojified for PostDto {
    fn want(&self, wanted: &mut Wanted) {
        let domain = self.author.domain.as_deref();
        wanted.add(domain, &self.text);
        wanted.add(domain, &self.cw);
        if let Some(poll) = &self.poll {
            for option in &poll.options {
                wanted.add(domain, &option.text);
            }
        }
        self.author.want(wanted);
        if let Some(reposter) = &self.reposted_by {
            reposter.want(wanted);
        }
        if let Some(quote) = &self.quote {
            quote.want(wanted);
        }
    }

    fn attach(&mut self, catalog: &Catalog) {
        let options = self
            .poll
            .iter()
            .flat_map(|poll| poll.options.iter().map(|option| option.text.as_str()));
        self.emojis = catalog.mentioned(
            self.author.domain.as_deref(),
            [self.text.as_str(), self.cw.as_str()]
                .into_iter()
                .chain(options),
        );
        self.author.attach(catalog);
        if let Some(reposter) = &mut self.reposted_by {
            reposter.attach(catalog);
        }
        if let Some(quote) = &mut self.quote {
            quote.attach(catalog);
        }
    }
}

/// Fills `emojis` on `items` and everything inside them with one query for the whole slice.
///
/// # Errors
///
/// Database failures.
pub async fn hydrate<T: Emojified>(db: &PgPool, items: &mut [T]) -> Result<(), AppError> {
    let mut wanted = Wanted::default();
    for item in items.iter() {
        item.want(&mut wanted);
    }
    let catalog = Catalog::load(db, wanted).await?;
    for item in items.iter_mut() {
        item.attach(&catalog);
    }
    Ok(())
}

/// The local emotes, ordered by category (uncategorised first) and shortcode.
///
/// # Errors
///
/// Database failures.
pub async fn list_local(db: &PgPool) -> Result<Vec<EmojiDto>, AppError> {
    let sql = format!(
        "{SELECT} where e.domain is null order by e.category nulls first, lower(e.shortcode)"
    );
    let rows: Vec<Row> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .fetch_all(db)
        .await?;
    Ok(rows.into_iter().map(Row::dto).collect())
}

/// The distinct emotes that the shortcodes in `texts` name for an author of `domain` (`None` is
/// a local author). One query; none when no text holds a token.
///
/// # Errors
///
/// Database failures.
pub async fn resolve(
    db: &PgPool,
    domain: Option<&str>,
    texts: &[&str],
) -> Result<Vec<EmojiDto>, AppError> {
    let mut wanted = Wanted::default();
    for text in texts {
        wanted.add(domain, text);
    }
    let catalog = Catalog::load(db, wanted).await?;
    Ok(catalog.mentioned(domain, texts.iter().copied()))
}

/// Stores or replaces the copy of `domain`'s emote `shortcode`, fetched from `source_url`. The
/// shortcode takes the spelling of the latest copy; a replaced image is left for cleanup.
///
/// # Errors
///
/// Database failures.
pub async fn upsert_remote(
    db: &PgPool,
    domain: &str,
    shortcode: &str,
    media_id: Uuid,
    source_url: &str,
) -> Result<(), AppError> {
    sqlx::query(
        "insert into custom_emojis (id, shortcode, domain, media_id, source_url)
         values ($1, $2, $3, $4, $5)
         on conflict (lower(shortcode), (coalesce(domain, '')))
         do update set shortcode = excluded.shortcode, media_id = excluded.media_id,
                       source_url = excluded.source_url",
    )
    .bind(Uuid::now_v7())
    .bind(shortcode)
    .bind(domain.to_ascii_lowercase())
    .bind(media_id)
    .bind(source_url)
    .execute(db)
    .await?;
    Ok(())
}

/// The icon address the copy of `domain`'s emote `shortcode` came from; `None` when there is no
/// copy.
///
/// # Errors
///
/// Database failures.
pub async fn remote_source(
    db: &PgPool,
    domain: &str,
    shortcode: &str,
) -> Result<Option<String>, AppError> {
    Ok(sqlx::query_scalar::<_, Option<String>>(
        "select source_url from custom_emojis
          where domain = $1 and lower(shortcode) = lower($2)",
    )
    .bind(domain.to_ascii_lowercase())
    .bind(shortcode)
    .fetch_optional(db)
    .await?
    .flatten())
}

/// How many local emotes exist.
///
/// # Errors
///
/// Database failures.
pub async fn count_local(conn: &mut PgConnection) -> Result<i64, AppError> {
    Ok(
        sqlx::query_scalar("select count(*) from custom_emojis where domain is null")
            .fetch_one(conn)
            .await?,
    )
}

/// Checks the shortcode rule of a new emote.
///
/// # Errors
///
/// `400` when `shortcode` is not 2 to 32 ASCII letters, digits or underscores.
pub fn validate_shortcode(shortcode: &str) -> Result<(), AppError> {
    if is_valid_shortcode(shortcode) {
        Ok(())
    } else {
        Err(AppError::bad_request(format!(
            "A shortcode is {MIN_LEN} to {MAX_LEN} letters, digits or underscores."
        )))
    }
}

/// Trims a category; blank means none.
///
/// # Errors
///
/// `400` when it is longer than [`MAX_CATEGORY_CHARS`] characters.
pub fn clean_category(raw: Option<&str>) -> Result<Option<String>, AppError> {
    let Some(category) = raw.map(str::trim).filter(|category| !category.is_empty()) else {
        return Ok(None);
    };
    if category.chars().count() > MAX_CATEGORY_CHARS {
        return Err(AppError::bad_request(format!(
            "A category is at most {MAX_CATEGORY_CHARS} characters."
        )));
    }
    Ok(Some(category.to_owned()))
}

/// Adds a local emote whose image is the stored upload `media_id` (file key `path`), in the
/// caller's transaction, and audits it.
///
/// # Errors
///
/// `400` for an invalid or taken shortcode, a long category or when [`MAX_LOCAL`] emotes exist;
/// database failures.
pub async fn create_local(
    conn: &mut PgConnection,
    actor: Uuid,
    shortcode: &str,
    category: Option<&str>,
    media_id: Uuid,
    path: &str,
) -> Result<EmojiDto, AppError> {
    validate_shortcode(shortcode)?;
    let category = clean_category(category)?;
    if count_local(&mut *conn).await? >= MAX_LOCAL {
        return Err(AppError::bad_request(format!(
            "This instance already has {MAX_LOCAL} emotes."
        )));
    }
    let inserted: Option<Uuid> = sqlx::query_scalar(
        "insert into custom_emojis (id, shortcode, domain, media_id, category, created_by)
         values ($1, $2, null, $3, $4, $5)
         on conflict do nothing returning id",
    )
    .bind(Uuid::now_v7())
    .bind(shortcode)
    .bind(media_id)
    .bind(&category)
    .bind(actor)
    .fetch_optional(&mut *conn)
    .await?;
    if inserted.is_none() {
        return Err(AppError::bad_request("That shortcode is already taken."));
    }
    audit::record(
        conn,
        Entry::new(Some(actor), "emoji.create").target(format!(":{shortcode}:")),
    )
    .await?;
    Ok(EmojiDto {
        shortcode: shortcode.to_owned(),
        url: media::url(path),
        category,
    })
}

/// Sets or clears the category of a local emote, in the caller's transaction, and audits it.
///
/// # Errors
///
/// `404` for an unknown emote, `400` for a long category; database failures.
pub async fn set_category(
    conn: &mut PgConnection,
    actor: Uuid,
    shortcode: &str,
    category: Option<&str>,
) -> Result<EmojiDto, AppError> {
    let category = clean_category(category)?;
    let row: Option<Row> = sqlx::query_as(
        "update custom_emojis e set category = $2 from media m
          where m.id = e.media_id and e.domain is null and lower(e.shortcode) = lower($1)
          returning e.shortcode, e.domain, m.path, e.category",
    )
    .bind(shortcode)
    .bind(&category)
    .fetch_optional(&mut *conn)
    .await?;
    let row = row.ok_or(AppError::Missing(NOT_FOUND))?;
    audit::record(
        conn,
        Entry::new(Some(actor), "emoji.update")
            .target(format!(":{}:", row.shortcode))
            .text(category.unwrap_or_default()),
    )
    .await?;
    Ok(row.dto())
}

/// Deletes a local emote and its image row, in the caller's transaction, and audits it. Returns
/// the image's file key; the caller removes the file once the transaction commits. Text that
/// names the emote keeps `:shortcode:` as plain text.
///
/// # Errors
///
/// `404` for an unknown emote; database failures.
pub async fn delete_local(
    conn: &mut PgConnection,
    actor: Uuid,
    shortcode: &str,
) -> Result<String, AppError> {
    let removed: Option<(String, Uuid)> = sqlx::query_as(
        "delete from custom_emojis
          where domain is null and lower(shortcode) = lower($1) returning shortcode, media_id",
    )
    .bind(shortcode)
    .fetch_optional(&mut *conn)
    .await?;
    let (shortcode, media_id) = removed.ok_or(AppError::Missing(NOT_FOUND))?;
    let path: Option<String> = sqlx::query_scalar("delete from media where id = $1 returning path")
        .bind(media_id)
        .fetch_optional(&mut *conn)
        .await?;
    audit::record(
        conn,
        Entry::new(Some(actor), "emoji.delete").target(format!(":{shortcode}:")),
    )
    .await?;
    path.ok_or(AppError::NotFound)
}

const NOT_FOUND: &str = "Emote not found.";
