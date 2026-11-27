//! `GET /instance`: what a visitor needs before signing in.

use crate::auth::bearer::TokenAuth;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::header::ACCEPT_LANGUAGE;
use axum::routing::get;
use axum::{Extension, Json, Router};
use serde_json::Value;
use tuita_core::locale::detect_language;

use crate::db::sessions::ClientMeta;
use crate::db::{announcements, settings};
use crate::dto::{AnnouncementDto, InstanceDto, InstanceStats};
use crate::error::AppError;
use crate::mastodon::{self, instance::registrations_open};
use crate::state::AppState;

pub(super) fn router() -> Router<AppState> {
    Router::new().route("/instance", get(instance)).route(
        "/announcements",
        get(mastodon::by_credential(
            shown_announcements,
            mastodon::misc::announcement_list,
        )),
    )
}

async fn instance(
    State(state): State<AppState>,
    meta: ClientMeta,
    headers: HeaderMap,
    token: Option<Extension<TokenAuth>>,
) -> Result<Json<Value>, AppError> {
    let config = state.config();
    let settings = settings::load(state.db()).await?;
    let users: i64 = sqlx::query_scalar(
        "select count(*) from users
         where domain is null and handle <> 'tuita' and suspended_at is null and deleted_at is null",
    )
    .fetch_one(state.db())
    .await?;
    let posts: i64 = sqlx::query_scalar(
        "select count(*) from posts p join users u on u.id = p.author_id
         where u.domain is null and p.deleted_at is null",
    )
    .fetch_one(state.db())
    .await?;
    let registrations_open = registrations_open(&state, settings.signups).await?;
    let dto = InstanceDto {
        name: settings.name,
        description: settings.description,
        domain: config.domain().to_owned(),
        version: env!("CARGO_PKG_VERSION").to_owned(),
        signups: settings.signups,
        registrations_open,
        contact: settings.contact,
        rules: settings.rules,
        min_age: settings.min_age,
        source_url: config.source_url.as_deref().map(str::to_owned),
        max_video_mb: config.max_video_mb.get(),
        stats: InstanceStats { users, posts },
        detected_language: detect_language(
            meta.country.as_deref(),
            headers
                .get(ACCEPT_LANGUAGE)
                .and_then(|value| value.to_str().ok()),
        )
        .to_owned(),
    };
    let oauth = token.is_some_and(|Extension(token)| token.app_id.is_some());
    Ok(Json(
        mastodon::instance::v1_object(&state, &dto, oauth).await?,
    ))
}

/// `GET /announcements`: the banners everyone sees.
async fn shown_announcements(
    State(state): State<AppState>,
) -> Result<Json<Vec<AnnouncementDto>>, AppError> {
    Ok(Json(announcements::shown(state.db()).await?))
}
