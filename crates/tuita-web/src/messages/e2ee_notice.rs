//! A card with a stripe and a lock or warning icon.

use topcoat::{
    Result,
    context::Cx,
    view::{Attributes, Child, View, component, view},
};

use crate::{icons::Icon, ui::icon::icon};

/// How loud a notice is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Tone {
    #[default]
    Info,
    Warning,
}

impl Tone {
    const fn class(self) -> &'static str {
        match self {
            Self::Info => "notice info",
            Self::Warning => "notice warning",
        }
    }
}

/// A status (info) or alert (warning) with an optional `title` and an `actions` row. `attrs` go
/// on the root, for an island's hooks.
#[component]
pub(crate) async fn e2ee_notice(
    cx: &Cx,
    #[default] tone: Tone,
    #[default]
    #[into]
    title: Option<&str>,
    #[default] actions: Option<Child<'_>>,
    #[default] attrs: Attributes,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    let (role, glyph) = match tone {
        Tone::Info => ("status", Icon::Lock),
        Tone::Warning => ("alert", Icon::WarningDiamond),
    };
    Ok(view! {
        <div class=(tone.class()) role=(role) data-c="e2ee-notice" (attrs)>
            <i class="stripe" aria-hidden="true"></i>
            <span class="icon">icon(name: glyph, size: 20)</span>
            <div class="text">
                if let Some(title) = title {
                    <h3>(title)</h3>
                }
                <div class="body">(child)</div>
                if let Some(actions) = actions {
                    <div class="actions">(actions)</div>
                }
            </div>
        </div>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn a_warning_is_an_alert_and_an_info_a_status() {
        let warning = render("en", |cx| {
            view! {
                e2ee_notice(tone: Tone::Warning, title: "Key changed", "Compare it.")
            }
            .boxed()
        });
        assert!(warning.contains(r#"role="alert""#), "{warning}");
        assert!(warning.contains("<h3>Key changed</h3>"), "{warning}");
        assert!(warning.contains("notice warning"), "{warning}");
        let info = render("en", |cx| view! { e2ee_notice("Sealed.") }.boxed());
        assert!(info.contains(r#"role="status""#), "{info}");
        assert!(!info.contains("<h3>"), "{info}");
    }
}
