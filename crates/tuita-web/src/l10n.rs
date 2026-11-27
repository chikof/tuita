//! Messages: the Fluent files under `l10n/`, `t!`, the strings islands read, flash messages.
//!
//! Every user-visible text is `t!(key, var: value)` keyed by message name. Import
//! the macro with `use crate::l10n::t;`. Add a key to all twelve files under a `## <task-id>`
//! comment.

use topcoat::{context::Cx, l10n::Locale};

topcoat::l10n::fluent::messages!(pub(crate) "{mod}/{locale}.ftl");

/// Fluent wraps every interpolation in these two isolation marks.
const ISOLATES: [char; 2] = ['\u{2068}', '\u{2069}'];

/// `text` without Fluent's isolation marks.
#[must_use]
pub(crate) fn strip_isolates(text: &str) -> String {
    text.chars().filter(|c| !ISOLATES.contains(c)).collect()
}

/// Counts every plural message is rendered for: one sample per plural category of the shipped
/// languages (`0 1 2 5` and a large one that is `many` or `other` everywhere).
const PLURAL_SAMPLES: [i32; 5] = [0, 1, 2, 5, 1_000_000];

/// A plural message as the browser needs it: the text of each sample with `{count}` in place of
/// the number. A script picks the sample whose `Intl.PluralRules` category matches the real count.
fn plural_value(
    texts: impl IntoIterator<Item = (i32, String)>,
    locale: &Locale,
) -> serde_json::Value {
    let map = texts
        .into_iter()
        .map(|(sample, text)| {
            let shown = topcoat::l10n::Number::new(f64::from(sample)).resolve_in(locale);
            let text = strip_isolates(&text).replace(&*shown, "{count}");
            (sample.to_string(), serde_json::Value::String(text))
        })
        .collect();
    serde_json::Value::Object(map)
}

/// Declares the messages islands translate in the browser.
///
/// `plain { key(var, ..) }` entries become `"key": "text with {var}"`; `plural { key(count, var,
/// ..) }` entries become `"key": {"0": "...", "1": "...", ...}` (see [`plural_value`]). The
/// generated function renders them for one locale as the JSON of `<script id="tuita-l10n">`.
macro_rules! client_strings {
    (
        plain { $( $key:ident $( ( $( $var:ident ),* ) )? ),* $(,)? }
        plural { $( $pkey:ident ( $count:ident $( , $pvar:ident )* ) ),* $(,)? }
    ) => {
        /// The client strings for `locale`, as JSON that is safe inside a `<script>` element.
        pub(crate) fn client_strings_json(locale: &Locale) -> String {
            let mut map = serde_json::Map::new();
            $(
                map.insert(
                    stringify!($key).to_owned(),
                    serde_json::Value::String(strip_isolates(
                        &t!($key $(, $( $var: concat!("{", stringify!($var), "}") ),* )?)
                            .resolve_in(locale),
                    )),
                );
            )*
            $(
                map.insert(
                    stringify!($pkey).to_owned(),
                    plural_value(
                        PLURAL_SAMPLES.iter().map(|&sample| {
                            let text = t!(
                                $pkey, $count: sample
                                $( , $pvar: concat!("{", stringify!($pvar), "}") )*
                            )
                            .resolve_in(locale)
                            .into_owned();
                            (sample, text)
                        }),
                        locale,
                    ),
                );
            )*
            serde_json::Value::Object(map).to_string().replace('<', "\\u003c")
        }
    };
}

