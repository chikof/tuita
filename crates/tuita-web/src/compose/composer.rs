//! The composer: the post box.

use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, component, element_id, view},
};
use tuita_server::dto::PostDto;

use super::{
    MAX_CW, MAX_TEXT, emoji_picker::emoji_picker, media_tray::media_tray, poll_editor::poll_editor,
    schedule_picker::schedule_picker,
};
use crate::{
    fmt::handle_of,
    icons::Icon,
    l10n::t,
    session,
    shell::parts::account_avatar,
    ui::{
        button::{ButtonSize, ButtonVariant, button},
        char_counter::char_counter,
        dropzone::dropzone,
        icon::icon,
        icon_button::icon_button,
        loader::{LoaderVariant, loader},
        menu::{MenuItem, menu},
    },
};

/// The MIME types the box accepts, as the `data-accept` list of the dropzone around it.
const ACCEPT: &str =
    "image/png,image/jpeg,image/gif,image/webp,video/mp4,video/webm,video/quicktime";

/// The post box for the signed-in visitor: text, content warning, poll, images or a video, an
/// emoji picker, a schedule, a reply policy and a thread of follow-up posts, inside a dropzone.
///
/// `reply_to` makes it a reply box for that post: the note "Replying to @handle" shows and a
/// reply can be neither scheduled nor continued as a thread. `in_dialog` is the box of the compose
/// dialog: it has no fixed target, `composer.js` fills the target (a reply or a quote) when the
/// dialog opens and keeps the draft of each target apart. `autofocus` focuses the text box when
/// the island starts (the dialog focuses it on every open). `placeholder` replaces the default
/// placeholder.
///
/// Everything the box does after load belongs to the `composer.js` island, whose header
/// documents the markup contract and the `tuita-posted` event a finished post fires; the poll
/// editor, the media tray and the pickers have their own components.
#[component]
pub(crate) async fn composer(
    cx: &Cx,
    #[default] reply_to: Option<&PostDto>,
    #[default] autofocus: bool,
    #[default]
    #[into]
    placeholder: Option<&str>,
    #[default] in_dialog: bool,
) -> Result<impl View> {
    let uid = element_id(cx);
    let me = session::viewer(cx).await?.as_ref().map(|v| &v.me.user);
    let max_video_mb = session::instance(cx).await?.max_video_mb;

    let reply = reply_to.map(|post| {
        (
            post.id.to_string(),
            handle_of(&post.author.handle, post.author.domain.as_deref()),
        )
    });
    let is_reply = reply.is_some();
    let slot = reply
        .as_ref()
        .map_or_else(|| "new".to_owned(), |(id, _)| format!("r{id}"));
    let reply_id = reply.as_ref().map(|(id, _)| id.clone());
    let reply_handle = reply.map(|(_, handle)| handle);
    let note = reply_handle.as_deref().map_or_else(String::new, |handle| {
        t!(composer_replying_to, handle: handle)
            .resolve(cx)
            .into_owned()
    });
    let send_label = if is_reply {
        t!(composer_reply).resolve(cx).into_owned()
    } else {
        t!(composer_post).resolve(cx).into_owned()
    };
    let user_id = me.map(|user| user.id.to_string());

    let text_label = t!(composer_text_label).resolve(cx).into_owned();
    let box_placeholder = placeholder.map_or_else(
        || {
            if is_reply {
                t!(composer_reply_placeholder).resolve(cx).into_owned()
            } else {
                t!(composer_placeholder).resolve(cx).into_owned()
            }
        },
        str::to_owned,
    );
    let suggest_id = format!("{uid}-suggest");
    let suggest_label = t!(composer_suggestions).resolve(cx).into_owned();
    let cw_label = t!(composer_cw_label).resolve(cx).into_owned();
    let cw_toggle = t!(composer_cw_toggle).resolve(cx).into_owned();
    let cw_placeholder = t!(composer_cw_placeholder).resolve(cx).into_owned();
    let options = t!(composer_options).resolve(cx).into_owned();
    let add_images = t!(composer_add_images).resolve(cx).into_owned();
    let add_video = t!(composer_add_video).resolve(cx).into_owned();
    let add_poll = t!(composer_add_poll).resolve(cx).into_owned();
    let sensitive = t!(composer_sensitive_toggle).resolve(cx).into_owned();
    let schedule_clear = t!(composer_schedule_clear).resolve(cx).into_owned();
    let extra_placeholder = t!(composer_extra_placeholder).resolve(cx).into_owned();
    let policy_label = t!(composer_policy_label).resolve(cx).into_owned();
    let everyone = t!(policy_everyone).resolve(cx).into_owned();
    let following = t!(policy_following).resolve(cx).into_owned();
    let mentioned = t!(policy_mentioned).resolve(cx).into_owned();
    let current_policy = everyone.clone();
    let max_cw = MAX_CW.to_string();
    let emoji_id = format!("{uid}-emoji");
    let schedule_id = format!("{uid}-schedule");
    let policy_id = format!("{uid}-policy");
    let context = if in_dialog { "dialog" } else { "inline" };

    Ok(view! {
        dropzone(
            accept: ACCEPT,
            <form
                class="composer"
                novalidate=""
                data-c="composer"
                data-island="composer"
                data-context=(context)
                data-slot=(&slot)
                data-user=(user_id)
                data-reply-to=(reply_id)
                data-reply-handle=(reply_handle)
                data-placeholder=(placeholder)
                data-autofocus=(autofocus.then_some(""))
            >
                if let Some(user) = me {
                    <div class="me" aria-hidden="true">
                        account_avatar(
                            handle: &user.handle,
                            src: user.avatar_url.as_deref(),
                            shape: user.avatar_shape,
                            ink: user.ink,
                            size: 40
                        )
                    </div>
                }
                <div class="main">
                    <p class="note" data-role="note" hidden=(note.is_empty())>
                        (&note)
                    </p>
                    <input
                        class="cw"
                        data-role="cw"
                        maxlength=(&max_cw)
                        placeholder=(&cw_placeholder)
                        aria-label=(&cw_label)
                        autocomplete="off"
                        hidden=""
                    >
                    <div class="field">
                        <textarea
                            data-role="text"
                            rows="2"
                            role="combobox"
                            aria-expanded="false"
                            aria-controls=(&suggest_id)
                            aria-autocomplete="list"
                            aria-label=(&text_label)
                            placeholder=(&box_placeholder)
                        ></textarea>
                        <ul
                            id=(&suggest_id)
                            class="suggest"
                            role="listbox"
                            aria-label=(&suggest_label)
                            data-role="suggest"
                            hidden=""
                        ></ul>
                    </div>
                    media_tray(max_video_mb: max_video_mb)
                    poll_editor()
                    <div data-role="quote" hidden=""></div>
                    <div class="extras" data-role="extras"></div>
                    <p class="chip" data-role="chip" hidden="">
                        icon(name: Icon::Calendar, size: 16)
                        <span data-role="chip-text"></span>
                        icon_button(
                            glyph: Icon::Close,
                            size: ButtonSize::Sm,
                            label: &schedule_clear,
                            attrs: attributes! { data-act="chip-clear" }
                        )
                    </p>
                    <p class="error" role="alert" data-role="error" hidden=""></p>
                    <div class="bar">
                        <div class="tools" role="toolbar" aria-label=(&options)>
                            icon_button(
                                glyph: Icon::Image,
                                label: &add_images,
                                size: ButtonSize::Sm,
                                attrs: attributes! { data-act="images" }
                            )
                            icon_button(
                                glyph: Icon::Video,
                                label: &add_video,
                                size: ButtonSize::Sm,
                                attrs: attributes! { data-act="video" }
                            )
                            icon_button(
                                glyph: Icon::ChartBarBig,
                                label: &add_poll,
                                pressed: false,
                                size: ButtonSize::Sm,
                                attrs: attributes! { data-act="poll" }
                            )
                            emoji_picker(id: &emoji_id)
                            icon_button(
                                glyph: Icon::WarningDiamond,
                                label: &cw_toggle,
                                pressed: false,
                                size: ButtonSize::Sm,
                                attrs: attributes! { data-act="cw" }
                            )
                            icon_button(
                                glyph: Icon::EyeOff,
                                label: &sensitive,
                                pressed: false,
                                size: ButtonSize::Sm,
                                attrs: attributes! { data-act="sensitive" hidden="" }
                            )
                            <span
                                class="schedule"
                                data-role="schedule"
                                hidden=(is_reply)
                            >
                                schedule_picker(id: &schedule_id)
                            </span>
                            menu(
                                id: &policy_id,
                                label: &policy_label,
                                items: vec![
                                    MenuItem {
                                        id: "everyone",
                                        label: &everyone,
                                        attrs: attributes! { data-policy="everyone" },
                                        ..MenuItem::default()
                                    },
                                    MenuItem {
                                        id: "following",
                                        label: &following,
                                        attrs: attributes! { data-policy="following" },
                                        ..MenuItem::default()
                                    },
                                    MenuItem {
                                        id: "mentioned",
                                        label: &mentioned,
                                        attrs: attributes! { data-policy="mentioned" },
                                        ..MenuItem::default()
                                    },
                                ],
                                trigger: view! {
                                    icon(name: Icon::Users, size: 16)
                                    <span data-role="policy-label">(current_policy)</span>
                                }.into(

                                )
                            )
                        </div>
                        <div class="send">
                            <span data-role="add-extra" hidden=(is_reply)>
                                button(
                                    variant: ButtonVariant::Ghost,
                                    size: ButtonSize::Sm,
                                    attrs: attributes! { data-act="add-extra" },
                                    icon(name: Icon::Plus, size: 16)
                                    (t!(composer_add_post))
                                )
                            </span>
                            <span data-role="counter" hidden="">
                                char_counter(value: 0, max: MAX_TEXT)
                            </span>
                            button(
                                size: ButtonSize::Sm,
                                submit: true,
                                disabled: true,
                                attrs: attributes! { data-role="send" },
                                (&send_label)
                            )
                        </div>
                    </div>
                </div>
                <input
                    type="file"
                    hidden=""
                    multiple=""
                    accept="image/png,image/jpeg,image/gif,image/webp"
                    aria-label=(&add_images)
                    data-role="images"
                >
                <input
                    type="file"
                    hidden=""
                    accept="video/mp4,video/webm,video/quicktime"
                    aria-label=(&add_video)
                    data-role="video"
                >
                <template data-tpl="extra">
                    <div class="extra">
                        <textarea
                            data-extra=""
                            rows="2"
                            placeholder=(&extra_placeholder)
                        ></textarea>
                        icon_button(
                            glyph: Icon::Close,
                            size: ButtonSize::Sm,
                            label: "",
                            attrs: attributes! { data-act="remove-extra" }
                        )
                    </div>
                </template>
                <template data-tpl="spinner">
                    loader(variant: LoaderVariant::Dots, size: 16)
                </template>
            </form>
        )
    })
}

#[cfg(test)]
mod test {
    use super::ACCEPT;

    #[test]
    fn the_dropzone_accepts_every_type_the_tray_does() {
        for kind in ["png", "jpeg", "gif", "webp", "mp4", "webm", "quicktime"] {
            assert!(ACCEPT.contains(kind), "{kind}");
        }
    }
}
