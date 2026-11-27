//! The developer docs: authentication, the endpoint table, the event
//! stream and two example bots.

use topcoat::{
    Result,
    context::{Cx, context},
    metadata::Metadata,
    view::{View, component, view},
};
use tuita_server::dto::TokenScope;

use crate::{
    api::Web,
    docs::{
        code::{PYTHON_BOT, SSE_SAMPLE, curl_me, curl_post, node_bot},
        endpoints::{ENDPOINTS, scope_for},
    },
    l10n::t,
    session, shell,
    shell::page_bar::page_bar,
    ui::brand::brand,
};

/// The tab title: `Developers · tuita`.
///
/// # Errors
///
/// Fails when the viewer cannot be loaded.
pub(crate) async fn title(cx: &Cx) -> Result<Metadata> {
    let title = t!(dev_title).resolve(cx).into_owned();
    shell::titled(cx, &title).await
}

/// A code block. A keyboard user reaches one that scrolls sideways by tabbing to it.
#[component]
async fn code_block(cx: &Cx, text: &str) -> Result<impl View> {
    Ok(view! { <pre tabindex="0"><code>(text)</code></pre> })
}

/// The documentation; `heading` adds the page title as `h1` (the standalone page has no bar).
#[component]
async fn article(cx: &Cx, name: &str, base: &str, heading: bool) -> Result<impl View> {
    let me = curl_me(base);
    let post = curl_post(base);
    let node = node_bot(base);
    let scopes = [
        (
            TokenScope::Read,
            t!(token_scope_read_hint).resolve(cx).into_owned(),
        ),
        (
            TokenScope::Write,
            t!(token_scope_write_hint).resolve(cx).into_owned(),
        ),
        (
            TokenScope::Messages,
            t!(token_scope_messages_hint).resolve(cx).into_owned(),
        ),
    ];
    let rows: Vec<_> = ENDPOINTS
        .iter()
        .map(|endpoint| {
            (
                endpoint.method,
                format!("/api/v1{}", endpoint.path),
                scope_for(endpoint.method, endpoint.path).map_or("", TokenScope::as_str),
                (endpoint.what)(cx),
            )
        })
        .collect();
    Ok(view! {
        <article class="doc" data-c="developers">
            if heading {
                <h1>(t!(dev_hero))</h1>
            }
            <p class="updated">(t!(dev_hero_sub, name: name))</p>

            <section id="intro">
                <h2>(t!(dev_intro_h, name: name))</h2>
                <p>(t!(dev_intro_p, name: name))</p>
                <ol>
                    <li>(t!(dev_intro_1))</li>
                    <li>(t!(dev_intro_2))</li>
                    <li>(t!(dev_intro_3))</li>
                </ol>
            </section>

            <section id="auth">
                <h2>(t!(dev_auth_h))</h2>
                <p>(t!(dev_auth_p))</p>
                code_block(text: &me)
                <p>(t!(dev_auth_prefix))</p>
                <ul>
                    for (scope, hint) in &scopes {
                        <li>
                            <code>(scope.as_str())</code>
                            (format!(": {hint}"))
                        </li>
                    }
                </ul>
                <p>(t!(dev_auth_refused))</p>
                <p>(t!(dev_auth_errors))</p>
            </section>

            <section id="post">
                <h2>(t!(dev_post_h))</h2>
                code_block(text: &post)
                <p>(t!(dev_post_p))</p>
            </section>

            <section id="endpoints">
                <h2>(t!(dev_endpoints_h))</h2>
                <p>(t!(dev_endpoints_intro))</p>
                <table>
                    <thead>
                        <tr>
                            <th scope="col">(t!(dev_col_method))</th>
                            <th scope="col">(t!(dev_col_path))</th>
                            <th scope="col">(t!(dev_col_scope))</th>
                            <th scope="col">(t!(dev_col_what))</th>
                        </tr>
                    </thead>
                    <tbody>
                        for (method, path, scope, what) in &rows {
                            <tr>
                                <td class="method">(*method)</td>
                                <td class="path"><code>(path)</code></td>
                                <td class="scope">(*scope)</td>
                                <td class="what">(what)</td>
                            </tr>
                        }
                    </tbody>
                </table>
            </section>

            <section id="realtime">
                <h2>(t!(dev_realtime_h))</h2>
                <p>(t!(dev_realtime_p))</p>
                code_block(text: SSE_SAMPLE)
            </section>

            <section id="example">
                <h2>(t!(dev_example_h))</h2>
                <p>(t!(dev_example_node))</p>
                code_block(text: &node)
                <p>(t!(dev_example_python))</p>
                code_block(text: PYTHON_BOT)
            </section>

            <section id="limits">
                <h2>(t!(dev_limits_h))</h2>
                <ul>
                    <li>(t!(dev_limit_rate))</li>
                    <li>(t!(dev_limit_tokens))</li>
                    <li>(t!(dev_limit_posting))</li>
                </ul>
            </section>

            <section id="rules">
                <h2>(t!(dev_rules_h))</h2>
                <ul>
                    <li>(t!(dev_rule_1))</li>
                    <li>(t!(dev_rule_2))</li>
                    <li>(t!(dev_rule_3))</li>
                    <li><a href="/terms">(t!(dev_rule_4))</a></li>
                </ul>
            </section>
        </article>
    })
}

/// The developer docs. A signed-in visitor gets them under a page bar inside the shell; a guest
/// gets a standalone page with the brand and `h1`. The examples name this instance.
#[component]
pub(crate) async fn developers(cx: &Cx) -> Result<impl View> {
    let instance = session::instance(cx).await?;
    let name = if instance.name.is_empty() {
        t!(app_name).resolve(cx).into_owned()
    } else {
        instance.name.clone()
    };
    let base = context::<Web>(cx).state.config().public_url.to_string();
    let member = session::me(cx).await?.is_some();
    let title = t!(dev_title).resolve(cx).into_owned();
    Ok(view! {
        if member {
            page_bar(title: &title, sub: name.as_str(), back: true)
            article(name: &name, base: &base, heading: false)
        } else {
            <main class="solo" data-c="developers-page">
                <header><a href="/" class="home">brand(size: 28)</a></header>
                article(name: &name, base: &base, heading: true)
            </main>
        }
    })
}
