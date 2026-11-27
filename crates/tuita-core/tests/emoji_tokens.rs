//! The token rule, checked against the vectors the web client's tokenizer is tested with too.

use serde::Deserialize;
use tuita_core::emoji::shortcodes;

#[derive(Deserialize)]
struct Case {
    input: String,
    shortcodes: Vec<String>,
}

#[test]
fn the_shared_vectors_hold() {
    let cases: Vec<Case> =
        serde_json::from_str(include_str!("fixtures/emoji_tokens.json")).expect("fixture parses");
    assert!(cases.len() > 10, "the fixture lost its cases");
    for case in cases {
        assert_eq!(
            shortcodes(&case.input),
            case.shortcodes,
            "input {:?}",
            case.input
        );
    }
}
