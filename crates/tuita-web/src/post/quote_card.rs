//! A quoted post.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};
use tuita_server::dto::QuoteDto;

use crate::{
    fmt::{acct_of, handle_of},
    l10n::t,
    paths,
    post::rich_text::{around, emote_text, user_name},
    ui::avatar::{AvatarShape, avatar},
};

/// The post a quote post embeds, or a note that it is gone.
#[component]
pub(crate) async fn quote_card(cx: &Cx, quote: &QuoteDto) -> Result<impl View> {
    let (before, after) = around(&t!(cw_label, reason: "\u{0}").resolve(cx));
    let when = crate::fmt::rel_label(cx, &time_text(quote));
    let author = quote.author.as_ref().filter(|_| quote.available);
    Ok(view! {
        if let Some(author) = author {
            <a
                class="quote"
                href=(paths::post(
                    &acct_of(&author.handle, author.domain.as_deref()),
                    &quote.id.to_string(),
                ))
                data-c="quote-card"
            >
                <span class="head">
                    <span aria-hidden="true">
                        avatar(
                            seed: &author.handle,
                            src: author.avatar_url.as_deref(),
                            alt: "",
                            shape: AvatarShape::from_name(author.avatar_shape.as_str()).unwrap_or_default(

                            ),
                            size: 24,
                            ink: author.ink.as_str()
                        )
                    </span>
                    user_name(
                        name: &author.display_name,
                        verified: author.verified,
                        bot: author.bot,
                        emojis: &author.emojis
                    )
                    <span class="dim">
                        (handle_of(&author.handle, author.domain.as_deref()))
                        " · "
                        (when)
                    </span>
                </span>
                if !quote.cw.is_empty() {
                    <span class="dim" translate="no">
                        (before)
                        emote_text(text: &quote.cw, emojis: &quote.emojis)
                        (after)
                    </span>
                } else {
                    <span class="text" translate="no">
                        emote_text(text: &quote.text, emojis: &quote.emojis)
                    </span>
                }
            </a>
        } else {
            <p class="quote gone" data-c="quote-card">(t!(post_unavailable))</p>
        }
    })
}

/// RFC 3339 text of the quote's creation time, which `rel_label` parses.
fn time_text(quote: &QuoteDto) -> String {
    quote
        .created_at
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default()
}
