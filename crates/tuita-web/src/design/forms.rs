//! Gallery sections: text-field, text-area, select, switch, checkbox, radio, slider, segmented,
//! tabs, char-counter.

use topcoat::{
    Result,
    context::Cx,
    runtime::signal,
    view::{View, attributes, component, view},
};

use super::section::{row, section};
use crate::ui::{
    char_counter::char_counter,
    checkbox::checkbox,
    radio::radio,
    segmented::segmented,
    select::{SelectOption, select},
    slider::slider,
    switch::switch,
    tabs::{Choice, tabs},
    text_area::text_area,
    text_field::text_field,
};

/// The three switches of the demo, each with its own state.
#[component]
async fn switches(cx: &Cx) -> Result<impl View> {
    let on = signal(cx, || true);
    let off = signal(cx, || false);
    let locked = signal(cx, || true);
    Ok(view! {
        row(
            switch(label: "Notifications on", checked: &on)
            switch(label: "Notifications off", checked: &off)
            switch(
                label: "Disabled",
                checked: &locked,
                attrs: attributes! { disabled=(true) }
            )
        )
    })
}

#[component]
async fn segmented_demo(cx: &Cx) -> Result<impl View> {
    let value = signal(cx, || "grid".to_owned());
    Ok(view! {
        row(
            segmented(
                label: "View",
                items: vec![
                    Choice::new("grid", "Grid"),
                    Choice::new("list", "List"),
                    Choice::new("map", "Map"),
                ],
                value: &value
            )
        )
    })
}

#[component]
async fn tabs_demo(cx: &Cx) -> Result<impl View> {
    let value = signal(cx, || "posts".to_owned());
    Ok(view! {
        tabs(
            label: "Profile sections",
            items: vec![
                Choice::new("posts", "Posts"),
                Choice::new("replies", "Replies"),
                Choice::new("media", "Media"),
            ],
            value: &value,
            panel: true,
            <p>
                "Showing the "
                $(value.get())
                " panel."
            </p>
        )
    })
}

#[component]
pub(crate) async fn forms(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        section(
            id: "text-field",
            title: "TextField",
            note: "focus draws a 2px ink bottom line; error uses danger",
            <div class="grid" data-c="design-grid">
                text_field(
                    label: "Display name",
                    value: "Ada Lovelace",
                    attrs: attributes! { placeholder="Your name" }
                )
                text_field(
                    label: "Empty",
                    hint: "Helper text under the field.",
                    attrs: attributes! { placeholder="Placeholder text" }
                )
                text_field(
                    label: "Email",
                    value: "ada@",
                    error: "Enter a full email address.",
                    attrs: attributes! { type="email" }
                )
                text_field(
                    label: "Disabled",
                    value: "Read only",
                    attrs: attributes! { disabled=(true) }
                )
            </div>
        )
        section(
            id: "text-area",
            title: "TextArea",
            note: "autogrows with its content",
            <div class="grid" data-c="design-grid">
                text_area(
                    label: "Bio",
                    value: "A short bio that grows as you type.\nTry adding more lines."
                )
                text_area(
                    label: "With error",
                    value: "Too long",
                    error: "Keep it under 500 characters."
                )
            </div>
        )
        section(
            id: "select",
            title: "Select",
            note: "native select with the field look; mono label above, pixel chevron",
            <div class="grid" data-c="design-grid">
                select(
                    label: "Duration",
                    options: vec![
                        SelectOption::new("1", "1 day"),
                        SelectOption::new("7", "1 week"),
                        SelectOption::new("0", "Until ended"),
                    ],
                    value: "7",
                    hint: "Picks how long it runs"
                )
                select(
                    label: "Disabled",
                    options: vec![SelectOption::new("a", "Locked")],
                    attrs: attributes! { disabled=(true) }
                )
            </div>
        )
        section(
            id: "switch",
            title: "Switch",
            note: "track fills through four Bayer levels; role switch",
            switches()
        )
        section(
            id: "checkbox",
            title: "Checkbox",
            note: "pixel square with the check glyph",
            row(
                checkbox(label: "I agree", checked: true)
                checkbox(label: "Send updates")
                checkbox(
                    label: "Disabled checked",
                    checked: true,
                    attrs: attributes! { disabled=(true) }
                )
                checkbox(label: "Disabled", attrs: attributes! { disabled=(true) })
            )
        )
        section(
            id: "radio",
            title: "Radio",
            note: "octagon",
            row(
                radio(
                    group: "flavour",
                    value: "vanilla",
                    label: "Vanilla",
                    checked: true
                )
                radio(group: "flavour", value: "mint", label: "Mint")
                radio(group: "flavour", value: "cocoa", label: "Cocoa")
                radio(
                    group: "flavour",
                    value: "none",
                    label: "Disabled",
                    attrs: attributes! { disabled=(true) }
                )
            )
        )
        section(
            id: "slider",
            title: "Slider",
            note: "Bayer ramp, denser toward the thumb, keyboard steps",
            <div class="narrow" data-c="design-narrow">
                slider(label: "Volume", value: 40.0)
                slider(label: "Steps of 25", value: 50.0, step: 25.0)
            </div>
        )
        section(
            id: "segmented",
            title: "Segmented",
            note: "dissolving pill; arrow keys",
            segmented_demo()
        )
        section(
            id: "tabs",
            title: "Tabs",
            note: "dissolving dithered underline; tablist, tab, tabpanel",
            tabs_demo()
        )
        section(
            id: "char-counter",
            title: "CharCounter",
            note: "12-segment ring; number in the last 20; danger over the limit",
            row(
                char_counter(value: 0, max: 500)
                char_counter(value: 125, max: 500)
                char_counter(value: 250, max: 500)
                char_counter(value: 480, max: 500)
                char_counter(value: 500, max: 500)
                char_counter(value: 512, max: 500)
            )
            <div class="narrow" data-c="design-narrow" data-design-counter="20">
                text_field(
                    label: "Type to count (limit 20)",
                    value: "Count me",
                    attrs: attributes! { data-design-counter-input="" }
                )
                char_counter(value: 8, max: 20)
            </div>
        )
    })
}
