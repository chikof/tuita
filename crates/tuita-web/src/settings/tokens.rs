//! API token scopes.

use topcoat::context::Cx;
use tuita_server::dto::TokenScope;

use crate::l10n::t;

/// Every scope, in the fixed order the list shows them.
pub(crate) const SCOPES: [TokenScope; 3] =
    [TokenScope::Read, TokenScope::Write, TokenScope::Messages];

/// What a new token may do unless the owner changes it.
pub(crate) const DEFAULT_SCOPES: [TokenScope; 2] = [TokenScope::Read, TokenScope::Write];

/// The name of a scope.
pub(crate) fn scope_label(cx: &Cx, scope: TokenScope) -> String {
    match scope {
        TokenScope::Read => t!(token_scope_read),
        TokenScope::Write => t!(token_scope_write),
        TokenScope::Messages => t!(token_scope_messages),
    }
    .resolve(cx)
    .into_owned()
}

/// The sentence that says what a scope allows.
pub(crate) fn scope_hint(cx: &Cx, scope: TokenScope) -> String {
    match scope {
        TokenScope::Read => t!(token_scope_read_hint),
        TokenScope::Write => t!(token_scope_write_hint),
        TokenScope::Messages => t!(token_scope_messages_hint),
    }
    .resolve(cx)
    .into_owned()
}

/// `scopes` once each, in the order of [`SCOPES`], whatever order they came in.
#[must_use]
pub(crate) fn order_scopes(scopes: &[TokenScope]) -> Vec<TokenScope> {
    SCOPES
        .into_iter()
        .filter(|scope| scopes.contains(scope))
        .collect()
}

/// `Read, Post and interact`: the labels of the scopes, ready for a list row.
pub(crate) fn scope_list(cx: &Cx, scopes: &[TokenScope]) -> String {
    order_scopes(scopes)
        .into_iter()
        .map(|scope| scope_label(cx, scope))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::testing::context;

    #[test]
    fn scopes_keep_the_fixed_order_whatever_order_they_came_in() {
        assert_eq!(
            order_scopes(&[TokenScope::Messages, TokenScope::Read]),
            [TokenScope::Read, TokenScope::Messages]
        );
        assert_eq!(
            order_scopes(&[TokenScope::Write, TokenScope::Read, TokenScope::Write]),
            [TokenScope::Read, TokenScope::Write]
        );
    }

    #[test]
    fn scopes_read_as_a_comma_separated_list_of_labels() {
        let cx = context("en");
        assert_eq!(
            scope_list(&cx, &[TokenScope::Write, TokenScope::Read]),
            "Read, Post and interact"
        );
        assert_eq!(scope_list(&cx, &[]), "");
    }
}
