//! Gallery sections: files, media, polls, composer attachments, chat and lists.
//!
//! The sections are server rendered and the
//! small amount of behaviour a demo needs (a sample picture for the cropper and the tray, the
//! lightbox and the dialogs that are fetched in the app, the schedule presets) is
//! `assets/js/design-parts/media.js`; its hooks are the `data-design-*` attributes below.

pub(super) mod parts;

use topcoat::{
    Result,
    asset::asset_config,
    context::Cx,
    view::{View, attributes, component, view},
};
use tuita_server::dto::MeDto;

use self::parts::{clip, drop_hint, frame};
use super::{
    sample,
    section::{row, section},
};
use crate::{
    assets::{DESIGN_BROKEN_VIDEO, DESIGN_SAMPLE_VIDEO},
    compose::{
        emoji_picker::emoji_picker, media_tray::media_tray, poll_editor::poll_editor,
        schedule_picker::schedule_picker,
    },
    feed::scheduled::{reschedule_dialog, scheduled_card},
    lists::{
        editor_dialog::{DIALOG_ID as LIST_EDITOR_ID, list_editor_dialog},
        list_row::list_row,
    },
    messages::{
        chat_thread::chat_thread,
        conversation_row::conversation_row,
        frame::NEW_ID,
        people::{PickMode, people_dialog},
    },
    post::{
        dialogs::{edit_dialog, history_dialog, report_dialog},
        lightbox::lightbox,
        link_card::link_card,
        media_grid::media_grid,
        poll::poll_view,
        post_card::{Me, card_view},
        sensitive_cover::sensitive_cover,
        video_player::{VideoFit, video_player},
    },
    ui::{
        avatar::AvatarShape,
        button::{ButtonVariant, button},
        cropper::{CropKind, cropper},
        dropzone::dropzone,
    },
};

/// The signed-in viewer the gallery plays, for the components that take one.
fn viewer(me: &MeDto) -> Me<'_> {
    Me {
        id: me.user.id,
        handle: me.user.handle.as_str(),
        pinned_post_id: None,
    }
}

#[component]
async fn files(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        section(
            id: "dropzone",
            title: "Dropzone",
            note: "drag files over it: a marching dithered border; it turns danger for refused types",
            dropzone(accept: "image/*", drop_hint())
        )
        section(
            id: "cropper",
            title: "Cropper",
            note: "avatar (shape mask) and banner (3:1); drag, pinch, wheel, slider, rotate, arrow keys",
            frame(
                stack: true,
                attrs: attributes! { data-design-sample="1200,900,20,photo.png" },
                cropper(kind: CropKind::Avatar, shape: AvatarShape::Squircle)
            )
            frame(
                stack: true,
                attrs: attributes! { data-design-sample="1800,700,200,banner.png" },
                cropper(kind: CropKind::Banner)
            )
            <p class="mono-tag" data-design-rect="">
                "Apply to see the crop rectangle"
            </p>
        )
    })
}

#[component]
async fn pictures(cx: &Cx) -> Result<impl View> {
    let photos = sample::photos();
    let three = sample::with_media(3);
    let sensitive = sample::media_sensitive();
    let me = sample::sample_me();
    let thread = sample::lightbox_thread();
    Ok(view! {
        section(
            id: "media-grid",
            title: "MediaGrid",
            note: "1 to 4 photos as uploaded, never dithered; click opens the lightbox; Skeleton while loading",
            frame(
                stack: true,
                media_grid(media: &photos[..1])
                media_grid(media: &photos[..2])
                media_grid(media: &photos[..3])
                media_grid(media: &photos[..4])
            )
            frame(
                card_view(post: &three, me: Some(viewer(&me)))
                card_view(post: &sensitive, me: Some(viewer(&me)))
            )
        )
        section(
            id: "lightbox",
            title: "Lightbox",
            note: "full-screen original pixels, arrows and swipe, alt text",
            <div data-design-sandbox="">
                row(
                    button(
                        variant: ButtonVariant::Outline,
                        attrs: attributes! { data-design-open="lightbox" },
                        "Open lightbox"
                    )
                )
                <template data-design-template="lightbox">
                    lightbox(thread: &thread, index: 0)
                </template>
            </div>
        )
    })
}