client_strings! {
    plain {
        dismiss,
        time_now,
        time_minutes(n),
        time_hours(n),
        time_days(n),
        notify_like(handle),
        notify_repost(handle),
        notify_reply(handle),
        notify_mention(handle),
        notify_quote(handle),
        notify_follow(handle),
        notify_follow_request(handle),
        notify_affiliation(handle),
        notify_message(handle),
        notify_poll,
        view,
        settings_desktop_state(state),
        settings_desktop_allowed,
        settings_desktop_blocked,
        settings_desktop_not_asked,
        settings_desktop_blocked_toast,
        passkey_already_here,
        passkey_unsupported,
        composer_title,
        composer_reply_title,
        composer_quote_title,
        composer_placeholder,
        composer_reply_placeholder,
        composer_replying_to(handle),
        composer_post,
        composer_reply,
        composer_post_all,
        composer_schedule,
        composer_schedule_chip(time),
        composer_scheduled(time),
        composer_extra_label(n),
        composer_extra_remove(n),
        post_sent,
        composer_err_type,
        composer_err_image_size,
        composer_err_video_size(mb),
        composer_err_max_images,
        composer_err_exclusive,
        composer_uploading_video(percent),
        composer_alt_label(index),
        composer_alt_image,
        composer_alt_video,
        composer_remove_image,
        composer_remove_video,
        retry,
        composer_upload_failed,
        composer_send_failed,
        composer_open_failed,
        policy_everyone,
        policy_following,
        policy_mentioned,
        schedule_hour,
        schedule_evening,
        schedule_tomorrow,
        schedule_week,
        schedule_window,
        image_field_not_image,
        image_field_too_large,
        image_field_saved,
        video_position(current, total),
        link_copied,
        text_copied,
        post_deleted,
        post_pinned_toast,
        post_unpinned,
        post_reply_hidden_toast,
        post_reply_unhidden_toast,
        edit_saved,
        report_sent,
        report_duplicate,
        muted_toast(handle),
        blocked_toast(handle),
        block_title(handle),
        media_pager(index, count),
        post_like_label(count),
        e2ee_line_sealed,
        e2ee_line_own_key,
        e2ee_line_peer_key,
        e2ee_line_unreadable,
        e2ee_line_unsupported,
        e2ee_preview,
        e2ee_unlock_wrong,
        e2ee_unsupported,
        e2ee_failed,
        e2ee_passphrase_short,
        e2ee_passphrase_mismatch,
        e2ee_setup_done,
        e2ee_unlock_done,
        e2ee_forget_done,
        e2ee_reset_done,
        e2ee_block_peer_off(handle),
        e2ee_block_self_off,
        e2ee_block_downgrade(handle),
        e2ee_block_unlock,
        e2ee_block_changed(handle),
        e2ee_block_error,
        share_chat_sent(handle),
        post_reposts_label(count),
        passkey_added(name),
        token_copied,
        token_copy_failed,
        pull_refresh,
        loading,
        scheduled_posted,
        scheduled_window_error,
        scheduled_zone(zone),
        admin_inv_created(code),
        admin_inv_created_copied(code),
        admin_inv_copied,
        admin_inv_copy_failed(code),
        admin_fed_actor_copied,
        admin_fed_actor_copy_failed(fingerprint),
        follow,
        following,
        follow_requested,
        follow_cancel_request,
        follow_requested_toast(handle),
        follow_cancelled_toast,
        unmuted_toast(handle),
        unblocked_toast(handle),
    }
    plural {
        char_counter_left(count),
        char_counter_over(count),
        badge_unread(count),
        composer_thread_sent(count),
        composer_tag_posts(count, compact),
        lists_members(count, compact),
        profile_followers(count),
    }
}

/// The text of a flash message: the `tuita_flash` cookie names one of these message keys.
///
/// A key that is not listed shows nothing, so the cookie cannot be used to print arbitrary text.
#[must_use]
pub(crate) fn flash(cx: &Cx, key: &str) -> Option<String> {
    macro_rules! flash_keys {
        ($($name:ident),* $(,)?) => {
            match key {
                $(stringify!($name) => Some(t!($name).resolve(cx).into_owned()),)*
                _ => None,
            }
        };
    }
    flash_keys!(
        settings_saved,
        settings_session_signed_out,
        settings_profile_saved,
        image_field_saved,
        image_field_removed,
    )
}

#[cfg(test)]
mod test {
    use std::collections::{BTreeMap, BTreeSet};

    use fluent_syntax::{ast, parser};
    use topcoat::l10n::locale;

    use super::*;

    /// The messages of one `.ftl` file: id to the variables its patterns use.
    fn messages(source: &'static str, name: &str) -> BTreeMap<String, BTreeSet<String>> {
        let resource = parser::parse(source)
            .unwrap_or_else(|(_, errors)| panic!("{name} does not parse: {errors:?}"));
        resource
            .body
            .iter()
            .filter_map(|entry| match entry {
                ast::Entry::Message(message) => {
                    let mut vars = BTreeSet::new();
                    for pattern in message
                        .value
                        .iter()
                        .chain(message.attributes.iter().map(|a| &a.value))
                    {
                        pattern_vars(pattern, &mut vars);
                    }
                    Some((message.id.name.to_owned(), vars))
                }
                _ => None,
            })
            .collect()
    }

    fn pattern_vars(pattern: &ast::Pattern<&str>, out: &mut BTreeSet<String>) {
        for element in &pattern.elements {
            if let ast::PatternElement::Placeable { expression } = element {
                expression_vars(expression, out);
            }
        }
    }

