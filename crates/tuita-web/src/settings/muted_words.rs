//! The muted words card.

use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, Signal, procedure, shard, signal},
    view::{View, attributes, component, view},
};

use crate::{
    icons::Icon,
    l10n::t,
    session,
    settings::{
        actions::{Done, patch_me},
        card::setting_card,
    },
    ui::{
        button::{ButtonVariant, button},
        icon::icon,
        text_field::text_field,
    },
};

/// The most words one account may mute.
pub(crate) const MAX_WORDS: usize = 100;

/// The muted words a change leaves: `word` added or removed (trimmed, lower case), or the list
/// unchanged when there is nothing to do. `None` when the list is full.
pub(crate) fn changed(words: &[String], word: &str, add: bool) -> Option<Vec<String>> {
    let word = word.trim().to_lowercase();
    let mut next = words.to_vec();
    if add {
        if word.is_empty() || next.contains(&word) {
            return Some(next);
        }
        if next.len() >= MAX_WORDS {
            return None;
        }
        next.push(word);
    } else {
        next.retain(|w| *w != word);
    }
    Some(next)
}

/// Adds or removes `word` on the account and answers the new list.
async fn change(cx: &Cx, word: &str, add: bool) -> Done<Vec<String>> {
    let Some(me) = session::me(cx).await? else {
        return Ok(Err("Not signed in".to_owned()));
    };
    let Some(next) = changed(&me.settings.muted_words, word, add) else {
        let limit = t!(settings_words_limit, count: 100)
            .resolve(cx)
            .into_owned();
        return Ok(Err(limit));
    };
    if next == me.settings.muted_words {
        return Ok(Ok(next));
    }
    Ok(patch_me(cx, &serde_json::json!({ "mutedWords": next }))
        .await
        .map(|me| me.settings.muted_words))
}

/// Mutes a word; answers the new list.
#[procedure]
pub(crate) async fn add_word(cx: &Cx, word: String) -> Done<Vec<String>> {
    change(cx, &word, true).await
}

/// Unmutes a word; answers the new list.
#[procedure]
pub(crate) async fn remove_word(cx: &Cx, word: String) -> Done<Vec<String>> {
    change(cx, &word, false).await
}

/// The chips of the muted words; reruns when `words` changes.
#[shard]
async fn word_chips(cx: &Cx, words: Signal<Vec<String>>) -> Result<impl View> {
    let list: Vec<(String, String)> = words
        .get()
        .into_iter()
        .map(|word| {
            let label = t!(settings_words_remove, word: word.as_str())
                .resolve(cx)
                .into_owned();
            (word, label)
        })
        .collect();
    Ok(view! {
        if list.is_empty() {
            <p class="none">(t!(settings_words_none))</p>
        } else {
            <ul>
                for (word, label) in list {
                    <li>
                        <button
                            type="button"
                            class="chip wash"
                            aria-label=(&label)
                            @click=$(async |_e: Event| {
                                let done = remove_word(word).await;
                                if done.is_ok() {
                                    words.set(done.unwrap());
                                    false
                                } else {
                                    let msg = done.unwrap_err();
                                    raw!(
                                        "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:${msg}}))",
                                        msg.is_empty(),
                                    )
                                }
                            })
                        >
                            <span class="word">(&word)</span>
                            icon(name: Icon::Close, size: 16)
                        </button>
                    </li>
                }
            </ul>
        }
    })
}

/// The add form and the chips of the account's muted words.
#[component]
pub(crate) async fn muted_words(cx: &Cx, words: Vec<String>) -> Result<impl View> {
    let list = signal(cx, move || words);
    let draft = signal(cx, String::new);
    let label = t!(settings_words_label).resolve(cx).into_owned();
    let placeholder = t!(settings_words_placeholder).resolve(cx).into_owned();
    Ok(view! {
        setting_card(
            title: t!(settings_words_title).resolve(cx).into_owned(),
            description: t!(settings_words_desc).resolve(cx).into_owned(),
            <div data-c="muted-words">
                <form
                    class="add"
                    @submit=$(async |e: Event| {
                        e.prevent_default();
                        let word = draft.get();
                        draft.set("".to_owned());
                        let done = add_word(word).await;
                        if done.is_ok() {
                            list.set(done.unwrap());
                            false
                        } else {
                            let msg = done.unwrap_err();
                            raw!(
                                "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:${msg}}))",
                                msg.is_empty(),
                            )
                        }
                    })
                >
                    text_field(
                        label: &label,
                        attrs: attributes! {
                            name="word"
                            autocomplete="off"
                            maxlength="40"
                            placeholder=(&placeholder)
                            :value=$(draft.get())
                            @input=$(|e: Event| draft.set(e.target.value))
                        }
                    )
                    button(
                        variant: ButtonVariant::Outline,
                        submit: true,
                        (t!(settings_words_add))
                    )
                </form>
                word_chips(words: list)
            </div>
        )
    })
}

#[cfg(test)]
mod test {
    use super::*;

    fn words(of: &[&str]) -> Vec<String> {
        of.iter().map(|w| (*w).to_owned()).collect()
    }

    #[test]
    fn a_new_word_is_trimmed_lowered_and_appended() {
        assert_eq!(
            changed(&words(&["spam"]), "  Ham ", true),
            Some(words(&["spam", "ham"]))
        );
    }

    #[test]
    fn blank_and_repeated_words_change_nothing() {
        assert_eq!(
            changed(&words(&["spam"]), "  ", true),
            Some(words(&["spam"]))
        );
        assert_eq!(
            changed(&words(&["spam"]), "SPAM", true),
            Some(words(&["spam"]))
        );
    }

    #[test]
    fn a_full_list_refuses_one_more() {
        let full: Vec<String> = (0..MAX_WORDS).map(|n| format!("w{n}")).collect();
        assert_eq!(changed(&full, "extra", true), None);
        assert_eq!(changed(&full, "w1", true), Some(full.clone()));
    }

    #[test]
    fn removing_drops_only_that_word() {
        assert_eq!(
            changed(&words(&["a", "b"]), "a", false),
            Some(words(&["b"]))
        );
    }
}
