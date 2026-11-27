//! The `?` dialog that lists the keyboard shortcuts.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use crate::{l10n::t, ui::dialog::dialog};

/// The id of the dialog: `shortcuts.js` opens it on `?`.
pub(crate) const DIALOG: &str = "shortcuts";

/// How the keys of one row read together.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Joiner {
    /// `j / k`
    Or,
    /// `g h`
    Then,
    /// `Ctrl + Enter`
    Plus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Label {
    NextPrev,
    Open,
    Search,
    GoHome,
    GoExplore,
    GoNotifications,
    GoMessages,
    GoBookmarks,
    GoLists,
    GoProfile,
    GoSettings,
    NewPost,
    Like,
    Reply,
    Repost,
    Bookmark,
    Send,
    Help,
}

impl Label {
    fn text(self, cx: &Cx) -> String {
        match self {
            Self::NextPrev => t!(shortcut_next_prev).resolve(cx).into_owned(),
            Self::Open => t!(shortcut_open).resolve(cx).into_owned(),
            Self::Search => t!(shortcut_search).resolve(cx).into_owned(),
            Self::GoHome => t!(shortcut_go_home).resolve(cx).into_owned(),
            Self::GoExplore => t!(shortcut_go_explore).resolve(cx).into_owned(),
            Self::GoNotifications => t!(shortcut_go_notifications).resolve(cx).into_owned(),
            Self::GoMessages => t!(shortcut_go_messages).resolve(cx).into_owned(),
            Self::GoBookmarks => t!(shortcut_go_bookmarks).resolve(cx).into_owned(),
            Self::GoLists => t!(shortcut_go_lists).resolve(cx).into_owned(),
            Self::GoProfile => t!(shortcut_go_profile).resolve(cx).into_owned(),
            Self::GoSettings => t!(shortcut_go_settings).resolve(cx).into_owned(),
            Self::NewPost => t!(shortcut_new_post).resolve(cx).into_owned(),
            Self::Like => t!(shortcut_like).resolve(cx).into_owned(),
            Self::Reply => t!(shortcut_reply).resolve(cx).into_owned(),
            Self::Repost => t!(shortcut_repost).resolve(cx).into_owned(),
            Self::Bookmark => t!(shortcut_bookmark).resolve(cx).into_owned(),
            Self::Send => t!(shortcut_send).resolve(cx).into_owned(),
            Self::Help => t!(shortcut_help).resolve(cx).into_owned(),
        }
    }
}

struct Item {
    keys: &'static [&'static str],
    joiner: Joiner,
    label: Label,
}

const fn item(keys: &'static [&'static str], joiner: Joiner, label: Label) -> Item {
    Item {
        keys,
        joiner,
        label,
    }
}

const NAVIGATION: [Item; 11] = [
    item(&["j", "k"], Joiner::Or, Label::NextPrev),
    item(&["o", "Enter"], Joiner::Or, Label::Open),
    item(&["/"], Joiner::Then, Label::Search),
    item(&["g", "h"], Joiner::Then, Label::GoHome),
    item(&["g", "e"], Joiner::Then, Label::GoExplore),
    item(&["g", "n"], Joiner::Then, Label::GoNotifications),
    item(&["g", "m"], Joiner::Then, Label::GoMessages),
    item(&["g", "b"], Joiner::Then, Label::GoBookmarks),
    item(&["g", "l"], Joiner::Then, Label::GoLists),
    item(&["g", "p"], Joiner::Then, Label::GoProfile),
    item(&["g", "s"], Joiner::Then, Label::GoSettings),
];

const ACTIONS: [Item; 7] = [
    item(&["n"], Joiner::Then, Label::NewPost),
    item(&["l"], Joiner::Then, Label::Like),
    item(&["r"], Joiner::Then, Label::Reply),
    item(&["t"], Joiner::Then, Label::Repost),
    item(&["b"], Joiner::Then, Label::Bookmark),
    item(&["Ctrl", "Enter"], Joiner::Plus, Label::Send),
    item(&["?"], Joiner::Then, Label::Help),
];

/// One table row as the view reads it: the keys with the joining word, and the label.
pub(crate) struct Row {
    pub(crate) keys: Vec<&'static str>,
    pub(crate) join: String,
    pub(crate) label: String,
}

fn rows(cx: &Cx, items: &[Item]) -> Vec<Row> {
    items
        .iter()
        .map(|item| Row {
            keys: item.keys.to_vec(),
            join: match item.joiner {
                Joiner::Or => t!(shortcut_or).resolve(cx).into_owned(),
                Joiner::Then => t!(shortcut_then).resolve(cx).into_owned(),
                Joiner::Plus => t!(shortcut_plus).resolve(cx).into_owned(),
            },
            label: item.label.text(cx),
        })
        .collect()
}

/// The two groups of shortcuts (navigation, actions) with their localized names.
pub(crate) fn groups(cx: &Cx) -> [(String, Vec<Row>); 2] {
    [
        (
            t!(shortcut_group_navigation).resolve(cx).into_owned(),
            rows(cx, &NAVIGATION),
        ),
        (
            t!(shortcut_group_actions).resolve(cx).into_owned(),
            rows(cx, &ACTIONS),
        ),
    ]
}

/// The shortcuts dialog, closed; any button with `command="show-modal" commandfor="shortcuts"`
/// opens it.
#[component]
pub(crate) async fn shortcuts_dialog(cx: &Cx) -> Result<impl View> {
    let title = t!(shortcuts_title).resolve(cx).into_owned();
    let groups = groups(cx);
    Ok(view! {
        dialog(
            id: DIALOG,
            title: &title,
            <div data-c="shortcuts-dialog">
                for (name, rows) in groups {
                    <section>
                        <h3>(name)</h3>
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
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn the_dialog_lists_every_shortcut_under_its_group() {
        let html = render("en", |cx| view! { shortcuts_dialog() }.boxed());
        assert!(html.contains(r#"<dialog id="shortcuts""#), "{html}");
        assert!(html.contains("<h3>Navigation</h3>"), "{html}");
        assert!(html.contains("<h3>Actions</h3>"), "{html}");
        assert!(html.contains("<kbd>Ctrl</kbd>"), "{html}");
        assert!(html.contains("<dd>Next / previous post</dd>"), "{html}");
        assert_eq!(html.matches("<kbd>").count(), 29, "{html}");
    }

    #[test]
    fn keys_read_with_the_languages_joining_word() {
        let html = render("pl", |cx| view! { shortcuts_dialog() }.boxed());
        assert!(html.contains(r#"<span class="join">"#), "{html}");
        assert!(html.contains("<kbd>j</kbd>"), "{html}");
    }
}
