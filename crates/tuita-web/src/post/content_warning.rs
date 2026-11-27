//! A post behind a content warning.

use topcoat::{
    Result,
    context::Cx,
    view::{Child, View, component, element_id, view},
};
use tuita_server::dto::EmojiDto;

use crate::{
    icons::Icon,
    l10n::t,
    post::rich_text::emote_text,
    ui::{dither_fill::dither_fill, icon::icon},
};

/// The warning and a button that reveals `child`.
///
/// The body starts `hidden` and the veil shows. `post.js` handles the button
/// (`data-act="cw"`): it flips `aria-expanded`, the body's `hidden`, the veil and the button text
/// (`data-show` / `data-hide` hold both labels).
#[component]
pub(crate) async fn content_warning(
    cx: &Cx,
    cw: &str,
    #[default] emojis: &[EmojiDto],
    #[default] child: Child<'_>,
) -> Result<impl View> {
    let uid = element_id(cx);
    let body_id = format!("{uid}-body");
    let show = t!(cw_show).resolve(cx).into_owned();
    let hide = t!(cw_hide).resolve(cx).into_owned();
    Ok(view! {
        <div class="cw" data-c="content-warning">
            <div class="head">
                icon(name: Icon::WarningDiamond, size: 20)
                <span class="reason" translate="no">
                    emote_text(text: cw, emojis: emojis)
                </span>
                <button
                    type="button"
                    aria-expanded="false"
                    aria-controls=(&body_id)
                    data-act="cw"
                    data-show=(&show)
                    data-hide=(&hide)
                >
                    (&show)
                </button>
            </div>
            <div class="veil" aria-hidden="true">dither_fill(level: 0.5)</div>
            <div id=(&body_id) class="body" hidden="">(child)</div>
        </div>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[component]
    async fn demo(cx: &Cx) -> Result<impl View> {
        Ok(view! { content_warning(cw: "spoilers", <p>"secret"</p>) })
    }

    #[test]
    fn the_body_is_hidden_until_the_button_opens_it() {
        let html = render("en", |cx| view! { demo() }.boxed());
        assert!(html.contains(r#"class="cw""#), "{html}");
        assert!(
            html.contains(r#"<span class="reason" translate="no">spoilers</span>"#),
            "{html}"
        );
        assert!(html.contains(r#"aria-expanded="false""#), "{html}");
        assert!(html.contains("aria-controls=\""), "{html}");
        assert!(html.contains(r#"class="body" hidden"#), "{html}");
        assert!(html.contains(r#"class="veil""#), "{html}");
        assert!(html.contains(">Show</button>"), "{html}");
        assert!(html.contains(r#"data-hide="Hide""#), "{html}");
    }
}
