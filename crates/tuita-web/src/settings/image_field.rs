//! Choosing, cropping and removing the avatar or banner.

use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, procedure},
    view::{View, attributes, component, view},
};
use tuita_server::dto::{MeDto, UserDto};

use crate::{
    api,
    app::FLASH_COOKIE,
    cookies,
    l10n::t,
    settings::actions::{Done, failed},
    shell::parts::account_avatar,
    ui::{
        avatar::AvatarShape,
        button::{ButtonSize, ButtonVariant, button},
        cropper::{CropKind, cropper},
        dialog::dialog,
        dither_field::{FieldKind, dither_field},
    },
};

/// The kinds of picture an account has.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ImageKind {
    Avatar,
    Banner,
}

impl ImageKind {
    const fn name(self) -> &'static str {
        match self {
            Self::Avatar => "avatar",
            Self::Banner => "banner",
        }
    }
}

/// Removes the account's `avatar` or `banner`. The page reloads afterwards, so the answer is a
/// flash message in a cookie that the next page shows (`tuita_flash`).
#[procedure]
pub(crate) async fn clear_image(cx: &Cx, kind: String) -> Done<bool> {
    let kind = match kind.as_str() {
        "avatar" | "banner" => kind,
        _ => return Ok(Err("Unknown picture".to_owned())),
    };
    let done = failed(api::delete::<MeDto>(cx, &format!("/me/{kind}")).await);
    if done.is_ok() {
        cookies::set(cx, FLASH_COOKIE, "image_field_removed", None);
    }
    Ok(done.map(|_| true))
}

/// The current picture, a Change button that opens the file chooser, a Remove button and the
/// cropper dialog.
///
/// The `crop-upload` island (`upload.js`) does the rest: a chosen file goes to the cropper in the
/// dialog, Apply uploads it with the crop, and the page reloads with a "Picture saved." toast.
/// The `shape` is the one saved on the account; the crop frame shows it.
#[component]
pub(crate) async fn image_field(cx: &Cx, kind: ImageKind, user: &UserDto) -> Result<impl View> {
    let avatar = kind == ImageKind::Avatar;
    let current = if avatar {
        &user.avatar_url
    } else {
        &user.banner_url
    };
    let has_picture = current.is_some();
    let title = if avatar {
        t!(image_field_avatar)
    } else {
        t!(image_field_banner)
    }
    .resolve(cx)
    .into_owned();
    let dialog_title = if avatar {
        t!(image_field_edit_avatar)
    } else {
        t!(image_field_edit_banner)
    }
    .resolve(cx)
    .into_owned();
    let crop_kind = if avatar {
        CropKind::Avatar
    } else {
        CropKind::Banner
    };
    let shape = AvatarShape::from_name(user.avatar_shape.as_str()).unwrap_or_default();
    let name = kind.name();
    let dialog_id = format!("crop-{name}");
    let accept = "image/png,image/jpeg,image/gif,image/webp";
    Ok(view! {
        <div class="field" data-c="image-field">
            <div class="current" data-ink=(user.ink.as_str())>
                if avatar {
                    account_avatar(
                        handle: &user.handle,
                        src: user.avatar_url.as_deref(),
                        shape: user.avatar_shape,
                        ink: user.ink,
                        size: 96
                    )
                } else {
                    <div class="banner">
                        if let Some(url) = &user.banner_url {
                            <img src=(url) alt="">
                        } else {
                            dither_field(
                                field: FieldKind::Seeded,
                                seed: user.handle.as_str(),
                                cell: 6,
                                opacity: 0.55,
                                animate: false
                            )
                        }
                    </div>
                }
            </div>
            <div class="controls" data-island="crop-upload" data-purpose=(name)>
                <div class="buttons">
                    button(
                        variant: ButtonVariant::Outline,
                        size: ButtonSize::Sm,
                        attrs: attributes! { data-act="pick" },
                        (t!(image_field_change))
                        <span class="sr-only">(&title)</span>
                    )
                    button(
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Sm,
                        disabled: !has_picture,
                        attrs: attributes! {
                            @click=$(async |_e: Event| {
                                let done = clear_image(name.to_owned()).await;
                                if done.is_ok() {
                                    raw!("location.reload()")
                                } else {
                                    let msg = done.unwrap_err();
                                    raw!(
                                        "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:${msg}}))",
                                        msg.is_empty(),
                                    )
                                }
                            })
                        },
                        (t!(image_field_remove))
                        <span class="sr-only">(&title)</span>
                    )
                </div>
                <p class="hint">(t!(image_field_hint))</p>
                <input
                    type="file"
                    data-role="pick"
                    accept=(accept)
                    aria-label=(&title)
                    hidden=(true)
                >
                dialog(
                    id: &dialog_id,
                    title: &dialog_title,
                    cropper(kind: crop_kind, shape: shape, cancel: true)
                )
            </div>
        </div>
    })
}
