//! Gallery sections: button, icon-button, menu, popover, tooltip.

use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, signal},
    view::{View, attributes, component, view},
};

use super::section::{row, section};
use crate::{
    icons::Icon,
    ui::{
        button::{ButtonSize, ButtonVariant, button},
        icon::icon,
        icon_button::{IconButtonVariant, icon_button},
        menu::{MenuItem, menu},
        popover::popover,
        tooltip::tooltip,
    },
};

const VARIANTS: [(&str, ButtonVariant); 4] = [
    ("solid", ButtonVariant::Solid),
    ("outline", ButtonVariant::Outline),
    ("ghost", ButtonVariant::Ghost),
    ("danger", ButtonVariant::Danger),
];

const SIZES: [(&str, ButtonSize); 3] = [
    ("sm", ButtonSize::Sm),
    ("md", ButtonSize::Md),
    ("lg", ButtonSize::Lg),
];

/// The repost button of the demo: `aria-pressed` flips with a click.
#[component]
async fn pressed_demo(cx: &Cx) -> Result<impl View> {
    let pressed = signal(cx, || false);
    Ok(view! {
        icon_button(
            glyph: Icon::Repeat,
            label: "Repost",
            attrs: attributes! {
                :aria-pressed=$(if pressed.get() { "true" } else { "false" })
                @click=$(|_e: Event| pressed.set(!pressed.get()))
            }
        )
    })
}

/// The menu and the "Picked: ..." line that follows its choice.
#[component]
async fn menu_demo(cx: &Cx) -> Result<impl View> {
    let picked = signal(cx, || "none".to_owned());
    Ok(view! {
        row(
            menu(
                id: "core-menu",
                label: "Post actions",
                items: vec![
                    MenuItem {
                        id: "copy",
                        label: "Copy link",
                        meta: Some("C"),
                        attrs: attributes! { @click=$(|_e: Event| picked.set("copy".to_owned())) },
                        ..MenuItem::default()
                    },
                    MenuItem {
                        id: "mute",
                        label: "Mute thread",
                        meta: Some("M"),
                        attrs: attributes! { @click=$(|_e: Event| picked.set("mute".to_owned())) },
                        ..MenuItem::default()
                    },
                    MenuItem {
                        id: "disabled",
                        label: "Unavailable",
                        disabled: true,
                        ..MenuItem::default()
                    },
                    MenuItem {
                        id: "report",
                        label: "Report",
                        danger: true,
                        attrs: attributes! { @click=$(|_e: Event| picked.set("report".to_owned())) },
                        ..MenuItem::default()
                    },
                ],
                trigger: view! {
                    "Open menu "
                    icon(name: Icon::ChevronDown, size: 16)
                }.into(

                )
            )
            <span class="mono-tag">
                "Picked: "
                $(picked.get())
            </span>
        )
    })
}

#[component]
pub(crate) async fn actions(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        section(
            id: "button",
            title: "Button",
            note: "solid, outline, ghost, danger / sm, md, lg / hover wash / press / loading",
            for (name, variant) in VARIANTS {
                row(
                    label: name,
                    for (size_name, size) in SIZES {
                        button(
                            variant: variant,
                            size: size,
                            (format!("{size_name} button"))
                        )
                    }
                    button(
                        variant: variant,
                        icon(name: Icon::Plus, size: 16)
                        " With icon"
                    )
                    button(variant: variant, disabled: true, "Disabled")
                    button(variant: variant, loading: true, "Loading label")
                )
            }
        )
        section(
            id: "icon-button",
            title: "IconButton",
            note: "label is required; pressed sets aria-pressed",
            row(
                label: "ghost / outline / pressed / disabled",
                icon_button(glyph: Icon::Heart, label: "Like")
                icon_button(
                    glyph: Icon::Bookmark,
                    label: "Bookmark",
                    variant: IconButtonVariant::Outline
                )
                pressed_demo()
                icon_button(glyph: Icon::Share, label: "Share (pressed)", pressed: true)
                icon_button(
                    glyph: Icon::Trash,
                    label: "Delete (disabled)",
                    attrs: attributes! { disabled=(true) }
                )
            )
            row(
                label: "sizes",
                icon_button(
                    glyph: Icon::MoreHorizontal,
                    label: "More, small",
                    size: ButtonSize::Sm,
                    variant: IconButtonVariant::Outline
                )
                icon_button(
                    glyph: Icon::MoreHorizontal,
                    label: "More, medium",
                    size: ButtonSize::Md,
                    variant: IconButtonVariant::Outline
                )
                icon_button(
                    glyph: Icon::MoreHorizontal,
                    label: "More, large",
                    size: ButtonSize::Lg,
                    variant: IconButtonVariant::Outline
                )
            )
        )
        section(
            id: "menu",
            title: "Menu",
            note: "native popover, arrow keys, Esc closes",
            menu_demo()
        )
        section(
            id: "popover",
            title: "Popover",
            note: "native popover with CSS anchor positioning, JS fallback",
            row(
                popover(
                    id: "core-popover",
                    label: "Details",
                    trigger: view! { "Show details" }.into(),
                    <p>
                        "Popovers hold any content. Click outside or press Esc to close."
                    </p>
                )
            )
        )
        section(
            id: "tooltip",
            title: "Tooltip",
            note: "mono label after 500 ms on hover or focus",
            row(
                tooltip(
                    id: "core-tip-reply",
                    text: "Reply",
                    icon_button(
                        glyph: Icon::Message,
                        label: "Reply",
                        variant: IconButtonVariant::Outline,
                        attrs: attributes! { aria-describedby="core-tip-reply" }
                    )
                )
                tooltip(
                    id: "core-tip-bookmark",
                    text: "Bookmark",
                    icon_button(
                        glyph: Icon::Bookmark,
                        label: "Bookmark",
                        variant: IconButtonVariant::Outline,
                        attrs: attributes! { aria-describedby="core-tip-bookmark" }
                    )
                )
            )
        )
    })
}
