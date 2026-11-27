//! The avatar shape radios.

use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, Signal},
    view::{View, component, element_id, view},
};

use crate::{
    l10n::t,
    ui::avatar::{AvatarShape, avatar},
};

/// One radio per shape, each showing the shape on the account's picture (or its identicon). The
/// chosen shape's name is the `value` signal (`square`, `circle`, `squircle`, `octagon`).
#[component]
pub(crate) async fn shape_picker(
    cx: &Cx,
    value: &Signal<String>,
    seed: &str,
    #[default]
    #[into]
    src: Option<&str>,
    label: &str,
) -> Result<impl View> {
    let group = element_id(cx);
    let shapes = [
        (
            "square",
            AvatarShape::Square,
            t!(settings_shape_square).resolve(cx).into_owned(),
        ),
        (
            "circle",
            AvatarShape::Circle,
            t!(settings_shape_circle).resolve(cx).into_owned(),
        ),
        (
            "squircle",
            AvatarShape::Squircle,
            t!(settings_shape_squircle).resolve(cx).into_owned(),
        ),
        (
            "octagon",
            AvatarShape::Octagon,
            t!(settings_shape_octagon).resolve(cx).into_owned(),
        ),
    ];
    Ok(view! {
        <fieldset data-c="shape-picker">
            <legend class="mono-tag">(label)</legend>
            <div class="options">
                for (id, shape, name) in shapes {
                    <label class="option">
                        <input
                            type="radio"
                            name=(&group)
                            value=(id)
                            :checked=$(value.get() == id)
                            @change=$(|_e: Event| value.set(id.to_owned()))
                        >
                        <span aria-hidden="true">
                            avatar(
                                seed: seed,
                                src: src,
                                alt: "",
                                shape: shape,
                                size: 48
                            )
                        </span>
                        <span class="name">(name)</span>
                    </label>
                }
            </div>
        </fieldset>
    })
}
