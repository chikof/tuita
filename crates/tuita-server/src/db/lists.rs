//! Database access for lists: the lists of an owner, their members and the list timeline.

use std::collections::HashMap;

use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

use super::users::{COLS, UserRow, dtos, refs, user_ref};
use super::{emoji, listable_post, posts, visible_author, visible_post, word_visible};
use crate::cursor::{EdgeCursor, Limit, page};
use crate::dto::{ListDto, Page, PostDto, UserDto};
use crate::error::AppError;

/// Most lists one account may own.
pub const MAX_LISTS: i64 = 50;
/// Most members one list may hold.
pub const MAX_MEMBERS: i64 = 500;

#[derive(FromRow)]
struct ListRow {
    list_id: Uuid,
    list_name: String,
    list_description: String,
    list_private: bool,
    list_created_at: OffsetDateTime,
    member_count: i64,
    has_member: Option<bool>,
    #[sqlx(flatten)]
    owner: UserRow,
}

impl ListRow {
    fn dto(&self, viewer: Uuid) -> ListDto {
        ListDto {
            id: self.list_id,
            name: self.list_name.clone(),
            description: self.list_description.clone(),
            private: self.list_private,
            created_at: self.list_created_at,
            owner: user_ref(&self.owner),
            is_owner: self.owner.id == viewer,
            members: self.member_count,
            has_member: self.has_member,
        }
    }
}

/// Selects a [`ListRow`]; `$2` is the account whose membership `has_member` reports, or null.
fn select_sql() -> String {
    format!(
        "select l.id as list_id, l.name as list_name, l.description as list_description,
                l.private as list_private, l.created_at as list_created_at,
                (select count(*) from list_members lm where lm.list_id = l.id) as member_count,
                case when $2::uuid is null then null
                     else exists (select 1 from list_members hm
                                   where hm.list_id = l.id and hm.user_id = $2) end as has_member,
                {COLS}
           from lists l join users u on u.id = l.owner_id"
    )
}

/// The lists `viewer` owns, newest first. With `member`, each says whether that account is in it.
///
/// # Errors
///
/// Database failures.
pub async fn owned(
    db: &PgPool,
    viewer: Uuid,
    member: Option<Uuid>,
) -> Result<Vec<ListDto>, AppError> {
    let sql = format!("{} where l.owner_id = $1 order by l.id desc", select_sql());
    let rows: Vec<ListRow> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(viewer)
        .bind(member)
        .fetch_all(db)
        .await?;
    let mut lists: Vec<ListDto> = rows.iter().map(|row| row.dto(viewer)).collect();
    let mut owners: Vec<_> = lists.iter_mut().map(|list| &mut list.owner).collect();
    emoji::hydrate(db, &mut owners).await?;
    Ok(lists)
}

/// List `id` as `viewer` may see it: not somebody else's private list.
///
/// # Errors
///
/// Database failures.
pub async fn find(db: &PgPool, viewer: Uuid, id: Uuid) -> Result<Option<ListDto>, AppError> {
    let sql = format!(
        "{} where l.id = $3 and (not l.private or l.owner_id = $1)",
        select_sql()
    );
    let row: Option<ListRow> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(viewer)
        .bind(None::<Uuid>)
        .bind(id)
        .fetch_optional(db)
        .await?;
    let mut list = row.map(|row| row.dto(viewer));
    if let Some(list) = &mut list {
        emoji::hydrate(db, std::slice::from_mut(&mut list.owner)).await?;
    }
    Ok(list)
}

/// Creates a list.
///
/// # Errors
///
/// `400 You can have up to 50 lists.` or a database failure.
pub async fn create(
    db: &PgPool,
    owner: Uuid,
    name: &str,
    description: &str,
    private: bool,
) -> Result<ListDto, AppError> {
    let id = Uuid::now_v7();
    let mut tx = db.begin().await?;
    sqlx::query("select pg_advisory_xact_lock(hashtextextended('lists:' || $1::text, 0))")
        .bind(owner)
        .execute(&mut *tx)
        .await?;
    let owned: i64 = sqlx::query_scalar("select count(*) from lists where owner_id = $1")
        .bind(owner)
        .fetch_one(&mut *tx)
        .await?;
    if owned >= MAX_LISTS {
        return Err(AppError::bad_request(format!(
            "You can have up to {MAX_LISTS} lists."
        )));
    }
    sqlx::query(
        "insert into lists (id, owner_id, name, description, private) values ($1, $2, $3, $4, $5)",
    )
    .bind(id)
    .bind(owner)
    .bind(name)
    .bind(description)
    .bind(private)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    find(db, owner, id)
        .await?
        .ok_or(AppError::Missing("List not found."))
}

/// Changes the fields that are present.
///
/// # Errors
///
/// Database failures.
pub async fn update(
    db: &PgPool,
    id: Uuid,
    name: Option<&str>,
    description: Option<&str>,
    private: Option<bool>,
) -> Result<(), AppError> {
    sqlx::query(
        "update lists set name = coalesce($2, name), description = coalesce($3, description),
                private = coalesce($4, private) where id = $1",
    )
    .bind(id)
    .bind(name)
    .bind(description)
    .bind(private)
    .execute(db)
    .await?;
    Ok(())
}

/// Deletes a list with its members.
///
/// # Errors
///
/// Database failures.
pub async fn delete(db: &PgPool, id: Uuid) -> Result<(), AppError> {
    sqlx::query("delete from lists where id = $1")
        .bind(id)
        .execute(db)
        .await?;
    Ok(())
}

