//! The emotes tab: upload custom emotes, change their category, delete them.
//!
//! The upload form is `[data-island="emote-form"]` around `[data-island="multipart-form"]`
//! (`admin.js`, `upload.js`); the list is a shard.

use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, Signal, procedure, shard, signal},
    view::{View, attributes, component, view},
};
use tuita_server::dto::{EmojiDto, OkDto, PatchEmojiRequest};

use crate::{
    admin::{
        common::{message, tr},
        setting_card::setting_card,
    },
    api,
    icons::Icon,
    l10n::{strip_isolates, t},
    ui::{
        button::{ButtonSize, ButtonVariant, button},
        confirm_dialog::confirm_dialog,
        dropzone::dropzone,
        icon_button::icon_button,
        text_field::text_field,
    },
};

/// The picture types the server takes for an emote.
const ACCEPT: &str = "image/png,image/jpeg,image/gif,image/webp";

/// `PATCH /admin/emojis/{shortcode}`: the category (`value`, empty for none) unless it is what
/// `current` already says. Answers the toast text, empty when nothing changed.
#[procedure]
pub(crate) async fn save_emote_category(
    cx: &Cx,
    shortcode: String,
    current: String,
    value: String,
) -> Result<std::result::Result<String, String>> {
    let next = value.trim();
    if next == current.trim() {
        return Ok(Ok(String::new()));
    }
    let body = PatchEmojiRequest {
        category: (!next.is_empty()).then(|| next.to_owned()),
    };
    let path = format!("/admin/emojis/{}", api::enc(&shortcode));
    Ok(api::patch::<_, EmojiDto>(cx, &path, &body)
        .await
        .map(|_| tr!(cx, admin_emote_saved))
        .map_err(message))
}

/// `DELETE /admin/emojis/{shortcode}`. Answers the toast text.
#[procedure]
pub(crate) async fn delete_emote(
    cx: &Cx,
    shortcode: String,
) -> Result<std::result::Result<String, String>> {
    let path = format!("/admin/emojis/{}", api::enc(&shortcode));
    Ok(api::delete::<OkDto>(cx, &path)
        .await
        .map(|_| tr!(cx, admin_emote_deleted))
        .map_err(message))
}

/// One emote with the words of its controls.
struct Row {
    shortcode: String,
    url: String,
    category: String,
    label: String,
    delete_label: String,
    dialog: String,
    delete_body: String,
}

/// The emotes of the instance, from the API again whenever `revision` changes.
#[shard]
async fn emote_list(cx: &Cx, revision: Signal<u32>) -> Result<impl View> {
    let _ = revision.get();
    let items = api::get::<Vec<EmojiDto>>(cx, "/admin/emojis").await?;
    let rows: Vec<Row> = items
        .iter()
        .map(|emote| Row {
            shortcode: emote.shortcode.clone(),
            url: emote.url.clone(),
            category: emote.category.clone().unwrap_or_default(),
            label: tr!(cx, admin_emote_category_for, shortcode: emote.shortcode.as_str()),
            delete_label: tr!(cx, admin_emote_delete, shortcode: emote.shortcode.as_str()),
            dialog: format!("delete-emote-{}", emote.shortcode),
            delete_body: tr!(cx, admin_emote_delete_body, shortcode: emote.shortcode.as_str()),
        })
        .collect();
    let placeholder = tr!(cx, admin_emote_no_category);
    let delete_title = tr!(cx, admin_emote_delete_title);
    let delete_confirm = tr!(cx, admin_emote_delete_confirm);
    Ok(view! {
        if rows.is_empty() {
            <p class="dim">(t!(admin_emote_none))</p>
        } else {
            <ul class="items">
                #[key(row.shortcode.as_bytes())]
                for row in rows {
                    let saved = row.shortcode.clone();
                    let was = row.category.clone();
                    let doomed = row.shortcode.clone();
                    let token = format!(":{}:", row.shortcode);
                    <li data-emote=(&row.shortcode)>
                        <img
                            class="thumb"
                            src=(&row.url)
                            alt=(&token)
                            title=(&token)
                            height="32"
                            loading="lazy"
                            decoding="async"
                        >
                        <code>(&token)</code>
                        <input
                            class="cat"
                            value=(&row.category)
                            aria-label=(&row.label)
                            placeholder=(&placeholder)
                            maxlength="32"
                            autocomplete="off"
                            @change=$(async |e: Event| {
                                let done = save_emote_category(saved, was, e.target.value).await;
                                if done.is_ok() {
                                    let msg = done.unwrap();
                                    raw!(
                                        "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:''+${msg}}));",
                                        drop(msg),
                                    );
                                    revision.increment();
                                    false
                                } else {
                                    let msg = done.unwrap_err();
                                    raw!(
                                        "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:''+${msg}}));",
                                        drop(msg),
                                    );
                                    revision.increment();
                                    false
                                }
                            })
                        >
                        icon_button(
                            glyph: Icon::Trash,
                            label: &row.delete_label,
                            attrs: attributes! { command="show-modal" commandfor=(&row.dialog) }
                        )
                        confirm_dialog(
                            id: &row.dialog,
                            title: &delete_title,
                            body: &row.delete_body,
                            confirm_label: &delete_confirm,
                            danger: true,
                            action: $(async |_e: Event| {
                                let done = delete_emote(doomed).await;
                                if done.is_ok() {
                                    let msg = done.unwrap();
                                    raw!(
                                        "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:''+${msg}}));",
                                        drop(msg),
                                    );
                                    revision.increment();
                                    false
                                } else {
                                    let msg = done.unwrap_err();
                                    raw!(
                                        "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:''+${msg}}))",
                                        msg.is_empty(),
                                    )
                                }
                            })
                        )
                    </li>
                }
            </ul>
        }
    })
}

