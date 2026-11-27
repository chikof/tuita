//! Direct messages: the conversation list, one conversation, sending, read receipts, deleting.

use axum::extract::{Path, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::Deserialize;
use tuita_core::sealed;
use uuid::Uuid;

use super::auth::text_field;
use crate::auth::{ApiJson, ApiQuery, CurrentUser};
use crate::cursor::{PageQuery, parse_id};
use crate::db::message_keys;
use crate::db::messages::{self, Content};
use crate::db::notifications::publish_badges;
use crate::db::users::{self, user_ref};
use crate::dto::{
    ConversationDto, ConversationViewDto, MessageDto, MessageEventDto, MessageKeyDto, MessageMode,
    MyMessageKeyDto, OkDto, Page, ReadEventDto, ReadKind, SendMessageRequest, SetMessageKeyRequest,
    StreamEvent,
};
use crate::error::AppError;
use crate::federation::lookup::{find_user, forbid_suspended};
use crate::mastodon::{self, by_credential};
use crate::state::AppState;

/// Longest message, in characters.
const MAX_MESSAGE_CHARS: usize = 2000;
/// Characters of a message that go into the live `message` event.
const PREVIEW_CHARS: usize = 280;
/// The `422` for plaintext sent to a sealed conversation.
const MUST_BE_SEALED: &str = "Messages to this account must be end-to-end encrypted.";

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/conversations",
            get(by_credential(list, mastodon::conversations::list)),
        )
        .route("/conversations/{handle}", get(view).post(send))
        .route(
            "/conversations/{handle}/read",
            post(by_credential(read, mastodon::conversations::read)),
        )
        .route("/conversations/{handle}/messages/{id}", delete(remove))
        .route("/me/message-key", get(my_key).put(set_my_key))
}

#[derive(Deserialize)]
struct ViewQuery {
    before: Option<String>,
    limit: Option<u32>,
    after: Option<String>,
}

async fn list(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    ApiQuery(query): ApiQuery<PageQuery>,
) -> Result<Json<Page<ConversationDto>>, AppError> {
    let page =
        messages::conversations(state.db(), auth.user.id, query.id_cursor()?, query.limit()?)
            .await?;
    Ok(Json(page))
}

async fn my_key(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
) -> Result<Json<MyMessageKeyDto>, AppError> {
    Ok(Json(MyMessageKeyDto {
        key: message_keys::get(state.db(), auth.user.id).await?,
    }))
}

async fn set_my_key(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    ApiJson(body): ApiJson<SetMessageKeyRequest>,
) -> Result<Json<MessageKeyDto>, AppError> {
    state.limits().check_message_key(auth.user.id)?;
    sealed::validate_public_key(&body.public_key)
        .map_err(|_| AppError::unprocessable("That is not a valid message key."))?;
    sealed::validate_backup(&body.backup)
        .map_err(|_| AppError::unprocessable("That is not a valid key backup."))?;
    let key = message_keys::set(
        state.db(),
        auth.user.id,
        &body.public_key,
        &body.backup,
        body.replace,
    )
    .await?
    .ok_or_else(|| AppError::conflict("You already have a message key."))?;
    Ok(Json(key))
}

async fn view(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(handle): Path<String>,
    ApiQuery(query): ApiQuery<ViewQuery>,
) -> Result<Json<ConversationViewDto>, AppError> {
    let db = state.db();
    let other = find_user(&state, &auth.user, &handle).await?;
    let after = query
        .after
        .as_deref()
        .map(|raw| Uuid::parse_str(raw).map_err(|_| AppError::bad_request("Invalid cursor.")))
        .transpose()?;
    let paging = PageQuery {
        before: query.before,
        limit: query.limit,
    };
    let page = messages::thread(
        db,
        auth.user.id,
        other.id,
        paging.id_cursor()?,
        after,
        paging.limit()?,
    )
    .await?;
    let route = messages::route(db, state.config(), auth.user.id, &other).await?;
    Ok(Json(ConversationViewDto {
        user: users::dto(db, auth.user.id, &other).await?,
        messages: page.items,
        next: page.next,
        blocked_reason: messages::blocked_reason(db, auth.user.id, &other).await?,
        mode: route.mode,
        peer_key: route.peer_key,
    }))
}

