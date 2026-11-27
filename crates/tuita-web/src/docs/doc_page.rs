//! The terms and the privacy policy as a page.

use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    router::request::headers,
    view::{View, component, view},
};

use crate::{
    docs::sections::{Block, DocCtx, Section, privacy_sections, terms_sections},
    l10n::t,
    session, shell,
    shell::page_bar::page_bar,
    ui::brand::brand,
};

/// Which document a [`doc_page`] shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DocKind {
    Terms,
    Privacy,
}

fn title_of(cx: &Cx, kind: DocKind) -> String {
    match kind {
        DocKind::Terms => t!(docs_terms_title).resolve(cx).into_owned(),
        DocKind::Privacy => t!(docs_privacy_title).resolve(cx).into_owned(),
    }
}

/// The tab title: `Terms of Service · tuita`.
///
/// # Errors
///
/// Fails when the viewer cannot be loaded.
pub(crate) async fn title(cx: &Cx, kind: DocKind) -> Result<Metadata> {
    shell::titled(cx, &title_of(cx, kind)).await
}

/// Where the instance contact leads: a web address as it is, an e-mail address as `mailto:`,
/// anything else nowhere.
#[must_use]
pub(crate) fn contact_href(contact: &str) -> Option<String> {
    let lower = contact.to_ascii_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") {
        Some(contact.to_owned())
    } else if contact.contains('@') {
        Some(format!("mailto:{contact}"))
    } else {
        None
    }
}

/// The instance facts the texts mention, with this site's host standing in for an empty domain.
async fn doc_ctx(cx: &Cx) -> Result<DocCtx> {
    let instance = session::instance(cx).await?;
    let host = headers(cx)
        .get("host")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    Ok(DocCtx {
        name: if instance.name.is_empty() {
            t!(app_name).resolve(cx).into_owned()
        } else {
            instance.name.clone()
        },
        domain: if instance.domain.is_empty() {
            host
        } else {
            instance.domain.clone()
        },
        min_age: instance.min_age,
        rules: instance.rules.clone(),
    })
}

/// The document itself; `heading` adds the title as `h1` (the standalone page has no bar).
#[component]
async fn doc_article(cx: &Cx, kind: DocKind, heading: bool) -> Result<impl View> {
    let ctx = doc_ctx(cx).await?;
    let sections: Vec<Section> = match kind {
        DocKind::Terms => terms_sections(cx, &ctx),
        DocKind::Privacy => privacy_sections(cx, &ctx),
    };
    let title = title_of(cx, kind);
    let contact = session::instance(cx).await?.contact.trim().to_owned();
    let href = contact_href(&contact);
    let fallback = t!(docs_contact_fallback).resolve(cx).into_owned();
    let updated = t!(docs_updated, date: t!(docs_date).resolve(cx).into_owned())
        .resolve(cx)
        .into_owned();
    Ok(view! {
        <article class="doc" data-c="doc-article">
            if heading {
                <h1>(&title)</h1>
            }
            <p class="updated">(&updated)</p>
            for section in &sections {
                <section id=(section.id)>
                    <h2>(&section.title)</h2>
                    for block in &section.blocks {
                        if let Block::P(text) = block {
                            <p>(text)</p>
                        } else if let Block::Ul(items) = block {
                            <ul>
                                for item in items {
                                    <li>(item)</li>
                                }
                            </ul>
                        } else if let Block::Ol(items) = block {
                            <ol>
                                for item in items {
                                    <li>(item)</li>
                                }
                            </ol>
                        } else if let Block::Contact(lead) = block {
                            <p>
                                (format!("{lead} "))
                                if let Some(href) = href.as_deref() {
                                    <a href=(href) rel="noopener">(&contact)</a>
                                } else if !contact.is_empty() {
                                    (&contact)
                                } else {
                                    (&fallback)
                                }
                                "."
                            </p>
                        }
                    }
                </section>
            }
            <p class="see">
                if kind == DocKind::Terms {
                    <a href="/privacy">(t!(docs_see_privacy))</a>
                } else {
                    <a href="/terms">(t!(docs_see_terms))</a>
                }
            </p>
        </article>
    })
}

/// One document. A signed-in visitor gets it under a page bar inside the shell; a guest gets a
/// standalone page with the brand and the title as `h1`.
#[component]
pub(crate) async fn doc_page(cx: &Cx, kind: DocKind) -> Result<impl View> {
    let member = session::me(cx).await?.is_some();
    let name = doc_ctx(cx).await?.name;
    let title = title_of(cx, kind);
    Ok(view! {
        if member {
            page_bar(title: &title, sub: name.as_str(), back: true)
            doc_article(kind: kind, heading: false)
        } else {
            <main class="solo" data-c="doc-page">
                <header><a href="/" class="home">brand(size: 28)</a></header>
                doc_article(kind: kind, heading: true)
            </main>
        }
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn a_contact_is_a_link_only_when_it_is_an_address() {
        assert_eq!(
            contact_href("https://example.org/contact").as_deref(),
            Some("https://example.org/contact")
        );
        assert_eq!(
            contact_href("HTTP://example.org").as_deref(),
            Some("HTTP://example.org")
        );
        assert_eq!(
            contact_href("admin@example.org").as_deref(),
            Some("mailto:admin@example.org")
        );
        assert_eq!(contact_href("the admins"), None);
        assert_eq!(contact_href(""), None);
    }
}
