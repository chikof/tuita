//! The `/design` gallery. Every theme, ink and motion
//! setting passes axe, the toolbar persists its prefs, every indexed component has an anchor, and
//! the federation and OAuth sections show their states.

use serde_json::{Value, json};
use tuita_e2e::{Ctx, Name, Page, Result, Violation};

/// What `open` stores under `localStorage['tuita:prefs']` before the page loads.
struct Setup {
    mode: &'static str,
    ink: &'static str,
    contrast: &'static str,
    text_scale: f64,
    motion: &'static str,
}

impl Setup {
    /// A normal-contrast, 100 % text setup in `mode` and `ink` with reduced motion.
    fn calm(mode: &'static str, ink: &'static str) -> Self {
        Self {
            mode,
            ink,
            contrast: "normal",
            text_scale: 1.0,
            motion: "reduce",
        }
    }

    /// The defaults of the toolbar tests: bone ink, motion from the system.
    fn system(mode: &'static str) -> Self {
        Self {
            motion: "system",
            ..Self::calm(mode, "bone")
        }
    }
}

/// Opens `/design` with `prefs` already stored and the fonts loaded.
async fn open(setup: &Setup) -> Result<(Ctx, Page)> {
    let ctx = Ctx::new().await?;
    let page = ctx.page().await?;
    let prefs = json!({
        "mode": setup.mode,
        "ink": setup.ink,
        "contrast": setup.contrast,
        "textScale": setup.text_scale,
        "motion": setup.motion,
    });
    page.init_script(&format!(
        "localStorage.setItem('tuita:prefs', JSON.stringify({prefs}))"
    ))
    .await?;
    page.goto("/design").await?;
    page.css("h1")
        .expect_text(Name::Exact("Design system"))
        .await?;
    page.eval("document.fonts.ready.then(() => true)").await?;
    // Staggered entrances are transparent until their turn; axe would read the mid-fade colours.
    page.wait_for_js(
        "[...document.querySelectorAll('.enter')].every((el) => el.getAnimations().every((a) => a.playState === 'finished'))",
    )
    .await?;
    Ok((ctx, page))
}

fn assert_clean(what: &str, violations: &[Violation]) {
    assert!(
        violations.is_empty(),
        "axe violations in {what}:\n{}",
        Violation::report(violations)
    );
}

async fn assert_no_console_errors(page: &Page) {
    let errors = page.console_errors().await;
    assert!(errors.is_empty(), "console errors: {errors:#?}");
}

/// Declares one `#[tokio::test]` per `name: theme, ink` that calls `$run(theme, ink)`.
macro_rules! theme_ink_tests {
    ($run:path; $($name:ident: $theme:literal, $ink:literal;)+) => {
        $(
            #[tokio::test]
            async fn $name() -> Result<()> {
                $run($theme, $ink).await
            }
        )+
    };
}

/// `theme_ink_tests!` over both themes and every ink.
macro_rules! every_theme_and_ink {
    ($run:path) => {
        theme_ink_tests! { $run;
            dark_bone: "dark", "bone";
            dark_phosphor: "dark", "phosphor";
            dark_amber: "dark", "amber";
            dark_cyan: "dark", "cyan";
            dark_rose: "dark", "rose";
            light_bone: "light", "bone";
            light_phosphor: "light", "phosphor";
            light_amber: "light", "amber";
            light_cyan: "light", "cyan";
            light_rose: "light", "rose";
        }
    };
}

mod gallery {
    use super::*;

    async fn run(theme: &'static str, ink: &'static str, reduced: bool) -> Result<()> {
        let mut setup = Setup::calm(theme, ink);
        if !reduced {
            setup.motion = "system";
        }
        let (_ctx, page) = open(&setup).await?;

        let root = page.css("html");
        root.expect_attr("data-theme", Name::Exact(theme)).await?;
        root.expect_attr("data-ink", Name::Exact(ink)).await?;
        if reduced {
            root.expect_attr("data-motion", Name::Exact("reduce"))
                .await?;
        } else {
            assert_eq!(
                page.eval("document.documentElement.hasAttribute('data-motion')")
                    .await?,
                Value::Bool(false),
                "motion follows the system"
            );
        }

        assert_clean("the whole gallery", &page.axe().await?);
        let motion = if reduced { "reduced" } else { "motion" };
        page.screenshot_full(format!("target/e2e/design/{theme}-{ink}-{motion}.png"))
            .await?;
        assert_no_console_errors(&page).await;
        Ok(())
    }

    #[tokio::test]
    async fn dark_bone_motion() -> Result<()> {
        run("dark", "bone", false).await
    }

    #[tokio::test]
    async fn dark_bone_reduced() -> Result<()> {
        run("dark", "bone", true).await
    }