#[component]
pub(crate) async fn emotes(cx: &Cx) -> Result<impl View> {
    let revision = signal(cx, || 0u32);
    let title = tr!(cx, admin_emote_title);
    let desc = tr!(cx, admin_emote_desc);
    let list_title = tr!(cx, admin_emote_list);
    let shortcode_label = tr!(cx, admin_emote_shortcode);
    let shortcode_hint = tr!(cx, admin_emote_shortcode_hint);
    let category_label = tr!(cx, admin_emote_category);
    let category_hint = tr!(cx, admin_emote_category_hint);
    let file_label = tr!(cx, admin_emote_file);
    let file_hint = tr!(cx, admin_emote_file_hint);
    let choose = tr!(cx, admin_emote_choose_file);
    let change = tr!(cx, admin_emote_change_file);
    let invalid = tr!(cx, admin_emote_invalid);
    let file_type = tr!(cx, admin_emote_file_type);
    let file_size = tr!(cx, admin_emote_file_size);
    let failed = tr!(cx, admin_emote_upload_failed);
    let added = strip_isolates(&tr!(cx, admin_emote_uploaded, shortcode: "{shortcode}"));
    Ok(view! {
        <div class="tab" data-c="admin-emotes">
            setting_card(
                title: &title,
                description: desc.as_str(),
                <div
                    data-island="emote-form"
                    data-invalid=(&invalid)
                    data-file-type=(&file_type)
                    data-file-size=(&file_size)
                    data-hint=(&file_hint)
                    data-choose=(&choose)
                    data-change=(&change)
                    data-added=(&added)
                >
                    dropzone(
                        accept: ACCEPT,
                        <form
                            class="form"
                            novalidate=""
                            data-island="multipart-form"
                            data-action="/api/v1/admin/emojis"
                            data-failure=(&failed)
                        >
                            text_field(
                                label: &shortcode_label,
                                hint: shortcode_hint.as_str(),
                                attrs: attributes! {
                                    name="shortcode"
                                    maxlength="40"
                                    autocapitalize="off"
                                    autocomplete="off"
                                    spellcheck="false"
                                    required=""
                                }
                            )
                            text_field(
                                label: &category_label,
                                hint: category_hint.as_str(),
                                attrs: attributes! { name="category" maxlength="32" autocomplete="off" }
                            )
                            <div class="file">
                                <div
                                    class="preview"
                                    data-role="preview"
                                    aria-hidden="true"
                                ></div>
                                <div class="pick">
                                    button(
                                        variant: ButtonVariant::Outline,
                                        size: ButtonSize::Sm,
                                        attrs: attributes! { data-role="pick" },
                                        (&choose)
                                    )
                                    <p class="dim" data-role="file-note">(&file_hint)</p>
                                </div>
                                <input
                                    type="file"
                                    name="file"
                                    accept=(ACCEPT)
                                    aria-label=(&file_label)
                                    hidden=""
                                >
                            </div>
                            <div class="end">
                                button(
                                    submit: true,
                                    disabled: true,
                                    (t!(admin_emote_upload))
                                )
                            </div>
                        </form>
                    )
                </div>
            )
            setting_card(title: &list_title, emote_list(revision: revision.clone()))
        </div>
    })
}
