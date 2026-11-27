//! A count pill or an attention dot.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

/// The text a count shows: the number, or `max+` past `max`.
fn shown(count: Option<u64>, max: u64) -> String {
    match count {
        None => String::new(),
        Some(count) if count > max => format!("{max}+"),
        Some(count) => count.to_string(),
    }
}

/// A pill with `count` (`99+` past `max`), or a blinking square when `dot`. `label` is the
/// accessible name: the pill hides its digits from assistive technology and reads `label`
/// instead; the dot becomes an image with that name.
#[component]
pub(crate) async fn badge(
    cx: &Cx,
    #[default]
    #[into]
    count: Option<u64>,
    #[default(99)] max: u64,
    #[default] dot: bool,
    #[default]
    #[into]
    label: Option<&str>,
) -> Result<impl View> {
    let text = shown(count, max);
    Ok(view! {
        if dot {
            <span
                class="dot"
                role=(label.map(|_| "img"))
                aria-label=(label)
                data-c="badge"
            ></span>
        } else {
            <span class="badge" data-c="badge">
                <span aria-hidden=(label.map(|_| "true"))>(text)</span>
                if let Some(label) = label {
                    <span class="sr-only">(label)</span>
                }
            </span>
        }
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn counts_past_max_collapse() {
        assert_eq!(shown(Some(5), 99), "5");
        assert_eq!(shown(Some(99), 99), "99");
        assert_eq!(shown(Some(100), 99), "99+");
        assert_eq!(shown(None, 99), "");
    }

    #[test]
    fn a_labelled_pill_hides_the_digits_and_reads_the_label() {
        let html = render("en", |cx| {
            view! { badge(count: 3, label: "unread") }.boxed()
        });
        assert!(html.contains(r#"class="badge""#), "{html}");
        assert!(
            html.contains(r#"<span aria-hidden="true">3</span>"#),
            "{html}"
        );
        assert!(
            html.contains(r#"<span class="sr-only">unread</span>"#),
            "{html}"
        );
    }

    #[test]
    fn an_unlabelled_pill_shows_just_the_digits() {
        let html = render("en", |cx| view! { badge(count: 120) }.boxed());
        assert!(html.contains(">99+</span>"), "{html}");
        assert!(
            !html.contains("sr-only") && !html.contains("aria-hidden"),
            "{html}"
        );
    }

    #[test]
    fn a_dot_is_an_image_only_with_a_label() {
        let named = render("en", |cx| view! { badge(dot: true, label: "new") }.boxed());
        assert!(named.contains(r#"role="img""#) && named.contains(r#"aria-label="new""#));
        let bare = render("en", |cx| view! { badge(dot: true) }.boxed());
        assert!(
            bare.contains(r#"class="dot""#) && !bare.contains("role="),
            "{bare}"
        );
    }
}