    #[tokio::test]
    async fn dark_phosphor_motion() -> Result<()> {
        run("dark", "phosphor", false).await
    }

    #[tokio::test]
    async fn dark_phosphor_reduced() -> Result<()> {
        run("dark", "phosphor", true).await
    }

    #[tokio::test]
    async fn light_bone_motion() -> Result<()> {
        run("light", "bone", false).await
    }

    #[tokio::test]
    async fn light_bone_reduced() -> Result<()> {
        run("light", "bone", true).await
    }

    #[tokio::test]
    async fn light_phosphor_motion() -> Result<()> {
        run("light", "phosphor", false).await
    }

    #[tokio::test]
    async fn light_phosphor_reduced() -> Result<()> {
        run("light", "phosphor", true).await
    }
}

#[tokio::test]
async fn more_contrast_at_125_percent_text() -> Result<()> {
    let (_ctx, page) = open(&Setup {
        mode: "dark",
        ink: "amber",
        contrast: "more",
        text_scale: 1.25,
        motion: "reduce",
    })
    .await?;
    page.css("html")
        .expect_attr("data-contrast", Name::Exact("more"))
        .await?;
    assert_clean("the whole gallery", &page.axe().await?);
    assert_no_console_errors(&page).await;
    Ok(())
}

#[tokio::test]
async fn toolbar_changes_theme_ink_and_persists_prefs() -> Result<()> {
    let (_ctx, page) = open(&Setup::system("dark")).await?;
    let root = page.css("html");

    page.role("tab", Name::Exact("Light")).click().await?;
    root.expect_attr("data-theme", Name::Exact("light")).await?;
    page.role("tab", Name::Exact("cyan")).click().await?;
    root.expect_attr("data-ink", Name::Exact("cyan")).await?;
    page.role("tab", Name::Exact("125%")).click().await?;
    root.expect_css("--text-scale", Name::Exact("1.25")).await?;

    let stored = page
        .eval("JSON.parse(localStorage.getItem('tuita:prefs') ?? '{}')")
        .await?;
    assert_eq!(stored["mode"], "light", "{stored}");
    assert_eq!(stored["ink"], "cyan", "{stored}");
    assert_eq!(stored["textScale"], 1.25, "{stored}");
    assert_no_console_errors(&page).await;
    Ok(())
}

