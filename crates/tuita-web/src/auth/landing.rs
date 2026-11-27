//! The guest home page.

use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    view::{View, attributes, component, view},
};
use tuita_server::dto::TrendDto;

use crate::{
    api,
    auth::{SignupState, language_picker::language_picker, signup_state},
    l10n::t,
    session,
    shell::parts::nav_button,
    ui::{
        bento::bento,
        brand::brand,
        button::ButtonVariant,
        card::{CardLayout, card},
        dither_field::dither_field,
        dither_number::dither_number,
        dither_text::dither_text,
    },
};

/// How many trends the landing page lists.
const TRENDS: usize = 8;

/// The tab title of `/` for a guest: `<instance name> · <tagline>`.
///
/// # Errors
///
/// Fails when the instance cannot be loaded.
pub(crate) async fn title(cx: &Cx) -> Result<Metadata> {
    let instance = session::instance(cx).await?;
    let name = instance_name(cx, &instance.name);
    let tagline = t!(tagline).resolve(cx);
    Ok(Metadata::new().title(format!("{name} · {tagline}")))
}

fn instance_name(cx: &Cx, name: &str) -> String {
    if name.is_empty() {
        t!(app_name).resolve(cx).into_owned()
    } else {
        name.to_owned()
    }
}

/// What a guest sees at `/`: the instance in a grid of cards (name and tagline, counts, a clock,
/// trends, rules and how to join), the language picker and the legal links.
///
/// The clock is `[data-island="clock"]`, which `shell.js` keeps ticking in the visitor's time
/// zone.
#[component]
pub(crate) async fn landing(cx: &Cx) -> Result<impl View> {
    let instance = session::instance(cx).await?;
    let state = signup_state(instance);
    let name = instance_name(cx, &instance.name);
    let mut trends = api::get::<Vec<TrendDto>>(cx, "/trends")
        .await
        .unwrap_or_default();
    trends.truncate(TRENDS);
    let trends: Vec<_> = trends
        .into_iter()
        .map(|trend| {
            let count = i32::try_from(trend.posts).unwrap_or(i32::MAX);
            let posts = t!(landing_trend, count: count).resolve(cx).into_owned();
            (trend.tag, posts)
        })
        .collect();
    let rules = &instance.rules;
    let rule_rows = u8::try_from((1 + rules.len().div_ceil(2)).min(4)).unwrap_or(4);
    let users = u64::try_from(instance.stats.users).unwrap_or(0);
    let posts = u64::try_from(instance.stats.posts).unwrap_or(0);
    let join = t!(landing_join, name: name.as_str())
        .resolve(cx)
        .into_owned();
    let version = t!(landing_powered, version: instance.version.as_str())
        .resolve(cx)
        .into_owned();
    let open = state != SignupState::Closed;
    let sign_in_variant = if open {
        ButtonVariant::Outline
    } else {
        ButtonVariant::Solid
    };
    Ok(view! {
        <main data-c="landing" data-grain="">
            <header>
                <a class="home" href="/">brand(size: 30)</a>
                <div class="language">language_picker()</div>
            </header>
            bento(
                card(
                    span: 3,
                    rows: 3,
                    entrance: 0u32,
                    attrs: attributes! { class="hero" },
                    dither_field(opacity: 0.3)
                    <div class="hero-body">
                        <h1>dither_text((&name))</h1>
                        <p class="tagline">(t!(tagline))</p>
                        if !instance.description.is_empty() {
                            <p class="desc">(&instance.description)</p>
                        }
                        <div class="cta">
                            if open {
                                nav_button(href: "/signup", (t!(auth_create)))
                            }
                            nav_button(
                                href: "/login",
                                variant: sign_in_variant,
                                (t!(auth_signin))
                            )
                        </div>
                    </div>
                )
                card(
                    rows: 2,
                    entrance: 1u32,
                    layout: CardLayout::Spread,
                    <h2>(t!(landing_stats))</h2>
                    <dl>
                        <div>
                            <dt>(t!(landing_people))</dt>
                            <dd>dither_number(value: users)</dd>
                        </div>
                        <div>
                            <dt>(t!(landing_posts))</dt>
                            <dd>dither_number(value: posts)</dd>
                        </div>
                    </dl>
                )
                card(
                    entrance: 2u32,
                    layout: CardLayout::Spread,
                    <h2>(t!(landing_clock))</h2>
                    <p class="clock" aria-hidden="true" data-island="clock">
                        <span data-part="h">"00"</span>
                        <span class="colon">":"</span>
                        <span data-part="m">"00"</span>
                        <span class="colon">":"</span>
                        <span data-part="s">"00"</span>
                    </p>
                )
                if !trends.is_empty() {
                    card(
                        span: 2,
                        rows: 2,
                        entrance: 3u32,
                        <h2>(t!(landing_trends))</h2>
                        <ul class="trends">
                            for (tag, posts) in trends {
                                <li>
                                    <span class="tag">
                                        "#"
                                        (tag)
                                    </span>
                                    <span class="count">(posts)</span>
                                </li>
                            }
                        </ul>
                    )
                }
                if !rules.is_empty() {
                    card(
                        span: 2,
                        rows: rule_rows,
                        entrance: 4u32,
                        <h2>(t!(landing_rules))</h2>
                        <ol class="rules">
                            for rule in rules {
                                <li>(rule)</li>
                            }
                        </ol>
                    )
                }
                card(
                    span: 4,
                    entrance: 5u32,
                    layout: CardLayout::Spread,
                    <h2>(join)</h2>
                    if state == SignupState::Open {
                        <p>(t!(landing_join_sub))</p>
                    } else if state == SignupState::Invite {
                        <p>(t!(landing_invite))</p>
                    } else {
                        <p>(t!(landing_closed))</p>
                    }
                )
            )
            <footer aria-label=(t!(landing_nav))>
                <a href="/terms">(t!(landing_terms))</a>
                <a href="/privacy">(t!(landing_privacy))</a>
                <a href="/developers">(t!(landing_developers))</a>
                if let Some(source) = &instance.source_url {
                    <a href=(source) target="_blank" rel="noopener">
                        (t!(landing_source))
                    </a>
                }
                <span>(version)</span>
            </footer>
        </main>
    })
}