/// Adds `user` to list `id`; adding a member twice does nothing.
///
/// # Errors
///
/// `400 Lists can have up to 500 members.` or a database failure.
pub async fn add_member(db: &PgPool, id: Uuid, user: Uuid) -> Result<(), AppError> {
    let mut tx = db.begin().await?;
    sqlx::query("select pg_advisory_xact_lock(hashtextextended('list:' || $1::text, 0))")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    let (members, present): (i64, bool) = sqlx::query_as(
        "select count(*), coalesce(bool_or(user_id = $2), false)
           from list_members where list_id = $1",
    )
    .bind(id)
    .bind(user)
    .fetch_one(&mut *tx)
    .await?;
    if !present {
        if members >= MAX_MEMBERS {
            return Err(AppError::bad_request(format!(
                "Lists can have up to {MAX_MEMBERS} members."
            )));
        }
        sqlx::query("insert into list_members (list_id, user_id) values ($1, $2)")
            .bind(id)
            .bind(user)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(())
}

/// Removes `user` from list `id`.
///
/// # Errors
///
/// Database failures.
pub async fn remove_member(db: &PgPool, id: Uuid, user: Uuid) -> Result<(), AppError> {
    sqlx::query("delete from list_members where list_id = $1 and user_id = $2")
        .bind(id)
        .bind(user)
        .execute(db)
        .await?;
    Ok(())
}

#[derive(FromRow)]
struct MemberRow {
    #[sqlx(flatten)]
    user: UserRow,
    added_at: OffsetDateTime,
}

/// One page of the members of list `id`, newest first, as profile cards for `viewer`.
///
/// # Errors
///
/// Database failures.
pub async fn members(
    db: &PgPool,
    viewer: Uuid,
    id: Uuid,
    before: Option<EdgeCursor>,
    limit: Limit,
) -> Result<Page<UserDto>, AppError> {
    let sql = format!(
        "select {COLS}, m.created_at as added_at
           from list_members m join users u on u.id = m.user_id
          where m.list_id = $1 and u.suspended_at is null and u.deleted_at is null
            and ($2::timestamptz is null or (m.created_at, m.user_id) < ($2, $3::uuid))
          order by m.created_at desc, m.user_id desc limit $4"
    );
    let rows: Vec<MemberRow> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(id)
        .bind(before.map(|cursor| cursor.at))
        .bind(before.map(|cursor| cursor.id))
        .bind(limit.fetch())
        .fetch_all(db)
        .await?;
    let Page { items, next } = page(rows, limit, |row| {
        EdgeCursor::new(row.added_at, row.user.id).encode()
    });
    let users: Vec<UserRow> = items.into_iter().map(|row| row.user).collect();
    Ok(Page {
        items: dtos(db, viewer, &users).await?,
        next,
    })
}

/// One row of the list timeline: an original post, or a repost of one.
#[derive(FromRow)]
struct Entry {
    /// Post id for an original, repost row id for a repost: the keyset of the timeline.
    key: Uuid,
    post_id: Uuid,
    reposter_id: Option<Uuid>,
}

/// One page of the list timeline: members' top-level posts, their replies to other members and
/// their reposts, newest first. `before` is the key of the last entry of the previous page.
///
/// # Errors
///
/// Database failures.
pub async fn timeline(
    db: &PgPool,
    viewer: Uuid,
    id: Uuid,
    before: Option<Uuid>,
    limit: Limit,
) -> Result<Page<PostDto>, AppError> {
    const MEMBERS: &str = "select user_id from list_members where list_id = $2";
    let author_ok = visible_author("a", "$1");
    let reposter_ok = visible_author("ru", "$1");
    let words_ok = word_visible("p", "$1");
    let listable = listable_post("p");
    let visible = visible_post("p", "$1");
    let sql = format!(
        "select key, post_id, reposter_id from (
            select p.id as key, p.id as post_id, null::uuid as reposter_id
              from posts p join users a on a.id = p.author_id
             where p.deleted_at is null and p.author_id in ({MEMBERS})
               and (p.reply_to_id is null
                    or exists (select 1 from posts rp
                                where rp.id = p.reply_to_id and rp.author_id in ({MEMBERS})))
               and {author_ok} and {words_ok} and {listable} and {visible}
            union all
            select r.id, r.post_id, r.user_id
              from reposts r
              join users ru on ru.id = r.user_id
              join posts p on p.id = r.post_id
              join users a on a.id = p.author_id
             where r.user_id in ({MEMBERS}) and p.deleted_at is null
               and {reposter_ok} and {author_ok} and {words_ok} and {listable} and {visible}
         ) e
         where $3::uuid is null or e.key < $3
         order by e.key desc limit $4"
    );
    let rows: Vec<Entry> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(viewer)
        .bind(id)
        .bind(before)
        .bind(limit.fetch())
        .fetch_all(db)
        .await?;
    let entries = page(rows, limit, |entry| entry.key.to_string());

    let mut post_ids: Vec<Uuid> = entries.items.iter().map(|entry| entry.post_id).collect();
    post_ids.sort_unstable();
    post_ids.dedup();
    let loaded: HashMap<Uuid, PostDto> = posts::load(db, viewer, &post_ids, None)
        .await?
        .into_iter()
        .map(|post| (post.id, post))
        .collect();
    let reposter_ids: Vec<Uuid> = entries
        .items
        .iter()
        .filter_map(|entry| entry.reposter_id)
        .collect();
    let reposters = refs(db, &reposter_ids).await?;

    let items = entries
        .items
        .iter()
        .filter_map(|entry| {
            let mut post = loaded.get(&entry.post_id)?.clone();
            post.entry_id = entry.key;
            if let Some(reposter) = entry.reposter_id {
                post.reposted_by = Some(reposters.get(&reposter)?.clone());
            }
            Some(post)
        })
        .collect();
    Ok(Page {
        items,
        next: entries.next,
    })
}
