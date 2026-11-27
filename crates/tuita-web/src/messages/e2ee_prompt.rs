//! The passphrase form of encrypted messages.

use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, class, component, element_id, view},
};

use crate::{
    icons::Icon,
    l10n::t,
    ui::{
        button::{ButtonVariant, button},
        icon::icon,
        text_field::text_field,
    },
};

/// What a prompt asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PromptKind {
    /// Makes a key and asks the passphrase twice.
    Setup,
    /// Asks the passphrase once.
    Unlock,
    /// `Setup` for a replacement; its explanation is left to the dialog around it.
    Reset,
}

impl PromptKind {
    const fn name(self) -> &'static str {
        match self {
            Self::Setup => "setup",
            Self::Unlock => "unlock",
            Self::Reset => "reset",
        }
    }

    /// The client string `e2ee.js` toasts when the form succeeds.
    const fn done(self) -> &'static str {
        match self {
            Self::Setup => "e2ee_setup_done",
            Self::Unlock => "e2ee_unlock_done",
            Self::Reset => "e2ee_reset_done",
        }
    }
}

/// A passphrase form. `bare` drops the card frame and heading (the section or dialog around it
/// supplies them); `toast` shows the success toast of the kind (the settings page, not the chat).
///
/// `e2ee.js` handles the submit of `[data-prompt] form`: it checks the passphrase, calls the keys
/// and shows failures under the field.
#[component]
pub(crate) async fn e2ee_prompt(
    cx: &Cx,
    kind: PromptKind,
    #[default] bare: bool,
    #[default] toast: bool,
) -> Result<impl View> {
    let id = element_id(cx);
    let unlock = kind == PromptKind::Unlock;
    let passphrase = t!(e2ee_passphrase).resolve(cx).into_owned();
    let confirm = t!(e2ee_passphrase_confirm).resolve(cx).into_owned();
    let hint = t!(e2ee_passphrase_hint).resolve(cx).into_owned();
    let autocomplete = if unlock {
        "current-password"
    } else {
        "new-password"
    };
    let done = toast.then(|| kind.done());
    let variant = if kind == PromptKind::Reset {
        ButtonVariant::Danger
    } else {
        ButtonVariant::Solid
    };
    Ok(view! {
        <section
            class=(class!("prompt", "bare" if bare))
            aria-labelledby=((!bare).then(|| format!("{id}-title")))
            data-prompt=(kind.name())
            data-done=(done)
            data-c="e2ee-prompt"
        >
            if !bare {
                <header>
                    <span class="art" aria-hidden="true">
                        icon(name: Icon::Lock, size: 24)
                    </span>
                    <h2 id=(format!("{id}-title"))>
                        if unlock {
                            (t!(e2ee_unlock_title))
                        } else {
                            (t!(e2ee_setup_title))
                        }
                    </h2>
                </header>
            }
            if kind != PromptKind::Reset {
                <p class="body">
                    if unlock {
                        (t!(e2ee_unlock_body))
                    } else {
                        (t!(e2ee_setup_body))
                    }
                </p>
            }
            if kind == PromptKind::Setup {
                <p class="warn">(t!(e2ee_setup_warning))</p>
            }
            <form novalidate="">
                if unlock {
                    text_field(
                        label: &passphrase,
                        attrs: attributes! {
                            type="password"
                            name="passphrase"
                            autocomplete=(autocomplete)
                        }
                    )
                } else {
                    text_field(
                        label: &passphrase,
                        hint: hint.as_str(),
                        attrs: attributes! {
                            type="password"
                            name="passphrase"
                            autocomplete=(autocomplete)
                        }
                    )
                    text_field(
                        label: &confirm,
                        attrs: attributes! {
                            type="password"
                            name="passphrase-again"
                            autocomplete="new-password"
                        }
                    )
                }
                <div class="end">
                    button(
                        submit: true,
                        variant: variant,
                        if unlock {
                            (t!(e2ee_unlock_submit))
                        } else if kind == PromptKind::Reset {
                            (t!(e2ee_reset_submit))
                        } else {
                            (t!(e2ee_setup_submit))
                        }
                    )
                </div>
            </form>
        </section>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn setup_asks_twice_and_unlock_once() {
        let setup = render("en", |cx| {
            view! { e2ee_prompt(kind: PromptKind::Setup) }.boxed()
        });
        assert!(setup.contains(r#"data-prompt="setup""#), "{setup}");
        assert!(setup.contains(">Passphrase</label>"), "{setup}");
        assert!(setup.contains(">Confirm passphrase</label>"), "{setup}");
        assert!(setup.contains("At least 10 characters."), "{setup}");
        assert!(setup.contains("<h2"), "{setup}");
        assert!(setup.contains(">Turn on</span>"), "{setup}");
        let unlock = render("en", |cx| {
            view! { e2ee_prompt(kind: PromptKind::Unlock) }.boxed()
        });
        assert!(!unlock.contains("Confirm passphrase"), "{unlock}");
        assert!(unlock.contains("Unlock encrypted messages"), "{unlock}");
        assert!(
            unlock.contains(r#"autocomplete="current-password""#),
            "{unlock}"
        );
    }

    #[test]
    fn bare_drops_the_heading_and_a_reset_has_no_explanation() {
        let reset = render("en", |cx| {
            view! { e2ee_prompt(kind: PromptKind::Reset, bare: true, toast: true) }.boxed()
        });
        assert!(!reset.contains("<h2"), "{reset}");
        assert!(!reset.contains(r#"class="body""#), "{reset}");
        assert!(reset.contains(r#"data-done="e2ee_reset_done""#), "{reset}");
        assert!(reset.contains(">Reset</span>"), "{reset}");
        assert!(reset.contains("danger"), "{reset}");
    }
}