    fn expression_vars(expression: &ast::Expression<&str>, out: &mut BTreeSet<String>) {
        match expression {
            ast::Expression::Inline(inline) => inline_vars(inline, out),
            ast::Expression::Select { selector, variants } => {
                inline_vars(selector, out);
                for variant in variants {
                    pattern_vars(&variant.value, out);
                }
            }
        }
    }

    fn inline_vars(inline: &ast::InlineExpression<&str>, out: &mut BTreeSet<String>) {
        match inline {
            ast::InlineExpression::VariableReference { id } => {
                out.insert(id.name.to_owned());
            }
            ast::InlineExpression::Placeable { expression } => expression_vars(expression, out),
            ast::InlineExpression::FunctionReference { arguments, .. }
            | ast::InlineExpression::TermReference {
                arguments: Some(arguments),
                ..
            } => {
                for argument in &arguments.positional {
                    inline_vars(argument, out);
                }
                for argument in &arguments.named {
                    inline_vars(&argument.value, out);
                }
            }
            _ => {}
        }
    }

    const FILES: [(&str, &str); 12] = [
        ("en", include_str!("l10n/en.ftl")),
        ("pl", include_str!("l10n/pl.ftl")),
        ("de", include_str!("l10n/de.ftl")),
        ("es", include_str!("l10n/es.ftl")),
        ("fr", include_str!("l10n/fr.ftl")),
        ("pt", include_str!("l10n/pt.ftl")),
        ("uk", include_str!("l10n/uk.ftl")),
        ("it", include_str!("l10n/it.ftl")),
        ("nl", include_str!("l10n/nl.ftl")),
        ("tr", include_str!("l10n/tr.ftl")),
        ("cs", include_str!("l10n/cs.ftl")),
        ("ja", include_str!("l10n/ja.ftl")),
    ];

    #[test]
    fn files_follow_the_locale_list() {
        let codes: Vec<_> = FILES.iter().map(|(code, _)| *code).collect();
        assert_eq!(codes, tuita_core::locale::LANGS);
    }

    #[test]
    fn every_locale_has_every_message() {
        let english = messages(FILES[0].1, "en");
        assert!(english.len() > 1000, "en has {} messages", english.len());
        for (code, source) in &FILES[1..] {
            let table = messages(source, code);
            let missing: Vec<_> = english
                .keys()
                .filter(|id| !table.contains_key(*id))
                .collect();
            let extra: Vec<_> = table
                .keys()
                .filter(|id| !english.contains_key(*id))
                .collect();
            assert!(missing.is_empty(), "{code} lacks {missing:?}");
            assert!(extra.is_empty(), "{code} has ids en lacks: {extra:?}");
            for (id, vars) in &english {
                assert_eq!(&table[id], vars, "{code}: variables of {id}");
            }
        }
    }

    #[test]
    fn tuita_stays_lowercase_in_every_language() {
        for (code, source) in FILES {
            assert!(
                !source.contains("Tuita") && !source.contains("TUITA"),
                "{code}"
            );
        }
    }

    #[test]
    fn plural_messages_follow_the_language() {
        let render = |locale: &Locale, count: i32| {
            strip_isolates(&t!(char_counter_left, count).resolve_in(locale))
        };
        let en = locale!("en");
        assert_eq!(render(&en, 1), "1 character left");
        assert_eq!(render(&en, 2), "2 characters left");
        let pl = locale!("pl");
        assert_eq!(render(&pl, 1), "1 znak pozostał");
        assert_eq!(render(&pl, 2), "2 znaki pozostały");
        assert_eq!(render(&pl, 5), "5 znaków pozostało");
        assert_eq!(render(&pl, 22), "22 znaki pozostały");
    }

    #[test]
    fn client_strings_hold_every_plural_sample_with_a_count_slot() {
        for (code, _) in FILES {
            let json: serde_json::Value =
                serde_json::from_str(&client_strings_json(&locale_of(code))).expect("valid json");
            assert!(
                json["time_minutes"]
                    .as_str()
                    .expect("plain")
                    .contains("{n}"),
                "{code}"
            );
            assert!(json["time_now"].is_string(), "{code}");
            let plural = json["char_counter_left"].as_object().expect("plural");
            assert_eq!(plural.len(), PLURAL_SAMPLES.len(), "{code}");
            for (sample, text) in plural {
                let text = text.as_str().expect("text");
                assert!(text.contains("{count}"), "{code} {sample}: {text}");
                assert!(!text.contains(ISOLATES), "{code} {sample}: {text}");
            }
        }
    }

