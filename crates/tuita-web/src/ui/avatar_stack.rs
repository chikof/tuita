//! Overlapping avatars with a "+N" tail.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use crate::{
    l10n::t,
    ui::avatar::{AvatarShape, avatar},
};

/// One person in an [`avatar_stack`].
pub(crate) struct StackPerson<'a> {
    pub(crate) seed: &'a str,
    pub(crate) alt: &'a str,
    pub(crate) src: Option<&'a str>,
}

/// The first `max` people as overlapping avatars of `size` px, then "+N" (named "N more") for the
/// rest.
#[component]
pub(crate) async fn avatar_stack(
    cx: &Cx,
    people: Vec<StackPerson<'_>>,
    #[default(4)] max: usize,
    #[default(32)] size: u32,
    #[default] shape: AvatarShape,
) -> Result<impl View> {
    let extra = people.len().saturating_sub(max);
    let more = t!(avatar_more, count: extra).resolve(cx).into_owned();
    Ok(view! {
        <span class="stack" style=(format!("--size:{size}px")) data-c="avatar-stack">
            for person in people.into_iter().take(max) {
                <span class="item">
                    avatar(
                        seed: person.seed,
                        src: person.src,
                        alt: person.alt,
                        size: size,
                        shape: shape
                    )
                </span>
            }
            if extra > 0 {
                <span class="more" role="img" aria-label=(more)>
                    "+"
                    (extra)
                </span>
            }
        </span>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::{l10n::strip_isolates, testing::render};

    fn people(n: usize) -> Vec<StackPerson<'static>> {
        ["ada", "bob", "cy", "di", "ed", "flo"]
            .into_iter()
            .take(n)
            .map(|seed| StackPerson {
                seed,
                alt: seed,
                src: None,
            })
            .collect()
    }

    #[test]
    fn it_shows_the_first_people_and_counts_the_rest() {
        let html = render("en", |cx| {
            view! { avatar_stack(people: people(6), max: 4) }.boxed()
        });
        assert_eq!(html.matches(r#"class="item""#).count(), 4, "{html}");
        assert!(html.contains(r#"role="img" aria-label="#), "{html}");
        assert!(strip_isolates(&html).contains("2 more"), "{html}");
        assert!(html.contains(">+2</span>"), "{html}");
        assert!(html.contains("--size:32px"), "{html}");
    }

    #[test]
    fn nothing_is_counted_when_everyone_fits() {
        let html = render("en", |cx| view! { avatar_stack(people: people(3)) }.boxed());
        assert_eq!(html.matches(r#"class="item""#).count(), 3, "{html}");
        assert!(!html.contains(r#"class="more""#), "{html}");
    }
}