#[tokio::test]
async fn every_component_has_an_anchor() -> Result<()> {
    let (_ctx, page) = open(&Setup::system("dark")).await?;
    let links = page
        .eval(
            "[...document.querySelectorAll('nav[aria-label=\"Components\"] a')]
                .map((a) => a.getAttribute('href').slice(1))",
        )
        .await?;
    let ids: Vec<&str> = links
        .as_array()
        .map(|all| all.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    assert!(ids.len() >= 36, "{} anchors", ids.len());
    for id in ids {
        page.css(&format!("section#{id}")).expect_count(1).await?;
    }
    Ok(())
}

mod m3 {
    use super::*;

    const SECTIONS: [&str; 12] = [
        "stat-card",
        "bar-chart",
        "sparkline",
        "heatmap",
        "admin-bento",
        "announcement-banner",
        "select",
        "admin-tabs",
        "report-card",
        "admin-user-row",
        "post-preview",
        "token-secret",
    ];

    async fn run(theme: &'static str, ink: &'static str) -> Result<()> {
        let (_ctx, page) = open(&Setup::calm(theme, ink)).await?;
        let selectors: Vec<String> = SECTIONS.iter().map(|id| format!("section#{id}")).collect();
        let selectors: Vec<&str> = selectors.iter().map(String::as_str).collect();
        assert_clean("the M3 sections", &page.axe_include(&selectors).await?);
        assert_no_console_errors(&page).await;
        Ok(())
    }

    every_theme_and_ink!(run);
}

#[tokio::test]
async fn charts_offer_a_text_summary_and_a_data_table() -> Result<()> {
    let (_ctx, page) = open(&Setup::calm("dark", "bone")).await?;
    let section = page.css("section#bar-chart");
    section
        .css("[role=\"img\"]")
        .first()
        .expect_attr(
            "aria-label",
            Name::Re("Posts per day.?: .* in total, most on "),
        )
        .await?;
    section
        .role("button", Name::Exact("Show Posts per day as a table"))
        .first()
        .click()
        .await?;
    page.role("dialog", Name::Any)
        .role("table", Name::Any)
        .role("row", Name::Any)
        .expect_count(15)
        .await
}

mod m4 {
    use super::*;

    const SECTIONS: [&str; 11] = [
        "queue-summary",
        "actor-key",
        "domain-block-row",
        "domain-chip",
        "visibility-badge",
        "remote-post",
        "remote-user",
        "remote-profile",
        "remote-notifications",
        "remote-lookup",
        "user-gate",
    ];

    async fn run(theme: &'static str, ink: &'static str) -> Result<()> {
        let (_ctx, page) = open(&Setup::calm(theme, ink)).await?;
        let selectors: Vec<String> = SECTIONS.iter().map(|id| format!("section#{id}")).collect();
        let selectors: Vec<&str> = selectors.iter().map(String::as_str).collect();
        assert_clean("the M4 sections", &page.axe_include(&selectors).await?);
        assert_no_console_errors(&page).await;
        Ok(())
    }

    every_theme_and_ink!(run);
}

#[tokio::test]
async fn remote_components_show_the_domain_the_original_link_and_the_follow_states() -> Result<()> {
    let (_ctx, page) = open(&Setup::calm("dark", "bone")).await?;

    let posts = page.css("section#remote-post");
    posts
        .css("[data-domain=\"fedi.example\"]")
        .first()
        .expect_text(Name::Exact("fedi.example"))
        .await?;
    posts
        .role("link", "View on fedi.example")
        .first()
        .expect_attr(
            "href",
            Name::Exact("https://fedi.example/@mara/112233445566"),
        )
        .await?;
    posts
        .css("[data-visibility=\"unlisted\"]")
        .expect_text(Name::Exact("Unlisted"))
        .await?;
    posts
        .css("[data-visibility=\"followers\"]")
        .expect_text(Name::Exact("Followers only"))
        .await?;
    posts
        .css("[data-visibility=\"direct\"]")
        .expect_text(Name::Exact("Direct"))
        .await?;

    let users = page.css("section#remote-user");
    users
        .role("button", Name::Exact("Follow"))
        .first()
        .expect_visible()
        .await?;
    users
        .role("button", "Requested")
        .first()
        .expect_visible()
        .await?;
    users
        .role("button", "Following")
        .first()
        .expect_visible()
        .await?;

    let profile = page.css("section#remote-profile").first();
    profile
        .text("Approves followers")
        .first()
        .expect_visible()
        .await?;
    profile
        .role("link", "View profile on fedi.example")
        .first()
        .expect_attr("href", Name::Exact("https://fedi.example/@mara"))
        .await
}

mod m5 {
    use super::*;

    const LANGUAGES: [&str; 12] = [
        "English",
        "Polski",
        "Deutsch",
        "Español",
        "Français",
        "Português",
        "Українська",
        "Italiano",
        "Nederlands",
        "Türkçe",
        "Čeština",
        "日本語",
    ];

    async fn run(theme: &'static str, ink: &'static str) -> Result<()> {
        let (_ctx, page) = open(&Setup::calm(theme, ink)).await?;
        let picker = page.css("section#language-picker");
        picker
            .label("Language")
            .expect_value(Name::Exact("en"))
            .await?;
        assert_eq!(picker.css("option").texts().await?, LANGUAGES);
        assert_clean(
            "the language picker",
            &page.axe_include(&["section#language-picker"]).await?,
        );
        assert_no_console_errors(&page).await;
        Ok(())
    }

    every_theme_and_ink!(run);
}

mod m6 {
    use super::*;

    async fn run(theme: &'static str, ink: &'static str) -> Result<()> {
        let (_ctx, page) = open(&Setup::calm(theme, ink)).await?;
        let section = page.css("section#consent-card");
        for (state, count) in [
            ("loading", 1),
            ("ask", 6),
            ("failed", 3),
            ("sent", 2),
            ("code", 1),
            ("denied", 1),
        ] {
            section
                .css(&format!("[data-state^=\"{state}\"]"))
                .expect_count(count)
                .await
                .map_err(|e| e.context(format!("the {state} state")))?;
        }
        section
            .css("[data-app-name]")
            .first()
            .expect_text(Name::Exact("Tuba"))
            .await?;
        section.css("[data-scope]").expect_count(12).await?;
        section
            .role("button", "Authorize")
            .first()
            .expect_enabled()
            .await?;
        section
            .css("button[aria-busy=\"true\"]")
            .expect_count(1)
            .await?;
        section
            .role("link", "https://tuba.example")
            .first()
            .expect_attr("href", Name::Exact("https://tuba.example/"))
            .await?;
        // A website that is not http(s) is text, never a link.
        section
            .role("link", Name::Re("^javascript:"))
            .expect_count(0)
            .await?;
        section
            .role("textbox", "Authorization code")
            .expect_visible()
            .await?;

        assert_clean(
            "the consent card",
            &page.axe_include(&["section#consent-card"]).await?,
        );
        assert_no_console_errors(&page).await;
        Ok(())
    }

    every_theme_and_ink!(run);
}
