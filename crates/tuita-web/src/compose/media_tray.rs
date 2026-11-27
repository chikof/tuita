//! The media tray of the composer.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use crate::{
    icons::Icon,
    l10n::t,
    ui::{
        button::{ButtonSize, ButtonVariant, button},
        icon_button::{IconButtonVariant, icon_button},
        progress_bar::progress_bar,
    },
};

/// The empty tray and the templates its island clones for each file.
///
/// `upload.js` runs it (`data-island="media-tray"`, contract at the top of that file): it keeps
/// the tray `hidden` while no file is attached and adds an `.item` per file from `template
/// [data-tpl="item"]`, whose status area it fills from `[data-tpl="progress"]` (an indeterminate
/// bar it turns into a determinate one) and `[data-tpl="retry"]`. `max_video_mb` is the
/// instance's video limit.
#[component]
pub(crate) async fn media_tray(cx: &Cx, max_video_mb: u16) -> Result<impl View> {
    let remove = t!(composer_remove_image).resolve(cx).into_owned();
    let uploading = t!(composer_uploading).resolve(cx).into_owned();
    Ok(view! {
        <div
            class="tray"
            data-c="media-tray"
            data-island="media-tray"
            data-role="tray"
            data-max-video-mb=(max_video_mb)
            hidden=""
        >
            <template data-tpl="item">
                <div class="item">
                    <div class="frame">
                        <span class="remove">
                            icon_button(
                                glyph: Icon::Close,
                                size: ButtonSize::Sm,
                                variant: IconButtonVariant::Outline,
                                label: &remove
                            )
                        </span>
                        <div class="status" hidden=""></div>
                    </div>
                    <input class="alt" maxlength="1000" autocomplete="off">
                </div>
            </template>
            <template data-tpl="progress">progress_bar(label: &uploading)</template>
            <template data-tpl="retry">
                button(
                    size: ButtonSize::Sm,
                    variant: ButtonVariant::Outline,
                    (t!(retry))
                )
            </template>
        </div>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn the_tray_starts_hidden_with_the_templates_its_island_clones() {
        let html = render("en", |cx| view! { media_tray(max_video_mb: 50) }.boxed());
        assert!(html.contains(r#"data-island="media-tray""#), "{html}");
        assert!(html.contains(r#"data-max-video-mb="50""#), "{html}");
        assert!(html.contains("hidden"), "{html}");
        for name in ["item", "progress", "retry"] {
            assert!(html.contains(&format!(r#"data-tpl="{name}""#)), "{html}");
        }
        assert!(html.contains(r#"class="alt""#), "{html}");
        assert!(html.contains(r#"role="progressbar""#), "{html}");
        assert!(html.contains("Try again"), "{html}");
    }
}