    #[test]
    fn client_strings_are_inert_inside_a_script_element() {
        assert!(!client_strings_json(&locale!("en")).contains('<'));
    }

    use crate::locale::locale_of;
}

#[cfg(test)]
mod plural_categories {
    use std::collections::BTreeSet;

    use fluent_syntax::{ast, parser};
    use icu_plurals::{PluralCategory, PluralRules};

    use crate::locale::locale_of;

    const FILES: [(&str, &str); 12] = [
        ("en", include_str!("l10n/en.ftl")),
        ("pl", include_str!("l10n/pl.ftl")),
        ("de", include_str!("l10n/de.ftl")),
        ("es", include_str!("l10n/es.ftl")),
        ("fr", include_str!("l10n/fr.ftl")),
        ("pt", include_str!("l10n/pt.ftl")),
        ("uk", include_str!("l10n/uk.ftl")),
        ("it", include_str!("l10n/it.ftl")),
        ("nl", include_str!("l10n/nl.ftl")),
        ("tr", include_str!("l10n/tr.ftl")),
        ("cs", include_str!("l10n/cs.ftl")),
        ("ja", include_str!("l10n/ja.ftl")),
    ];

    /// The CLDR cardinal plural categories of `code`, by their TR35 names.
    fn cldr_categories(code: &str) -> BTreeSet<&'static str> {
        let icu = locale_of(code).into_icu();
        PluralRules::try_new_cardinal((&icu).into())
            .expect("plural rules ship with the locale data")
            .categories()
            .map(|category| match category {
                PluralCategory::Zero => "zero",
                PluralCategory::One => "one",
                PluralCategory::Two => "two",
                PluralCategory::Few => "few",
                PluralCategory::Many => "many",
                PluralCategory::Other => "other",
            })
            .collect()
    }

    /// The variant names of one plural-looking select. A select whose arms are all numbers or
    /// `other` (such as `{ $others -> *[other] ... }`) is not a plural select.
    fn collect(pattern: &ast::Pattern<&str>, out: &mut Vec<(String, BTreeSet<String>)>, at: &str) {
        for element in &pattern.elements {
            if let ast::PatternElement::Placeable { expression } = element {
                walk(expression, out, at);
            }
        }
    }

    fn walk(
        expression: &ast::Expression<&str>,
        out: &mut Vec<(String, BTreeSet<String>)>,
        at: &str,
    ) {
        match expression {
            ast::Expression::Inline(ast::InlineExpression::Placeable { expression }) => {
                walk(expression, out, at);
            }
            ast::Expression::Inline(_) => {}
            ast::Expression::Select { variants, .. } => {
                let names: BTreeSet<String> = variants
                    .iter()
                    .filter_map(|variant| match &variant.key {
                        ast::VariantKey::Identifier { name } => Some((*name).to_owned()),
                        ast::VariantKey::NumberLiteral { .. } => None,
                    })
                    .collect();
                if names
                    .iter()
                    .any(|name| ["zero", "one", "two", "few", "many"].contains(&name.as_str()))
                {
                    out.push((at.to_owned(), names));
                }
                for variant in variants {
                    collect(&variant.value, out, at);
                }
            }
        }
    }

    fn selects(source: &str, code: &str) -> Vec<(String, BTreeSet<String>)> {
        let resource = parser::parse(source)
            .unwrap_or_else(|(_, errors)| panic!("{code} does not parse: {errors:?}"));
        let mut out = Vec::new();
        for entry in &resource.body {
            let (id, value) = match entry {
                ast::Entry::Message(message) => (message.id.name, message.value.as_ref()),
                ast::Entry::Term(term) => (term.id.name, Some(&term.value)),
                _ => continue,
            };
            if let Some(pattern) = value {
                collect(pattern, &mut out, id);
            }
        }
        out
    }

    /// Japanese messages that still carry a `[one]` arm the language never selects (it has only
    /// `other`). They read the same as the `other` arm, so nothing renders wrongly; delete the arm
    /// and its entry here together.
    const DEAD_JAPANESE_ARMS: [&str; 9] = [
        "notif_verb_like",
        "notif_verb_repost",
        "notif_verb_follow",
        "notif_verb_follow_request",
        "notif_verb_reply",
        "notif_verb_mention",
        "notif_verb_quote",
        "notif_verb_poll",
        "notif_verb_affiliation",
    ];

    #[test]
    fn plural_selects_use_only_categories_the_language_has() {
        let mut dead_japanese = Vec::new();
        for (code, source) in FILES {
            let has = cldr_categories(code);
            for (id, names) in selects(source, code) {
                let unknown: Vec<_> = names
                    .iter()
                    .filter(|name| !has.contains(name.as_str()) && *name != "other")
                    .collect();
                if code == "ja" && DEAD_JAPANESE_ARMS.contains(&id.as_str()) {
                    dead_japanese.push(id);
                    continue;
                }
                assert!(
                    unknown.is_empty(),
                    "{code}: {id} uses {unknown:?}, CLDR has {has:?}"
                );
            }
        }
        dead_japanese.sort();
        let mut listed = DEAD_JAPANESE_ARMS.map(str::to_owned).to_vec();
        listed.sort();
        assert_eq!(dead_japanese, listed, "stale entry in DEAD_JAPANESE_ARMS");
    }

    /// `one` and `other` where the language has them, and `few` and `many` for the languages that
    /// split the rest by the last digits (Polish, Ukrainian, Czech): the same rule as the
    /// `covers the CLDR plural categories` test.
    #[test]
    fn plural_selects_cover_the_categories_the_language_needs() {
        for (code, source) in FILES {
            let has = cldr_categories(code);
            let mut need: BTreeSet<&str> = BTreeSet::from(["other"]);
            if has.contains("one") {
                need.insert("one");
            }
            if has.contains("few") {
                need.extend(["few", "many"]);
            }
            for (id, names) in selects(source, code) {
                let missing: Vec<_> = need
                    .iter()
                    .filter(|name| **name != "other" && !names.contains(**name))
                    .collect();
                assert!(missing.is_empty(), "{code}: {id} lacks {missing:?}");
            }
        }
    }

    #[test]
    fn the_shipped_languages_have_the_categories_the_rules_above_assume() {
        let set = |code| cldr_categories(code).into_iter().collect::<Vec<_>>();
        assert_eq!(set("en"), ["one", "other"]);
        assert_eq!(set("ja"), ["other"]);
        for code in ["pl", "uk", "cs"] {
            assert!(cldr_categories(code).contains("few"), "{code}");
        }
    }
}