#[component]
async fn video(cx: &Cx) -> Result<impl View> {
    let config = asset_config(cx);
    let sample_src = config.resolve(cx, DESIGN_SAMPLE_VIDEO);
    let broken_src = config.resolve(cx, DESIGN_BROKEN_VIDEO);
    let photos = sample::photos();
    Ok(view! {
        section(
            id: "video-player",
            title: "VideoPlayer",
            note: "native video with dithered chrome: scrubber with buffered range, volume, speed, picture-in-picture, fullscreen; space/k m f arrows when focused; the bar hides while playing",
            frame(
                stack: true,
                clip(
                    video_player(
                        src: &sample_src,
                        poster: photos[0].url.as_str(),
                        alt: "A moving gradient"
                    )
                )
                <p class="mono-tag">"loop, muted, autoplay, cover: a GIF-like clip"</p>
                clip(
                    square: true,
                    video_player(
                        src: &sample_src,
                        alt: "A looping gradient",
                        looping: true,
                        muted: true,
                        autoplay: true,
                        fit: VideoFit::Cover
                    )
                )
                <p class="mono-tag">"broken source: error with retry"</p>
                clip(video_player(src: &broken_src, alt: "A missing video"))
            )
        )
        section(
            id: "sensitive-cover",
            title: "SensitiveCover",
            note: "CSS blur, a 0.5 dither veil and a Show button",
            frame(stack: true, sensitive_cover(media_grid(media: &photos[..2])))
        )
    })
}

#[component]
async fn links_and_polls(cx: &Cx) -> Result<impl View> {
    let large = sample::link_card();
    let small = tuita_server::dto::LinkCardDto {
        large: false,
        ..sample::link_card()
    };
    let bare = tuita_server::dto::LinkCardDto {
        image: None,
        large: false,
        ..sample::link_card()
    };
    let (open, multi, voted, closed) = (
        sample::poll_open(),
        sample::poll_multi(),
        sample::poll_voted(),
        sample::poll_closed(),
    );
    Ok(view! {
        section(
            id: "link-card",
            title: "LinkCard",
            note: "large with image, small with image, no image",
            frame(
                stack: true,
                link_card(card: &large)
                link_card(card: &small)
                link_card(card: &bare)
            )
        )
        section(
            id: "poll",
            title: "Poll",
            note: "choices; multiple choice; voted (solid bar); closed with final results",
            frame(
                stack: true,
                poll_view(post: &open, mine: false)
                poll_view(post: &multi, mine: false)
                poll_view(post: &voted, mine: false)
                poll_view(post: &closed, mine: false)
            )
        )
    })
}

#[component]
async fn composing(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        section(
            id: "poll-editor",
            title: "PollEditor",
            note: "2 to 4 choices, length, multiple choice",
            frame(
                stack: true,
                attrs: attributes! { data-design-poll="" },
                <div data-design-poll-editor="">poll_editor()</div>
                <div data-design-poll-add="" hidden="">
                    button(
                        variant: ButtonVariant::Outline,
                        attrs: attributes! { data-design-act="add-poll" },
                        "Add a poll"
                    )
                </div>
            )
        )
        section(
            id: "media-tray",
            title: "MediaTray",
            note: "composer attachments: previews, progress, retry, alt text per image",
            frame(
                stack: true,
                attrs: attributes! { data-design-tray="800,600,90,a.png;800,600,260,b.png" },
                media_tray(max_video_mb: 50)
            )
        )
        section(
            id: "emoji-picker",
            title: "EmojiPicker",
            note: "64 static emoji in a popover",
            row(<div data-design-emoji="">emoji_picker(id: "design-emoji")</div>)
        )
        section(
            id: "schedule-picker",
            title: "SchedulePicker",
            note: "presets plus a date and time",
            row(
                <div data-design-schedule="">
                    schedule_picker(id: "design-schedule")
                    <span class="mono-tag" data-design-schedule-value="">
                        "not scheduled"
                    </span>
                </div>
            )
        )
    })
}

