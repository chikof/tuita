//! An on/off switch.

use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, Signal, signal},
    view::{Attributes, View, component, view},
};

/// A `role="switch"` button named by `label`, whose state is the caller's `checked` signal.
///
/// A click flips `checked` in the browser. The `touched` signal only keeps the fill animation from
/// running on first paint (`data-touched`). To react to a change, read `checked` in a `$(..)` or a
/// shard, or put `@click=$(..)` on an ancestor: the click bubbles after the flip, so the handler
/// sees the new value. `attrs` go on the `<button>` (`id`, `disabled`, `aria-describedby`, ...).
#[component]
pub(crate) async fn switch(
    cx: &Cx,
    label: &str,
    checked: &Signal<bool>,
    #[default] attrs: Attributes,
) -> Result<impl View> {
    let touched = signal(cx, || false);
    Ok(view! {
        <button
            (attrs)
            type="button"
            role="switch"
            class="switch"
            :aria-checked=$(if checked.get() { "true" } else { "false" })
            :data-touched=$(touched.get())
            @click=$(|_e: Event| {
                touched.set(true);
                checked.set(!checked.get());
            })
            data-c="switch"
        >
            <span class="track" aria-hidden="true">
                <i class="fill"></i>
                <i class="thumb"></i>
            </span>
            <span class="text">(label)</span>
        </button>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[component]
    async fn demo(cx: &Cx, on: bool) -> Result<impl View> {
        let checked = signal(cx, move || on);
        Ok(view! { switch(label: "Dark mode", checked: &checked) })
    }

    #[test]
    fn it_is_a_switch_named_by_its_label_in_the_signals_state() {
        let off = render("en", |cx| view! { demo(on: false) }.boxed());
        let on = render("en", |cx| view! { demo(on: true) }.boxed());
        assert!(
            off.contains(r#"role="switch""#) && off.contains(r#"type="button""#),
            "{off}"
        );
        assert!(off.contains(r#"aria-checked="false""#), "{off}");
        assert!(on.contains(r#"aria-checked="true""#), "{on}");
        assert!(
            off.contains(r#"<span class="text">Dark mode</span>"#),
            "{off}"
        );
        assert!(
            off.contains(r#"<span class="track" aria-hidden="true">"#),
            "{off}"
        );
    }

    #[test]
    fn the_fill_animation_waits_for_the_first_click() {
        let html = render("en", |cx| view! { demo(on: true) }.boxed());
        assert!(!html.contains(r#" data-touched="#), "{html}");
        assert!(html.contains("data-topcoat-bind:data-touched"), "{html}");
        assert!(html.contains("data-topcoat-on:click"), "{html}");
    }
}