#[cfg(test)]
mod rendered {
    use topcoat::l10n::locale;

    use super::{strip_isolates, t};

    fn verb(code: &str, count: i32) -> String {
        strip_isolates(&t!(notif_verb_like, count).resolve_in(&crate::locale::locale_of(code)))
    }

    fn posts(code: &str, count: i32, shown: &str) -> String {
        strip_isolates(
            &t!(admin_user_posts, count, compact: shown)
                .resolve_in(&crate::locale::locale_of(code)),
        )
    }

    #[test]
    fn counted_posts_follow_polish_and_ukrainian_rules() {
        assert_eq!(
            [(1, "1"), (2, "2"), (5, "5"), (22, "22")].map(|(n, s)| posts("pl", n, s)),
            ["1 wpis", "2 wpisy", "5 wpisów", "22 wpisy"]
        );
        assert_eq!(
            [(1, "1"), (3, "3"), (11, "11")].map(|(n, s)| posts("uk", n, s)),
            ["1 допис", "3 дописи", "11 дописів"]
        );
    }

    #[test]
    fn counted_posts_follow_czech_rules() {
        assert_eq!(
            [(1, "1"), (3, "3"), (5, "5")].map(|(n, s)| posts("cs", n, s)),
            ["1 příspěvek", "3 příspěvky", "5 příspěvků"]
        );
        let fraction = strip_isolates(
            &t!(admin_user_posts, count: 1.5, compact: "1,5").resolve_in(&locale!("cs")),
        );
        assert_eq!(fraction, "1,5 příspěvku");
    }

    #[test]
    fn english_and_japanese_posts() {
        assert_eq!(posts("en", 1, "1"), "1 post");
        assert_eq!(posts("en", 1200, "1.2K"), "1.2K posts");
        assert_eq!(posts("ja", 1, "1"), "1件のポスト");
        assert_eq!(posts("ja", 2, "2"), "2件のポスト");
    }

