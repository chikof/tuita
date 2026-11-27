//! The picture cropper.

use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, component, view},
};

use crate::{
    icons::Icon,
    l10n::t,
    ui::{
        avatar::AvatarShape,
        button::{ButtonVariant, button},
        icon_button::icon_button,
        slider::slider,
    },
};

/// What the cropped picture is for; it sets the frame's shape.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum CropKind {
    /// A square frame, clipped to the account's avatar shape.
    #[default]
    Avatar,
    /// A 3 to 1 frame.
    Banner,
}

impl CropKind {
    const fn name(self) -> &'static str {
        match self {
            Self::Avatar => "avatar",
            Self::Banner => "banner",
        }
    }
}

/// The name an [`AvatarShape`] has in the `data-shape` attribute.
const fn shape_name(shape: AvatarShape) -> &'static str {
    match shape {
        AvatarShape::Square => "square",
        AvatarShape::Circle => "circle",
        AvatarShape::Squircle => "squircle",
        AvatarShape::Octagon => "octagon",
    }
}

/// A `<form>` for choosing the area of a picture to keep: drag to move, pinch, scroll or the
/// slider to zoom, buttons to zoom and rotate, Apply to finish. The tools start disabled until a
/// picture is loaded. With `cancel` there is a Cancel button.
///
/// The `cropper.js` island runs it (`data-island="cropper"`); its event contract is documented
/// at the top of that file: send the picture with a `tuita-crop-load` event (`detail.file`),
/// listen for `tuita-crop` (`ev.data` is the `CropRect` JSON the server's `crop` field takes; the
/// hidden `crop` input holds it too) and `tuita-crop-cancel`, and set `data-busy` while the result
/// is being sent.
#[component]
pub(crate) async fn cropper(
    cx: &Cx,
    #[default] kind: CropKind,
    #[default] shape: AvatarShape,
    #[default] cancel: bool,
) -> Result<impl View> {
    let zoom_out = t!(cropper_zoom_out).resolve(cx).into_owned();
    let zoom_in = t!(cropper_zoom_in).resolve(cx).into_owned();
    let rotate = t!(cropper_rotate).resolve(cx).into_owned();
    let reset = t!(cropper_reset).resolve(cx).into_owned();
    let zoom = t!(cropper_zoom).resolve(cx).into_owned();
    let stage = t!(cropper_stage).resolve(cx).into_owned();
    Ok(view! {
        <form
            class="cropper"
            data-island="cropper"
            data-kind=(kind.name())
            data-shape=(shape_name(shape))
            novalidate=(true)
            data-c="cropper"
        >
            <div
                class=(format!("stage {}", kind.name()))
                role="application"
                tabindex="0"
                aria-label=(stage)
            ></div>
            <p class="hint">(t!(cropper_hint))</p>
            <div class="tools">
                icon_button(
                    glyph: Icon::ZoomOut,
                    label: &zoom_out,
                    attrs: attributes! { data-act="zoom-out" disabled=(true) }
                )
                <div class="zoom">
                    slider(
                        label: &zoom,
                        value: 1.0,
                        min: 1.0,
                        max: 6.0,
                        step: 0.01,
                        attrs: attributes! { data-role="zoom" disabled=(true) }
                    )
                </div>
                icon_button(
                    glyph: Icon::ZoomIn,
                    label: &zoom_in,
                    attrs: attributes! { data-act="zoom-in" disabled=(true) }
                )
                icon_button(
                    glyph: Icon::Redo,
                    label: &rotate,
                    attrs: attributes! { data-act="rotate" disabled=(true) }
                )
                icon_button(
                    glyph: Icon::Undo,
                    label: &reset,
                    attrs: attributes! { data-act="reset" disabled=(true) }
                )
            </div>
            <div class="actions">
                if cancel {
                    button(
                        variant: ButtonVariant::Ghost,
                        attrs: attributes! { data-act="cancel" },
                        (t!(cancel))
                    )
                }
                button(submit: true, disabled: true, (t!(cropper_apply)))
            </div>
            <input type="hidden" name="crop">
        </form>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn it_is_a_crop_stage_with_named_tools_that_start_off() {
        let html = render("en", |cx| view! { cropper() }.boxed());
        assert!(html.contains(r#"data-island="cropper""#), "{html}");
        assert!(html.contains(r#"data-kind="avatar""#), "{html}");
        assert!(html.contains(r#"data-shape="circle""#), "{html}");
        assert!(html.contains(r#"class="stage avatar""#), "{html}");
        assert!(html.contains(r#"role="application""#), "{html}");
        for name in ["Zoom out", "Zoom in", "Rotate", "Reset"] {
            assert!(
                html.contains(&format!(r#"aria-label="{name}""#)),
                "{name}: {html}"
            );
        }
        for act in ["zoom-out", "zoom-in", "rotate", "reset"] {
            assert!(
                html.contains(&format!(r#"data-act="{act}""#)),
                "{act}: {html}"
            );
        }
        assert!(html.contains(r#"type="range""#), "{html}");
        assert!(
            html.contains(r#"<span class="label">Apply</span>"#),
            "{html}"
        );
        assert!(html.contains(r#"name="crop""#), "{html}");
        assert!(!html.contains(r#"data-act="cancel""#), "{html}");
    }

    #[test]
    fn a_banner_in_a_cancellable_dialog_has_its_own_stage_and_a_cancel_button() {
        let html = render("en", |cx| {
            view! {
                cropper(
                    kind: CropKind::Banner,
                    shape: AvatarShape::Octagon,
                    cancel: true
                )
            }
            .boxed()
        });
        assert!(html.contains(r#"class="stage banner""#), "{html}");
        assert!(html.contains(r#"data-shape="octagon""#), "{html}");
        assert!(html.contains(r#"data-act="cancel""#), "{html}");
        assert!(
            html.contains(r#"<span class="label">Cancel</span>"#),
            "{html}"
        );
    }
}
