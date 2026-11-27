//! A link preview.

use topcoat::{
    Result,
    context::Cx,
    view::{View, class, component, view},
};
use tuita_server::dto::LinkCardDto;

use crate::{icons::Icon, ui::icon::icon};

/// The host of the card's URL without `www.`, or the site name when the URL does not parse.
fn host(card: &LinkCardDto) -> String {
    url::Url::parse(&card.url)
        .ok()
        .and_then(|parsed| {
            parsed
                .host_str()
                .map(|host| host.strip_prefix("www.").unwrap_or(host).to_owned())
        })
        .unwrap_or_else(|| card.site.clone())
}

/// A preview of the first link of a post: picture or link icon, site, title and description.
#[component]
pub(crate) async fn link_card(cx: &Cx, card: &LinkCardDto) -> Result<impl View> {
    let host = host(card);
    let site = if !card.site.is_empty() && card.site != host {
        format!("{} · {host}", card.site)
    } else {
        host
    };
    let large = card.large && card.image.is_some();
    Ok(view! {
        <a
            class=(class!("card", "large" if large))
            href=(&card.url)
            target="_blank"
            rel="noopener noreferrer nofollow"
            translate="no"
            data-c="link-card"
        >
            <span class="media">
                if let Some(image) = &card.image {
                    <img src=(image) alt="" loading="lazy" decoding="async">
                } else {
                    icon(name: Icon::Link, size: 24)
                }
            </span>
            <span class="text">
                <span class="site">(site)</span>
                <span class="title">(&card.title)</span>
                if !card.description.is_empty() {
                    <span class="desc">(&card.description)</span>
                }
            </span>
        </a>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    fn card(image: Option<&str>, large: bool) -> LinkCardDto {
        LinkCardDto {
            url: "https://www.example.com/a".to_owned(),
            title: "A title".to_owned(),
            description: "About it".to_owned(),
            site: "Example".to_owned(),
            image: image.map(str::to_owned),
            large,
        }
    }

    #[test]
    fn the_card_opens_the_link_safely_and_names_site_and_host() {
        let card = card(Some("/img.png"), true);
        let html = render("en", |cx| view! { link_card(card: &card) }.boxed());
        assert!(html.contains(r#"class="card large""#), "{html}");
        assert!(html.contains(r#"target="_blank""#), "{html}");
        assert!(
            html.contains(r#"rel="noopener noreferrer nofollow""#),
            "{html}"
        );
        assert!(
            html.contains(r#"<span class="site">Example · example.com</span>"#),
            "{html}"
        );
        assert!(
            html.contains(r#"<span class="desc">About it</span>"#),
            "{html}"
        );
    }

    #[test]
    fn a_large_card_without_a_picture_stays_small_and_shows_the_link_icon() {
        let card = card(None, true);
        let html = render("en", |cx| view! { link_card(card: &card) }.boxed());
        assert!(html.contains(r#"class="card""#), "{html}");
        assert!(html.contains("<svg"), "{html}");
    }
}
