//! Gallery sections: avatar, avatar-stack, brand, icon, empty-state, not-found.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use super::{
    sample,
    section::{row, section},
};
use crate::{
    icons::Icon,
    ui::{
        avatar::{AVATAR_SIZES, AvatarShape, avatar},
        avatar_stack::{StackPerson, avatar_stack},
        brand::brand,
        button::{ButtonVariant, button},
        empty_state::empty_state,
        icon::icon,
        not_found::not_found,
    },
};

const SHAPES: [(&str, AvatarShape); 4] = [
    ("square", AvatarShape::Square),
    ("circle", AvatarShape::Circle),
    ("squircle", AvatarShape::Squircle),
    ("octagon", AvatarShape::Octagon),
];

const NAMES: [&str; 6] = ["ada", "bob", "carol", "dave", "eve", "frank"];

/// The first `count` of the six demo people.
fn people(count: usize) -> Vec<StackPerson<'static>> {
    NAMES
        .into_iter()
        .take(count)
        .map(|name| StackPerson {
            seed: name,
            alt: name,
            src: None,
        })
        .collect()
}

#[component]
pub(crate) async fn identity(cx: &Cx) -> Result<impl View> {
    let photo = sample::photo_url();
    Ok(view! {
        section(
            id: "avatar",
            title: "Avatar",
            note: "four pixel-grid shapes; identicon fallback; sizes 24 to 128",
            for (name, shape) in SHAPES {
                let label = format!("{name} / identicon");
                row(
                    label: label.as_str(),
                    for size in AVATAR_SIZES {
                        let seed = format!("{name}-{size}");
                        let alt = format!("{name} {size}");
                        avatar(seed: &seed, alt: &alt, shape: shape, size: size)
                    }
                )
            }
            row(
                label: "photo (never dithered)",
                for (name, shape) in SHAPES {
                    let alt = format!("photo {name}");
                    avatar(
                        seed: name,
                        src: photo.as_str(),
                        alt: &alt,
                        shape: shape,
                        size: 64
                    )
                }
            )
        )
        section(
            id: "avatar-stack",
            title: "AvatarStack",
            note: "overlapping avatars with a +N chip",
            row(
                avatar_stack(people: people(3))
                avatar_stack(people: people(6), max: 4)
                avatar_stack(
                    people: people(6),
                    max: 2,
                    size: 48,
                    shape: AvatarShape::Squircle
                )
            )
        )
        section(
            id: "brand",
            title: "Brand",
            note: "2×2 mark with flicking cells and the pixel wordmark",
            row(
                brand()
                brand(size: 56)
                brand(wordmark: false)
            )
        )
        section(
            id: "icon",
            title: "Icon",
            note: "pixelarticons, 24×24 grid, currentColor",
            <div class="icons" data-c="design-icons">
                for glyph in Icon::ALL {
                    <span class="cell">
                        icon(name: glyph)
                        <span class="mono-tag">(glyph.name())</span>
                    </span>
                }
            </div>
        )
        section(
            id: "empty-state",
            title: "EmptyState",
            note: "seeded dither vignette, title, body, action",
            empty_state(
                context: "no-posts",
                title: "No posts yet",
                body: "Posts from people you follow will show up here.",
                action: Some(
                    view! { button(variant: ButtonVariant::Outline, "Find people") }.into(

                    ),
                )
            )
        )
        section(
            id: "not-found",
            title: "NotFound",
            note: "dithered 404 digits and a back button; grain on loud pages",
            not_found()
        )
    })
}