#[component]
async fn post_dialogs(cx: &Cx) -> Result<impl View> {
    let post = sample::plain();
    let edits = sample::history();
    Ok(view! {
        section(
            id: "edit-dialogs",
            title: "Edit, history, report",
            note: "EditPostDialog, EditHistoryDialog and ReportDialog",
            <div data-design-sandbox="">
                row(
                    button(
                        variant: ButtonVariant::Outline,
                        attrs: attributes! { data-design-open="edit" },
                        "Edit post"
                    )
                    button(
                        variant: ButtonVariant::Outline,
                        attrs: attributes! { data-design-open="history" },
                        "Edit history"
                    )
                    button(
                        variant: ButtonVariant::Outline,
                        attrs: attributes! { data-design-open="report" },
                        "Report post"
                    )
                )
                <template data-design-template="edit">
                    edit_dialog(post: &post)
                </template>
                <template data-design-template="history">
                    history_dialog(post: &post, edits: &edits)
                </template>
                <template data-design-template="report">
                    report_dialog(post: Some(&post))
                </template>
            </div>
        )
    })
}

#[component]
async fn chats(cx: &Cx) -> Result<impl View> {
    let conversations = sample::conversations();
    let messages = sample::chat();
    let ada = sample::user_of(&sample::ada());
    let bob = sample::user_of(&sample::bob());
    let me = sample::sample_me().user.id.to_string();
    Ok(view! {
        section(
            id: "conversation-row",
            title: "ConversationRow",
            note: "unread count, last message preview",
            frame(
                #[key(conversation.user.id.as_bytes())]
                for conversation in &conversations {
                    conversation_row(conversation: conversation)
                }
            )
        )
        section(
            id: "chat-thread",
            title: "ChatThread",
            note: "grouped bubbles, links, seen marker, delete on your own",
            frame(chat_thread(user: &ada, messages: &messages, older: "#chat-thread"))
            frame(chat_thread(user: &bob, messages: &[]))
            <div data-design-sandbox="">
                row(
                    button(
                        variant: ButtonVariant::Outline,
                        attrs: attributes! { command="show-modal" commandfor=(NEW_ID) },
                        "New message dialog"
                    )
                )
                people_dialog(
                    id: NEW_ID,
                    title: "New message",
                    mode: PickMode::New,
                    me: &me
                )
            </div>
        )
    })
}

#[component]
async fn lists_and_scheduled(cx: &Cx) -> Result<impl View> {
    let public = sample::sample_list();
    let private = sample::private_list();
    let scheduled = sample::scheduled();
    Ok(view! {
        section(
            id: "list-row",
            title: "ListRow",
            note: "public and private lists",
            frame(
                list_row(list: &public)
                list_row(list: &private)
            )
            <div data-design-sandbox="">
                row(
                    button(
                        variant: ButtonVariant::Outline,
                        attrs: attributes! { command="show-modal" commandfor=(LIST_EDITOR_ID) },
                        "List editor dialog"
                    )
                )
                list_editor_dialog()
            </div>
        )
        section(
            id: "scheduled-card",
            title: "ScheduledCard",
            note: "pending and failed scheduled posts",
            <div data-design-sandbox="">
                frame(
                    #[key(item.id.as_bytes())]
                    for item in &scheduled {
                        scheduled_card(item: item, acct: "sam")
                    }
                )
                reschedule_dialog()
            </div>
        )
    })
}

#[component]
pub(crate) async fn media(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        files()
        pictures()
        video()
        links_and_polls()
        composing()
        post_dialogs()
        chats()
        lists_and_scheduled()
    })
}
