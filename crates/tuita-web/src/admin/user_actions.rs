//! What the menu of an account does on the server.

use axum::http::Method;
use serde_json::json;
use topcoat::{Result, context::Cx, runtime::procedure};
use tuita_server::dto::{
    AdminUserDto, PatchAdminUserRequest, PurgedDto, RenameUserRequest, Role, SetOrgRequest,
};
use uuid::Uuid;

use crate::{
    admin::common::{message, tr},
    api,
};

type Outcome = Result<std::result::Result<String, String>>;

/// `/admin/users/id:<id>`, the reference for the account with that id.
fn path_of(id: Uuid) -> String {
    format!("/admin/users/{}", api::enc(&format!("id:{id}")))
}

/// Runs one menu action on the account `id` and answers the toast text. `action` is one of
/// `verify`, `unverify`, `org`, `unorg`, `role:user|moderator|admin`, `suspend`, `unsuspend`,
/// `purge`, `resetName`, `clearBio`, `delete`. `handle` only names the account in the toast.
#[procedure]
pub(crate) async fn run_user_action(
    cx: &Cx,
    id: String,
    action: String,
    handle: String,
) -> Outcome {
    let Ok(id) = id.parse::<Uuid>() else {
        return Ok(Err(tr!(cx, error_title)));
    };
    let path = path_of(id);
    let who = format!("@{}", handle.trim_start_matches('@'));
    let patch = |body: PatchAdminUserRequest| {
        let path = path.clone();
        async move { api::patch::<_, AdminUserDto>(cx, &path, &body).await }
    };
    let none = PatchAdminUserRequest {
        verified: None,
        suspended: None,
        role: None,
    };
    let done = match action.as_str() {
        "verify" => patch(PatchAdminUserRequest {
            verified: Some(true),
            ..none
        })
        .await
        .map(|_| tr!(cx, admin_done_verified, handle: who)),
        "unverify" => patch(PatchAdminUserRequest {
            verified: Some(false),
            ..none
        })
        .await
        .map(|_| tr!(cx, admin_done_unverified)),
        "suspend" => patch(PatchAdminUserRequest {
            suspended: Some(true),
            ..none
        })
        .await
        .map(|_| tr!(cx, admin_done_suspended, handle: who)),
        "unsuspend" => patch(PatchAdminUserRequest {
            suspended: Some(false),
            ..none
        })
        .await
        .map(|_| tr!(cx, admin_done_unsuspended, handle: who)),
        "org" | "unorg" => {
            let org = action == "org";
            api::patch::<_, AdminUserDto>(cx, &format!("{path}/org"), &SetOrgRequest { org })
                .await
                .map(|_| {
                    if org {
                        tr!(cx, admin_done_org, handle: who)
                    } else {
                        tr!(cx, admin_done_unorg)
                    }
                })
        }
        "purge" => api::delete::<PurgedDto>(cx, &format!("{path}/posts"))
            .await
            .map(|purged| tr!(cx, admin_done_purged, count: purged.deleted)),
        "resetName" | "clearBio" => {
            let reset_name = action == "resetName";
            let body = RenameUserRequest {
                handle: None,
                reset_name,
                reset_bio: !reset_name,
            };
            api::post::<_, AdminUserDto>(cx, &format!("{path}/rename"), &body)
                .await
                .map(|_| {
                    if reset_name {
                        tr!(cx, admin_done_name_reset)
                    } else {
                        tr!(cx, admin_done_bio_cleared)
                    }
                })
        }
        "delete" => api::send(cx, Method::DELETE, &path, Some(json!({ "reason": null })))
            .await
            .map(|_| tr!(cx, admin_done_deleted, handle: who)),
        other => match other.strip_prefix("role:").and_then(Role::parse) {
            Some(role) => patch(PatchAdminUserRequest {
                role: Some(role),
                ..none
            })
            .await
            .map(|_| tr!(cx, admin_done_role)),
            None => return Ok(Err(tr!(cx, error_title))),
        },
    };
    Ok(done.map_err(message))
}

/// `POST /admin/users/{ref}/rename`: a new handle (`current` is the one it has now, so an
/// unchanged field changes nothing) and optionally a reset display name. Answers the toast text.
#[procedure]
pub(crate) async fn rename_user(
    cx: &Cx,
    id: String,
    handle: String,
    current: String,
    reset_name: bool,
) -> Outcome {
    let Ok(id) = id.parse::<Uuid>() else {
        return Ok(Err(tr!(cx, error_title)));
    };
    let next = handle.trim().trim_start_matches('@');
    let body = RenameUserRequest {
        handle: (next != current).then(|| next.to_owned()),
        reset_name,
        reset_bio: false,
    };
    Ok(
        api::post::<_, AdminUserDto>(cx, &format!("{}/rename", path_of(id)), &body)
            .await
            .map(|user| tr!(cx, admin_done_renamed, handle: format!("@{}", user.user.handle)))
            .map_err(message),
    )
}