async fn send(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(handle): Path<String>,
    ApiJson(body): ApiJson<SendMessageRequest>,
) -> Result<Json<MessageDto>, AppError> {
    state.limits().check_dm(auth.user.id)?;
    let db = state.db();
    forbid_suspended(&state, &handle, "You can’t message this account.").await?;
    let other = find_user(&state, &auth.user, &handle).await?;
    if let Some(reason) = messages::blocked_reason(db, auth.user.id, &other).await? {
        return Err(AppError::forbidden(reason));
    }
    let route = messages::route(db, state.config(), auth.user.id, &other).await?;
    let plain_text;
    let content = match route.mode {
        MessageMode::Unavailable => {
            return Err(AppError::conflict(if route.peer_key.is_none() {
                format!("@{} hasn't turned on encrypted messages yet.", other.handle)
            } else {
                "Turn on encrypted messages first.".to_owned()
            }));
        }
        MessageMode::Sealed => {
            let Some(ciphertext) = body.ciphertext.as_deref() else {
                return Err(AppError::unprocessable(MUST_BE_SEALED));
            };
            if body.text.as_deref().is_some_and(|text| !text.is_empty()) {
                return Err(AppError::unprocessable(MUST_BE_SEALED));
            }
            let keys = sealed::validate_sealed(ciphertext)
                .map_err(|_| AppError::unprocessable("That is not a valid encrypted message."))?;
            Content::Sealed {
                ciphertext,
                sender_key: keys.sender,
                recipient_key: keys.recipient,
            }
        }
        MessageMode::Plain => {
            if body.ciphertext.is_some() {
                return Err(AppError::unprocessable(
                    "This conversation is not encrypted.",
                ));
            }
            plain_text = text_field(
                body.text.as_deref().unwrap_or(""),
                "Message",
                1,
                MAX_MESSAGE_CHARS,
            )?;
            Content::Plain(&plain_text)
        }
    };
    let message = messages::send(db, state.config(), auth.user.id, other.id, content).await?;
    let (preview, encrypted) = match content {
        Content::Plain(text) => (text.chars().take(PREVIEW_CHARS).collect(), false),
        Content::Sealed { .. } => (String::new(), true),
    };
    state.hub().publish(
        other.id,
        StreamEvent::Message(Box::new(MessageEventDto {
            from: user_ref(&auth.user),
            text: preview,
            encrypted,
            created_at: message.created_at,
        })),
    );
    publish_badges(&state, other.id).await;
    Ok(Json(message))
}

async fn read(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(handle): Path<String>,
) -> Result<Json<OkDto>, AppError> {
    let other = find_user(&state, &auth.user, &handle).await?;
    if messages::mark_read(state.db(), auth.user.id, other.id).await? > 0 {
        state.hub().publish(
            auth.user.id,
            StreamEvent::Read(ReadEventDto {
                kind: ReadKind::Messages,
            }),
        );
        publish_badges(&state, auth.user.id).await;
    }
    Ok(Json(OkDto { ok: true }))
}

async fn remove(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path((_handle, id)): Path<(String, String)>,
) -> Result<Json<OkDto>, AppError> {
    const MISSING: &str = "Message not found.";
    let id = parse_id(&id, MISSING)?;
    let deleted = messages::delete(state.db(), state.config(), auth.user.id, id)
        .await?
        .ok_or(AppError::Missing(MISSING))?;
    if deleted.was_unread {
        publish_badges(&state, deleted.recipient).await;
    }
    Ok(Json(OkDto { ok: true }))
}
