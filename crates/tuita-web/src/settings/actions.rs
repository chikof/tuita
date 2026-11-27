//! What the settings controls call: small procedures over the JSON API.
//!
//! Every procedure answers `Ok(Ok(message))` with the toast to show (empty for none) or
//! `Ok(Err(message))` with the API's message. The browser keeps an optimistic copy of the
//! control and puts it back on `Err`.

use serde::Serialize;
use topcoat::{Result, context::Cx, runtime::procedure};
use tuita_server::dto::{DmPolicy, MeDto};

use crate::{api, l10n::t, session};

/// The answer every settings procedure gives.
pub(crate) type Done<T = String> = Result<std::result::Result<T, String>>;

/// `PATCH /me` with `body`; the API's message on failure.
pub(crate) async fn patch_me(cx: &Cx, body: &impl Serialize) -> std::result::Result<MeDto, String> {
    api::patch::<_, MeDto>(cx, "/me", body)
        .await
        .map_err(|error| error.message)
}

/// Turns an API call's failure into the message a toast shows.
pub(crate) fn failed<T>(
    result: std::result::Result<T, api::ApiError>,
) -> std::result::Result<T, String> {
    result.map_err(|error| error.message)
}

/// Saves one on/off setting: `discoverable`, `showClient` or one notification kind (`like`,
/// `repost`, `reply`, `mention`, `quote`, `follow`).
#[procedure]
pub(crate) async fn save_toggle(cx: &Cx, field: String, on: bool) -> Done {
    let body = match field.as_str() {
        "discoverable" => serde_json::json!({ "discoverable": on }),
        "showClient" => serde_json::json!({ "showClient": on }),
        kind @ ("like" | "repost" | "reply" | "mention" | "quote" | "follow") => {
            let Some(me) = session::me(cx).await? else {
                return Ok(Err("Not signed in".to_owned()));
            };
            let mut prefs = me.settings.notif_prefs;
            match kind {
                "like" => prefs.like = on,
                "repost" => prefs.repost = on,
                "reply" => prefs.reply = on,
                "mention" => prefs.mention = on,
                "quote" => prefs.quote = on,
                _ => prefs.follow = on,
            }
            serde_json::json!({ "notifPrefs": prefs })
        }
        _ => return Ok(Err("Unknown setting".to_owned())),
    };
    Ok(patch_me(cx, &body).await.map(|_| String::new()))
}

/// Saves who may send direct messages: `everyone`, `following` or `none`.
#[procedure]
pub(crate) async fn save_dm_policy(cx: &Cx, policy: String) -> Done {
    let Some(policy) = DmPolicy::parse(&policy) else {
        return Ok(Err("Unknown policy".to_owned()));
    };
    let done = patch_me(cx, &serde_json::json!({ "dmPolicy": policy })).await;
    Ok(done.map(|_| t!(settings_dm_saved).resolve(cx).into_owned()))
}
