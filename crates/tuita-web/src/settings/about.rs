//! `/settings/about`: the instance, its rules and the keyboard shortcuts.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use crate::{
    l10n::t,
    session,
    settings::{card::setting_card, contact::contact_href, page::settings_page},
    shell::shortcuts_dialog::groups,
};

/// Domain, version, contact and links of the instance, its rules, and every shortcut.
#[component]
pub(crate) async fn about(cx: &Cx) -> Result<impl View> {
    let instance = session::instance(cx).await?;
    let title = t!(settings_about).resolve(cx).into_owned();
    let contact = instance.contact.trim().to_owned();
    let href = (!contact.is_empty())
        .then(|| contact_href(&contact))
        .flatten();
    let groups = groups(cx);
    Ok(view! {
        settings_page(
            title: &title,
            setting_card(
                title: instance.name.as_str(),
                description: instance.description.as_str(),
                <dl class="facts" data-c="about-facts">
                    <div>
                        <dt>(t!(settings_about_domain))</dt>
                        <dd>(&instance.domain)</dd>
                    </div>
                    <div>
                        <dt>(t!(settings_about_version))</dt>
                        <dd>(&instance.version)</dd>
                    </div>
                    if !contact.is_empty() {
                        <div>
                            <dt>(t!(settings_about_contact))</dt>
                            <dd>
                                if let Some(href) = &href {
                                    <a href=(href) rel="noopener noreferrer">(&contact)</a>
                                } else {
                                    (&contact)
                                }
                            </dd>
                        </div>
                    }
                </dl>
                <ul class="links" data-c="about-links">
                    <li><a href="/terms">(t!(settings_about_terms))</a></li>
                    <li><a href="/privacy">(t!(settings_about_privacy))</a></li>
                    if let Some(source) = &instance.source_url {
                        <li>
                            <a href=(source) target="_blank" rel="noopener noreferrer">
                                (t!(settings_about_source))
                            </a>
                        </li>
                    }
                </ul>
            )
            if !instance.rules.is_empty() {
                setting_card(
                    title: t!(settings_about_rules).resolve(cx).into_owned(),
                    <ol class="rules" data-c="about-rules">
                        for rule in &instance.rules {
                            <li>(rule)</li>
                        }
                    </ol>
                )
            }
            setting_card(
                title: t!(shortcuts_title).resolve(cx).into_owned(),
                <div data-c="about-shortcuts">
                    for (name, rows) in groups {
                        <section class="group">
                            <h3 class="mono-tag">(name)</h3>
                            <dl>
                                for row in rows {
                                    <div class="item">
                                        <dt>
                                            for (at, key) in row.keys.iter().enumerate() {
                                                if at > 0 {
                                                    <span class="join">(&row.join)</span>
                                                }
                                                <kbd>(*key)</kbd>
                                            }
                                        </dt>
                                        <dd>(&row.label)</dd>
                                    </div>
                                }
                            </dl>
                        </section>
                    }
                </div>
            )
        )
    })
}