    #[test]
    fn a_notification_verb_agrees_with_one_or_several_people() {
        assert_eq!(
            [1, 2].map(|n| verb("pl", n)),
            ["polubił(a) Twój wpis", "polubili Twój wpis"]
        );
        assert_eq!(
            [1, 2].map(|n| verb("uk", n)),
            ["вподобав(-ла) ваш допис", "вподобали ваш допис"]
        );
        assert_eq!(
            [1, 2].map(|n| verb("fr", n)),
            ["a aimé votre post", "ont aimé votre post"]
        );
        assert_eq!(
            [1, 2].map(|n| verb("cs", n)),
            ["dává lajk vašemu příspěvku", "dávají lajk vašemu příspěvku"]
        );
        assert_eq!(
            [1, 2].map(|n| verb("en", n)),
            ["liked your post", "liked your post"]
        );
        assert_eq!(
            [1, 2].map(|n| verb("ja", n)),
            [
                "があなたのポストにいいねしました",
                "があなたのポストにいいねしました"
            ]
        );
    }

    #[test]
    fn the_known_followers_sentence_follows_the_total_and_the_remainder() {
        let followed = |count: i32, others: i32| {
            strip_isolates(
                &t!(profile_followed_by, count, others, a: "A", b: "B").resolve_in(&locale!("en")),
            )
        };
        assert_eq!(followed(1, -1), "Followed by A");
        assert_eq!(followed(2, 0), "Followed by A and B");
        assert_eq!(followed(3, 1), "Followed by A, B and 1 other you follow");
        assert_eq!(followed(9, 7), "Followed by A, B and 7 others you follow");
    }

    #[test]
    fn the_known_followers_sentence_exists_in_polish_and_ukrainian() {
        for code in ["pl", "uk", "ja"] {
            let locale = crate::locale::locale_of(code);
            let text = strip_isolates(
                &t!(profile_followed_by, count: 3, others: 1, a: "A", b: "B").resolve_in(&locale),
            );
            assert!(text.contains('A') && text.contains('B'), "{code}: {text}");
            assert!(!text.contains("profile_followed_by"), "{code}: {text}");
        }
    }
}

#[cfg(test)]
mod island_strings {
    use super::{ISOLATES, client_strings_json};
    use crate::locale::locale_of;

    const WITH_HANDLE: [&str; 9] = [
        "notify_like",
        "notify_repost",
        "notify_reply",
        "notify_mention",
        "notify_quote",
        "notify_follow",
        "notify_follow_request",
        "notify_affiliation",
        "notify_message",
    ];
    const PLAIN: [&str; 6] = [
        "notify_poll",
        "view",
        "settings_desktop_allowed",
        "settings_desktop_blocked",
        "settings_desktop_not_asked",
        "settings_desktop_blocked_toast",
    ];

    /// `stream.js` and the settings island read these keys through `text(entry, vars, lang)` in
    /// `time.js`: a string with `{name}` holes, or a map of sample counts to such strings.
    #[test]
    fn every_language_carries_the_strings_the_stream_and_settings_islands_read() {
        for code in tuita_core::locale::LANGS {
            let json: serde_json::Value =
                serde_json::from_str(&client_strings_json(&locale_of(code))).expect("valid json");
            for key in WITH_HANDLE {
                let text = json[key].as_str().unwrap_or_else(|| panic!("{code} {key}"));
                assert!(text.contains("{handle}"), "{code} {key}: {text}");
            }
            for key in PLAIN {
                let text = json[key].as_str().unwrap_or_else(|| panic!("{code} {key}"));
                assert!(
                    !text.is_empty() && !text.contains('{') && !text.contains(ISOLATES),
                    "{code} {key}: {text}"
                );
            }
            let state = json["settings_desktop_state"].as_str().expect("plain");
            assert!(state.contains("{state}"), "{code}: {state}");
            let unread = json["badge_unread"].as_object().expect("plural");
            assert_eq!(unread.len(), 5, "{code}");
            for (sample, text) in unread {
                let text = text.as_str().expect("text");
                assert!(text.contains("{count}"), "{code} {sample}: {text}");
                assert!(!text.contains(ISOLATES), "{code} {sample}: {text}");
            }
        }
    }

    #[test]
    fn english_strings_read_as_the_page_shows_them() {
        let json: serde_json::Value =
            serde_json::from_str(&client_strings_json(&locale_of("en"))).expect("valid json");
        assert_eq!(json["notify_like"], "@{handle} liked your post");
        assert_eq!(json["view"], "View");
        assert_eq!(json["settings_desktop_state"], "Permission: {state}");
        assert_eq!(json["badge_unread"]["1"], "{count} unread");
        assert_eq!(json["badge_unread"]["1000000"], "{count} unread");
    }
}
