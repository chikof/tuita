//! Payloads shaped like what Mastodon, Misskey and GoToSocial send, stored in
//! `tests/fixtures/ap/*.json`, read through the `tuita_core::ap` types.
//!
//! Every fixture is named in this file (`fixtures_dir_is_fully_covered` fails otherwise), so a
//! new payload cannot be added without an assertion about what it parses to.
#![allow(missing_docs)]

use std::path::PathBuf;

use serde_json::{Value, json};
use time::Duration;
use tuita_core::ap::{
    self, Activity, ActivityKind, Actor, ActorKind, Collection, IdOrObject, MAX_DEPTH, MAX_NODES,
    Note, NoteKind, ParseError, PublicKey, RequestParts, SignatureError, Tag, Tombstone, id_of,
};

const BEA: &str = "https://social.example/users/bea";
const BEA_POST: &str =
    "https://social.example/users/bea/statuses/0192a4b6-7c3e-7f10-9a42-3d5c8e1f2a07";
const ADA_FOLLOW_OF_BEA: &str =
    "https://social.example/activities/0192a4b6-7c3e-7f10-9a42-3d5c8e1f2a08";

fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/ap")
}

fn bytes(name: &str) -> Vec<u8> {
    std::fs::read(dir().join(format!("{name}.json"))).expect("fixture file")
}

/// The fixture as `parse_document` accepts a received body, with the production byte limit.
fn doc(name: &str) -> Value {
    ap::parse_document(&bytes(name), 1 << 20).expect("fixture passes the document guards")
}

fn actor(name: &str) -> Actor {
    serde_json::from_value(doc(name)).expect("actor")
}

fn note(name: &str) -> Note {
    serde_json::from_value(doc(name)).expect("note")
}

fn activity(name: &str) -> Activity {
    serde_json::from_value(doc(name)).expect("activity")
}

fn collection(name: &str) -> Collection {
    serde_json::from_value(doc(name)).expect("collection")
}

fn embedded_activity(parent: &Activity) -> Activity {
    serde_json::from_value(parent.object.clone()).expect("embedded activity")
}

fn embedded_note(parent: &Activity) -> Note {
    serde_json::from_value(parent.object.clone()).expect("embedded note")
}

fn hashtag(name: &str, href: &str) -> Tag {
    Tag::Hashtag {
        name: name.to_owned(),
        href: Some(href.to_owned()),
    }
}

fn mention(href: &str, name: &str) -> Tag {
    Tag::Mention {
        href: href.to_owned(),
        name: Some(name.to_owned()),
    }
}

fn emoji(id: &str, name: &str, icon: &str) -> Tag {
    Tag::Emoji {
        id: id.to_owned(),
        name: name.to_owned(),
        icon: icon.to_owned(),
        media_type: Some("image/png".to_owned()),
    }
}

// ------------------------------------------------------------------ the fixture directory

#[test]
fn fixtures_dir_is_fully_covered() {
    let source = include_str!("ap_fixtures.rs");
    let mut unused = Vec::new();
    let mut count = 0;
    for entry in std::fs::read_dir(dir()).expect("fixtures dir") {
        let path = entry.expect("entry").path();
        let name = path
            .file_name()
            .expect("name")
            .to_string_lossy()
            .into_owned();
        let stem = name
            .strip_suffix(".json")
            .unwrap_or_else(|| panic!("{name} is not a .json file"));
        count += 1;
        let (software, rest) = stem.split_once('-').expect("<software>-<kind> name");
        assert!(
            matches!(software, "mastodon" | "misskey" | "gts" | "generic") && !rest.is_empty(),
            "{name} is not named <software>-<kind>[-<variant>]"
        );
        if !source.contains(&format!("\"{stem}\"")) {
            unused.push(name);
        }
    }
    assert!(unused.is_empty(), "fixtures no test refers to: {unused:?}");
    assert!(count >= 30, "only {count} fixtures");
}

#[test]
fn every_fixture_passes_the_document_guards_and_survives_every_reader() {
    for entry in std::fs::read_dir(dir()).expect("fixtures dir") {
        let path = entry.expect("entry").path();
        let body = std::fs::read(&path).expect("read");
        let name = path.display().to_string();
        let value =
            ap::parse_document(&body, 1 << 20).unwrap_or_else(|err| panic!("{name}: {err}"));
        assert_eq!(
            ap::parse_document(&body, body.len().saturating_sub(1)),
            Err(ParseError::TooLarge),
            "{name}"
        );
        let again = ap::parse_document(value.to_string().as_bytes(), 1 << 20).expect("round trip");
        assert_eq!(again, value, "{name}: serde_json round trip");
        // Whatever the shape, the readers return a value or an error and never panic.
        let _ = serde_json::from_value::<Actor>(value.clone());
        let _ = serde_json::from_value::<Note>(value.clone());
        let _ = serde_json::from_value::<Activity>(value.clone());
        let _ = serde_json::from_value::<Collection>(value.clone());
        let _ = serde_json::from_value::<Tombstone>(value.clone());
        let _ = id_of(&value);
    }
}

// ------------------------------------------------------------------ @context

#[test]
fn context_comes_as_a_string_an_array_or_a_mixed_array_and_never_matters() {
    let shape = |name: &str| match &doc(name)["@context"] {
        Value::String(_) => "string",
        Value::Array(items) if items.iter().all(Value::is_string) => "array",
        Value::Array(_) => "mixed",
        Value::Null => "none",
        other => panic!("{name}: odd context {other}"),
    };
    assert_eq!(shape("mastodon-activity-follow"), "string");
    assert_eq!(shape("gts-collection-outbox"), "string");
    assert_eq!(shape("gts-actor-service"), "mixed");
    assert_eq!(shape("mastodon-note-public"), "mixed");
    assert_eq!(shape("misskey-note-public"), "mixed");
    assert_eq!(shape("misskey-actor-person"), "mixed");
    assert_eq!(shape("gts-collection-outbox-page"), "none");
    let array_of_strings = json!([
        "https://www.w3.org/ns/activitystreams",
        "https://w3id.org/security/v1"
    ]);
    let mut value = doc("mastodon-activity-follow");
    value["@context"] = array_of_strings;
    assert_eq!(
        serde_json::from_value::<Activity>(value)
            .expect("activity")
            .kind,
        ActivityKind::Follow
    );
    // The same note with each context shape reads the same.
    let expected = note("mastodon-note-public");
    for context in [
        json!("https://www.w3.org/ns/activitystreams"),
        json!(["https://www.w3.org/ns/activitystreams"]),
        json!(["https://www.w3.org/ns/activitystreams", {"sensitive": "as:sensitive"}]),
        Value::Null,
    ] {
        let mut value = doc("mastodon-note-public");
        value["@context"] = context;
        assert_eq!(
            serde_json::from_value::<Note>(value).expect("note"),
            expected
        );
    }
}

// ------------------------------------------------------------------ actors

#[test]
fn mastodon_person() {
    let actor = actor("mastodon-actor-person");
    assert_eq!(actor.id, "https://mastodon.example/users/ada");
    assert_eq!(actor.kind, ActorKind::Person);
    assert_eq!(actor.preferred_username, "ada");
    assert_eq!(actor.name.as_deref(), Some("Ada Example :blobcat:"));
    assert_eq!(actor.url.as_deref(), Some("https://mastodon.example/@ada"));
    assert_eq!(actor.inbox, "https://mastodon.example/users/ada/inbox");
    assert_eq!(
        actor.outbox.as_deref(),
        Some("https://mastodon.example/users/ada/outbox")
    );
    assert_eq!(
        actor.followers.as_deref(),
        Some("https://mastodon.example/users/ada/followers")
    );
    assert_eq!(
        actor.following.as_deref(),
        Some("https://mastodon.example/users/ada/following")
    );
    assert_eq!(
        actor.featured.as_deref(),
        Some("https://mastodon.example/users/ada/collections/featured")
    );
    assert_eq!(
        actor
            .endpoints
            .as_ref()
            .and_then(|e| e.shared_inbox.as_deref()),
        Some("https://mastodon.example/inbox")
    );
    assert_eq!(actor.delivery_inbox(), "https://mastodon.example/inbox");
    assert!(!actor.manually_approves_followers);
    assert_eq!(actor.discoverable, Some(true));
    assert_eq!(actor.published.as_deref(), Some("2022-11-09T00:00:00Z"));
    assert_eq!(actor.moved_to, None);
    let key = actor.public_key.as_ref().expect("key");
    assert_eq!(key.id, "https://mastodon.example/users/ada#main-key");
    assert_eq!(key.owner, actor.id);
    PublicKey::from_pem(&key.public_key_pem).expect("the fixture key is a real RSA key");
    let icon = actor.icon.as_ref().expect("icon");
    assert_eq!(
        icon.url,
        "https://mastodon.example/system/accounts/avatars/000/000/001/original/ada.jpg"
    );
    assert_eq!(icon.media_type.as_deref(), Some("image/jpeg"));
    let image = actor.image.as_ref().expect("image");
    assert_eq!(
        image.url,
        "https://mastodon.example/system/accounts/headers/000/000/001/original/ada-banner.png"
    );
    let fields: Vec<(&str, &str)> = actor
        .attachment
        .iter()
        .map(|field| (field.name.as_str(), field.value.as_str()))
        .collect();
    assert_eq!(actor.attachment.len(), 3);
    assert_eq!(fields[1].1, "she/her");
    assert_eq!(fields[2], ("Location", "A wood"));
    assert_eq!(fields[0].0, "Website");
    assert_eq!(ap::html_to_text(fields[0].1), "https://ada.example/");
    assert_eq!(
        ap::html_to_text(actor.summary.as_deref().expect("summary")),
        "Writes Rust, walks in woods. #rustlang"
    );
}

#[test]
fn mastodon_locked_bot_with_an_icon_array_and_one_profile_field() {
    let actor = actor("mastodon-actor-locked-bot");
    assert_eq!(actor.kind, ActorKind::Service);
    assert!(actor.manually_approves_followers);
    assert_eq!(actor.discoverable, Some(false));
    assert_eq!(actor.featured, None);
    assert_eq!(
        actor.icon.as_ref().map(|icon| icon.url.as_str()),
        Some("https://mastodon.example/system/accounts/avatars/000/000/002/original/bot.webp"),
        "the first of an icon array wins"
    );
    assert_eq!(
        actor.icon.as_ref().and_then(|i| i.media_type.as_deref()),
        Some("image/webp")
    );
    assert_eq!(actor.image, None);
    assert_eq!(
        actor.attachment.len(),
        1,
        "a single attachment object is one field"
    );
    assert_eq!(actor.attachment.as_slice()[0].name, "Source");
    assert_eq!(actor.delivery_inbox(), "https://mastodon.example/inbox");
}

#[test]
fn misskey_person() {
    let value = doc("misskey-actor-person");
    assert_eq!(value["isCat"], true, "the fixture carries isCat");
    assert_eq!(
        value["_misskey_summary"],
        "Hello! I like #cats and https://home.example/ada"
    );
    let actor: Actor = serde_json::from_value(value).expect("actor");
    assert_eq!(actor.id, "https://misskey.example/users/9a0example01");
    assert_eq!(actor.kind, ActorKind::Person);
    assert_eq!(actor.preferred_username, "ada");
    assert_eq!(actor.name.as_deref(), Some("Ada :ablobcat:"));
    assert_eq!(actor.url.as_deref(), Some("https://misskey.example/@ada"));
    assert_eq!(
        actor.inbox,
        "https://misskey.example/users/9a0example01/inbox"
    );
    assert_eq!(
        actor.featured.as_deref(),
        Some("https://misskey.example/users/9a0example01/collections/featured")
    );
    assert_eq!(actor.delivery_inbox(), "https://misskey.example/inbox");
    assert_eq!(actor.discoverable, Some(true));
    assert!(!actor.manually_approves_followers);
    assert_eq!(actor.published, None, "Misskey sends no creation time");
    let key = actor.public_key.as_ref().expect("key");
    assert_eq!(
        key.id,
        "https://misskey.example/users/9a0example01#main-key"
    );
    assert_eq!(key.owner, actor.id);
    let icon = actor.icon.as_ref().expect("icon");
    assert_eq!(icon.url, "https://misskey.example/files/9a0avatar0001.webp");
    assert_eq!(icon.media_type, None);
    assert_eq!(
        actor.image.as_ref().map(|image| image.url.as_str()),
        Some("https://misskey.example/files/9a0banner0001.png")
    );
    assert_eq!(actor.attachment.len(), 2);
    assert_eq!(actor.attachment.as_slice()[1].value, "she/they");
    let bio = ap::html_to_text(actor.summary.as_deref().expect("summary"));
    assert!(bio.starts_with("Hello! I like #cats"), "{bio}");
    assert!(bio.ends_with("and https://home.example/ada"), "{bio}");
}

#[test]
fn gotosocial_person_with_a_single_emoji_tag_and_no_shared_inbox() {
    let actor = actor("gts-actor-person");
    assert_eq!(actor.id, "https://gts.example/users/ada");
    assert_eq!(actor.kind, ActorKind::Person);
    assert_eq!(actor.name.as_deref(), Some("ada :rainbow:"));
    assert_eq!(actor.url.as_deref(), Some("https://gts.example/@ada"));
    assert_eq!(actor.endpoints, None);
    assert_eq!(
        actor.delivery_inbox(),
        "https://gts.example/users/ada/inbox"
    );
    assert_eq!(actor.discoverable, Some(true));
    assert!(!actor.manually_approves_followers);
    assert_eq!(actor.published.as_deref(), Some("2022-09-29T13:57:24Z"));
    let names: Vec<&str> = actor.attachment.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names.len(), 3);
    assert_eq!(names[1], "website");
    assert_eq!(actor.attachment.as_slice()[0].value, "they/them");
    assert_eq!(
        ap::html_to_text(&actor.attachment.as_slice()[1].value),
        "https://ada.example/"
    );
    assert_eq!(
        actor.icon.as_ref().and_then(|i| i.media_type.as_deref()),
        Some("image/jpeg")
    );
    assert_eq!(
        actor.image.as_ref().and_then(|i| i.media_type.as_deref()),
        Some("image/png")
    );
    assert_eq!(
        actor.public_key.as_ref().map(|k| k.owner.as_str()),
        Some(actor.id.as_str())
    );
}

#[test]
fn gotosocial_service_with_no_attachment_key() {
    let value = doc("gts-actor-service");
    assert!(value.get("attachment").is_none());
    let actor: Actor = serde_json::from_value(value).expect("actor");
    assert_eq!(actor.id, "https://gts.example/users/1happyturtle");
    assert_eq!(actor.kind, ActorKind::Service);
    assert_eq!(actor.preferred_username, "1happyturtle");
    assert!(actor.manually_approves_followers);
    assert_eq!(actor.discoverable, Some(false));
    assert!(actor.attachment.is_empty());
    assert_eq!(actor.icon, None);
    assert_eq!(actor.image, None);
}

// ------------------------------------------------------------------ notes

#[test]
fn mastodon_public_note_with_attachments_tags_and_a_null_reply() {
    let value = doc("mastodon-note-public");
    assert!(value["inReplyTo"].is_null() && value["summary"].is_null());
    let note: Note = serde_json::from_value(value).expect("note");
    assert_eq!(
        note.id,
        "https://mastodon.example/users/ada/statuses/113200000000000001"
    );
    assert_eq!(note.kind, NoteKind::Note);
    assert_eq!(
        note.attributed_to.as_deref(),
        Some("https://mastodon.example/users/ada")
    );
    assert_eq!(note.in_reply_to, None);
    assert_eq!(note.summary, None);
    assert!(!note.sensitive);
    assert_eq!(note.published.as_deref(), Some("2026-11-08T09:30:12Z"));
    assert_eq!(note.updated, None);
    assert_eq!(
        note.url.as_deref(),
        Some("https://mastodon.example/@ada/113200000000000001")
    );
    assert_eq!(note.to.as_slice(), [ap::PUBLIC]);
    assert_eq!(
        note.cc.as_slice(),
        ["https://mastodon.example/users/ada/followers", BEA]
    );
    assert_eq!(note.attachment.len(), 2);
    let first = &note.attachment.as_slice()[0];
    assert_eq!(first.kind, "Document");
    assert_eq!(first.media_type.as_deref(), Some("image/jpeg"));
    assert_eq!(
        first.url.as_deref(),
        Some(
            "https://mastodon.example/system/media_attachments/files/113/200/000/000/000/001/original/walk1.jpg"
        )
    );
    assert_eq!(
        first.name.as_deref(),
        Some("A path through a wood in autumn")
    );
    assert_eq!((first.width, first.height), (Some(1600), Some(1200)));
    let second = &note.attachment.as_slice()[1];
    assert_eq!(second.media_type.as_deref(), Some("image/png"));
    assert_eq!(second.name, None, "a null name is no alt text");
    assert_eq!(
        note.tag.as_slice(),
        [
            mention(BEA, "@bea@social.example"),
            hashtag("#rustlang", "https://mastodon.example/tags/rustlang"),
            hashtag("#Autumn", "https://mastodon.example/tags/autumn"),
            emoji(
                "https://mastodon.example/emojis/17",
                "blobcat",
                "https://mastodon.example/system/custom_emojis/images/000/000/017/original/blobcat.png"
            ),
        ]
    );
    assert_eq!(note.quoted(), None);
    assert!(note.one_of.is_empty() && note.any_of.is_empty());
    assert_eq!(
        ap::html_to_text(note.content.as_deref().expect("content")),
        "Good morning @bea@social.example! Photos from the walk #rustlang #Autumn :blobcat:\n\nSecond paragraph & a line\nbreak."
    );
}

#[test]
fn mastodon_note_with_a_content_warning_an_edit_time_and_audience_swapped_for_unlisted() {
    let note = note("mastodon-note-cw");
    assert_eq!(
        note.summary.as_deref(),
        Some("Spoilers: season finale &amp; <b>more</b>")
    );
    assert_eq!(
        ap::html_to_text(note.summary.as_deref().expect("summary")),
        "Spoilers: season finale & more"
    );
    assert!(note.sensitive);
    assert_eq!(
        note.in_reply_to.as_deref(),
        Some("https://mastodon.example/users/ada/statuses/113200000000000001")
    );
    assert_eq!(note.updated.as_deref(), Some("2026-11-08T10:05:30Z"));
    assert_eq!(
        note.to.as_slice(),
        ["https://mastodon.example/users/ada/followers"]
    );
    assert_eq!(note.cc.as_slice(), [ap::PUBLIC], "unlisted: Public in cc");
    assert_eq!(note.attachment.len(), 1);
    assert!(note.tag.is_empty(), "an empty tag array is no tags");
}

#[test]
fn mastodon_direct_and_followers_only_notes() {
    let direct = note("mastodon-note-direct");
    assert_eq!(direct.to.as_slice(), [BEA]);
    assert!(direct.cc.is_empty());
    assert_eq!(direct.tag.as_slice(), [mention(BEA, "@bea@social.example")]);
    assert_eq!(
        ap::html_to_text(direct.content.as_deref().expect("content")),
        "@bea@social.example only you should read this."
    );
    let followers = note("mastodon-note-followers-only");
    assert_eq!(
        followers.to.as_slice(),
        ["https://mastodon.example/users/ada/followers"]
    );
    assert!(followers.cc.is_empty() && followers.tag.is_empty() && followers.attachment.is_empty());
}

#[test]
fn mastodon_single_choice_poll() {
    let note = note("mastodon-question-oneof");
    assert_eq!(note.kind, NoteKind::Question);
    assert!(!note.poll_is_multiple());
    let options: Vec<(&str, Option<u64>)> = note
        .poll_options()
        .iter()
        .map(|option| {
            (
                option.name.as_str(),
                option.replies.as_ref().and_then(|r| r.total_items),
            )
        })
        .collect();
    assert_eq!(options, [("Tea", Some(4)), ("Coffee", Some(3))]);
    assert_eq!(note.voters_count, Some(7));
    assert_eq!(note.end_time.as_deref(), Some("2026-11-09T13:00:00Z"));
    assert_eq!(note.closed, None);
    assert_eq!(note.any_of.len(), 0);
}

#[test]
fn mastodon_closed_multiple_choice_poll() {
    let note = note("mastodon-question-anyof-closed");
    assert_eq!(note.kind, NoteKind::Question);
    assert!(note.poll_is_multiple());
    assert!(note.one_of.is_empty());
    let options: Vec<(&str, Option<u64>)> = note
        .poll_options()
        .iter()
        .map(|option| {
            (
                option.name.as_str(),
                option.replies.as_ref().and_then(|r| r.total_items),
            )
        })
        .collect();
    assert_eq!(
        options,
        [("Rust", Some(8)), ("Go", Some(5)), ("Zig", Some(0))]
    );
    assert_eq!(note.voters_count, Some(10));
    assert_eq!(note.closed.as_deref(), Some("2026-11-02T08:00:00Z"));
    assert_eq!(note.end_time, note.closed);
}

#[test]
fn misskey_note_with_a_quote_in_both_spellings_and_misskey_content() {
    let value = doc("misskey-note-public");
    assert_eq!(
        value["_misskey_content"],
        "Look at this #Cats from @bea@social.example :ablobcat:"
    );
    assert_eq!(value["source"]["mediaType"], "text/x.misskeymarkdown");
    let note: Note = serde_json::from_value(value).expect("note");
    let quoted = "https://mastodon.example/users/ada/statuses/113200000000000001";
    assert_eq!(note.id, "https://misskey.example/notes/9a0noteid001");
    assert_eq!(
        note.attributed_to.as_deref(),
        Some("https://misskey.example/users/9a0example01")
    );
    assert_eq!(note.quote_url.as_deref(), Some(quoted));
    assert_eq!(note.misskey_quote.as_deref(), Some(quoted));
    assert_eq!(note.quoted(), Some(quoted));
    assert_eq!(note.published.as_deref(), Some("2026-11-08T08:00:00.000Z"));
    let at = ap::parse_time(note.published.as_deref().expect("published")).expect("time");
    assert_eq!(ap::format_time(at), "2026-11-08T08:00:00Z");
    assert_eq!(note.to.as_slice(), [ap::PUBLIC]);
    assert_eq!(
        note.cc.as_slice(),
        ["https://misskey.example/users/9a0example01/followers", BEA]
    );
    assert_eq!(note.in_reply_to, None);
    assert_eq!(note.attachment.len(), 1);
    let file = &note.attachment.as_slice()[0];
    assert_eq!(file.media_type.as_deref(), Some("image/webp"));
    assert_eq!(file.name.as_deref(), Some("A cat asleep on a sofa"));
    assert_eq!((file.width, file.height), (Some(1024), Some(768)));
    assert_eq!(
        note.tag.as_slice(),
        [
            hashtag("#Cats", "https://misskey.example/tags/Cats"),
            mention(BEA, "@bea@social.example"),
            emoji(
                "https://misskey.example/emojis/ablobcat",
                "ablobcat",
                "https://misskey.example/files/emoji/ablobcat.png"
            ),
        ]
    );
    // The quote link that follows the text, as Misskey writes it, is part of the content.
    assert!(
        note.content
            .as_deref()
            .expect("content")
            .contains("quote-inline")
    );
}

#[test]
fn misskey_quote_spellings_are_each_enough() {
    let quoted = "https://mastodon.example/users/ada/statuses/113200000000000001";
    for (key, other) in [
        ("quoteUrl", "_misskey_quote"),
        ("_misskey_quote", "quoteUrl"),
        ("quoteUri", "quoteUrl"),
    ] {
        let mut value = doc("misskey-note-public");
        let object = value.as_object_mut().expect("object");
        object.remove("quoteUrl");
        object.remove("_misskey_quote");
        object.insert(key.to_owned(), json!(quoted));
        let note: Note = serde_json::from_value(value).expect("note");
        assert_eq!(note.quoted(), Some(quoted), "{key} alone ({other} absent)");
    }
}

#[test]
fn a_misskey_hashtag_link_reads_as_the_tag_in_plain_text() {
    let note = note("misskey-note-public");
    assert_eq!(
        ap::html_to_text(note.content.as_deref().expect("content")),
        "Look at this #Cats from @bea@social.example :ablobcat:\n\nRE: https://mastodon.example/users/ada/statuses/113200000000000001"
    );
}

#[test]
fn misskey_home_note_with_a_content_warning() {
    let note = note("misskey-note-cw");
    assert_eq!(note.summary.as_deref(), Some("Mildly spicy opinions"));
    assert!(note.sensitive);
    assert_eq!(
        note.to.as_slice(),
        ["https://misskey.example/users/9a0example01/followers"]
    );
    assert_eq!(note.cc.as_slice(), [ap::PUBLIC]);
    assert_eq!(
        ap::html_to_text(note.content.as_deref().expect("content")),
        "Pineapple belongs on pizza."
    );
}

#[test]
fn misskey_empty_content_warning_is_a_zero_width_space() {
    let note = note("misskey-note-cw-empty");
    assert_eq!(note.summary.as_deref(), Some("\u{200b}"));
    assert!(note.sensitive);
    assert_eq!(note.to.as_slice(), [ap::PUBLIC]);
}

#[test]
fn misskey_poll_has_one_of_three_and_no_voter_count() {
    let note = note("misskey-question-oneof");
    assert_eq!(note.kind, NoteKind::Question);
    assert!(!note.poll_is_multiple());
    let options: Vec<(&str, Option<u64>)> = note
        .poll_options()
        .iter()
        .map(|option| {
            (
                option.name.as_str(),
                option.replies.as_ref().and_then(|r| r.total_items),
            )
        })
        .collect();
    assert_eq!(
        options,
        [("Apple", Some(2)), ("Banana", Some(5)), ("Cherry", Some(0))]
    );
    assert_eq!(note.voters_count, None);
    assert_eq!(note.end_time.as_deref(), Some("2026-11-10T09:30:00.000Z"));
}

#[test]
fn gotosocial_note_has_string_addressing_a_single_tag_and_a_blank_summary() {
    let value = doc("gts-note-public");
    assert!(value["to"].is_string() && value["cc"].is_string() && value["tag"].is_object());
    assert!(value["contentMap"]["en"].is_string());
    let note: Note = serde_json::from_value(value).expect("note");
    assert_eq!(
        note.id,
        "https://gts.example/users/ada/statuses/01FF25D5Q0DH7CHD57CTRS6WK0"
    );
    assert_eq!(note.to.as_slice(), [ap::PUBLIC]);
    assert_eq!(
        note.cc.as_slice(),
        ["https://gts.example/users/ada/followers"]
    );
    assert_eq!(note.summary.as_deref(), Some(""));
    assert!(!note.sensitive);
    assert_eq!(
        note.tag.as_slice(),
        [hashtag("#welcome", "https://gts.example/tags/welcome")]
    );
    assert_eq!(note.attachment.len(), 2);
    let first = &note.attachment.as_slice()[0];
    assert_eq!(first.kind, "Image");
    assert_eq!(first.media_type.as_deref(), Some("image/jpeg"));
    assert_eq!(first.width, None);
    assert_eq!(
        first.name.as_deref(),
        Some("Black and white image of some 50's style text saying: Welcome On Board")
    );
    assert_eq!(
        ap::html_to_text(note.content.as_deref().expect("content")),
        "Welcome to the #welcome thread. This is an example note."
    );
    assert_eq!(note.published.as_deref(), Some("2021-11-20T13:32:16Z"));
    assert_eq!(
        note.url.as_deref(),
        Some("https://gts.example/@ada/statuses/01FF25D5Q0DH7CHD57CTRS6WK0")
    );
}

#[test]
fn gotosocial_note_with_one_attachment_and_one_emoji_as_bare_objects() {
    let value = doc("gts-note-single-values");
    assert!(value["attachment"].is_object() && value["tag"].is_object());
    let note: Note = serde_json::from_value(value).expect("note");
    assert_eq!(note.attachment.len(), 1);
    assert_eq!(note.attachment.as_slice()[0].kind, "Image");
    assert_eq!(
        note.attachment.as_slice()[0].name.as_deref(),
        Some("A stinky creature")
    );
    assert_eq!(
        note.tag.as_slice(),
        [emoji(
            "https://gts.example/emoji/01AZY1Y5YQD6TREB5W50HGTCSZ",
            "shocked_pikachu",
            "https://gts.example/fileserver/01BPSX2MKCRVMD4YN4D71G9CP5/emoji/original/01AZY1Y5YQD6TREB5W50HGTCSZ.png"
        )],
        "an emoji is neither a hashtag nor a mention"
    );
    assert_eq!(note.to.as_slice(), [ap::PUBLIC]);
    assert_eq!(note.cc.len(), 1);
    assert_eq!(
        ap::html_to_text(note.content.as_deref().expect("content")),
        "here's a stinky creature -> :shocked_pikachu:"
    );
}

#[test]
fn gotosocial_reply_with_two_mentions() {
    let note = note("gts-note-reply-mention");
    assert_eq!(note.in_reply_to.as_deref(), Some(BEA_POST));
    assert_eq!(note.to.as_slice(), [ap::PUBLIC]);
    assert_eq!(note.cc.len(), 3);
    assert_eq!(
        note.tag.as_slice(),
        [
            mention(BEA, "@bea@social.example"),
            mention("https://other.example/users/zed", "@zed@other.example"),
        ]
    );
    assert_eq!(
        ap::html_to_text(note.content.as_deref().expect("content")),
        "@bea@social.example @zed@other.example thanks both!"
    );
}

#[test]
fn attributed_to_may_be_an_array_or_an_object_and_the_first_id_wins() {
    let ada = "https://mastodon.example/users/ada";
    let array = note("generic-note-attributedto-array");
    assert_eq!(array.attributed_to.as_deref(), Some(ada));
    let object = note("generic-note-attributedto-object");
    assert_eq!(object.attributed_to.as_deref(), Some(ada));
    let string = note("mastodon-note-public");
    assert_eq!(string.attributed_to.as_deref(), Some(ada));
    assert_eq!(
        ap::type_names(&doc("generic-note-attributedto-array")["attributedTo"][1]),
        ["Group"]
    );
}

#[test]
fn addressing_is_a_string_an_array_an_empty_array_or_missing() {
    let missing = note("generic-note-addressing-missing");
    assert!(missing.to.is_empty() && missing.cc.is_empty());
    let strings = note("gts-note-public");
    assert_eq!((strings.to.len(), strings.cc.len()), (1, 1));
    let arrays = note("mastodon-note-public");
    assert_eq!((arrays.to.len(), arrays.cc.len()), (1, 2));
    let empty = note("mastodon-note-direct");
    assert_eq!((empty.to.len(), empty.cc.len()), (1, 0));
    let mixed: Note = serde_json::from_value(json!({
        "id": "https://x.example/n/1", "type": "Note",
        "to": ["https://x.example/a", 7, {"id": "https://x.example/b"}, null, "https://x.example/c"],
        "cc": null
    }))
    .expect("note");
    assert_eq!(
        mixed.to.as_slice(),
        ["https://x.example/a", "https://x.example/c"]
    );
    assert!(mixed.cc.is_empty());
}

#[test]
fn an_article_reads_as_a_note_and_its_html_link_wins() {
    let note = note("generic-note-article");
    assert_eq!(note.kind, NoteKind::Article);
    assert_eq!(note.name.as_deref(), Some("Rust in the woods"));
    assert_eq!(
        note.url.as_deref(),
        Some("https://mastodon.example/articles/rust-in-the-woods")
    );
    assert_eq!(note.attachment.len(), 1);
    assert_eq!(note.attachment.as_slice()[0].name.as_deref(), Some("Cover"));
    assert_eq!(
        ap::html_to_text(note.content.as_deref().expect("content")),
        "Long form text, first paragraph.\n\nSecond paragraph."
    );
}

// ------------------------------------------------------------------ activities

#[test]
fn mastodon_create_carries_the_note_and_its_audience() {
    let create = activity("mastodon-activity-create");
    assert_eq!(create.kind, ActivityKind::Create);
    assert_eq!(
        create.id.as_deref(),
        Some("https://mastodon.example/users/ada/statuses/113200000000000010/activity")
    );
    assert_eq!(create.actor, "https://mastodon.example/users/ada");
    assert_eq!(create.published.as_deref(), Some("2026-11-08T14:00:00Z"));
    assert_eq!(create.to.as_slice(), [ap::PUBLIC]);
    assert_eq!(create.cc.len(), 1);
    assert_eq!(create.object_type(), Some("Note"));
    assert_eq!(
        create.object_id(),
        Some("https://mastodon.example/users/ada/statuses/113200000000000010")
    );
    let note = embedded_note(&create);
    assert_eq!(note.in_reply_to.as_deref(), Some(BEA_POST));
    assert_eq!(
        note.cc.as_slice(),
        ["https://mastodon.example/users/ada/followers", BEA]
    );
    assert_eq!(note.tag.as_slice(), [mention(BEA, "@bea@social.example")]);
}

#[test]
fn mastodon_update_carries_an_edited_note() {
    let update = activity("mastodon-activity-update-note");
    assert_eq!(update.kind, ActivityKind::Update);
    assert_eq!(update.object_type(), Some("Note"));
    let note = embedded_note(&update);
    assert_eq!(
        note.id,
        "https://mastodon.example/users/ada/statuses/113200000000000001"
    );
    assert_eq!(note.updated.as_deref(), Some("2026-11-08T13:30:00Z"));
    assert_eq!(
        ap::html_to_text(note.content.as_deref().expect("content")),
        "Good morning! (edited: fixed a typo)"
    );
}

#[test]
fn deletes_name_a_tombstone_a_bare_note_id_or_an_actor() {
    let tombstone = activity("mastodon-activity-delete-tombstone");
    assert_eq!(tombstone.kind, ActivityKind::Delete);
    assert_eq!(tombstone.object_type(), Some("Tombstone"));
    assert_eq!(
        tombstone.object_id(),
        Some("https://mastodon.example/users/ada/statuses/113200000000000001")
    );
    let stone: IdOrObject<Tombstone> =
        serde_json::from_value(tombstone.object.clone()).expect("tombstone");
    assert!(matches!(stone, IdOrObject::Object(_)));
    assert_eq!(stone.id(), tombstone.object_id());

    let misskey = activity("misskey-activity-delete-tombstone");
    assert_eq!(misskey.kind, ActivityKind::Delete);
    assert_eq!(misskey.object_type(), Some("Tombstone"));
    assert_eq!(
        misskey.object_id(),
        Some("https://misskey.example/notes/9a0noteid001")
    );
    assert_eq!(
        misskey.published.as_deref(),
        Some("2026-11-08T18:00:00.000Z")
    );

    let bare = activity("generic-activity-delete-bare-id");
    assert_eq!(bare.kind, ActivityKind::Delete);
    assert_eq!(bare.object_type(), None);
    assert_eq!(
        bare.object_id(),
        Some("https://mastodon.example/users/ada/statuses/113200000000000020")
    );
    let reference: IdOrObject<Tombstone> = serde_json::from_value(bare.object.clone()).expect("id");
    assert_eq!(
        reference,
        IdOrObject::Id("https://mastodon.example/users/ada/statuses/113200000000000020".to_owned())
    );

    let actor_delete = activity("mastodon-activity-delete-actor");
    assert_eq!(
        actor_delete.object_id(),
        Some(actor_delete.actor.as_str()),
        "an account deletes itself"
    );
    assert_eq!(actor_delete.object_type(), None);
}

#[test]
fn follows_accepts_and_rejects() {
    let follow = activity("mastodon-activity-follow");
    assert_eq!(follow.kind, ActivityKind::Follow);
    assert_eq!(
        follow.id.as_deref(),
        Some("https://mastodon.example/ba3d1c2e-5f6a-4a0b-8d2e-0c1f2a3b4c5d")
    );
    assert_eq!(follow.actor, "https://mastodon.example/users/ada");
    assert_eq!(follow.object_id(), Some(BEA));
    assert_eq!(follow.object_type(), None);

    for (name, kind, follow_id) in [
        (
            "mastodon-activity-accept",
            ActivityKind::Accept,
            "https://social.example/activities/0192a4b6-7c3e-7f10-9a42-3d5c8e1f2a08",
        ),
        (
            "mastodon-activity-reject",
            ActivityKind::Reject,
            "https://social.example/activities/0192a4b6-7c3e-7f10-9a42-3d5c8e1f2a09",
        ),
    ] {
        let reply = activity(name);
        assert_eq!(reply.kind, kind, "{name}");
        assert_eq!(reply.object_type(), Some("Follow"), "{name}");
        assert_eq!(reply.object_id(), Some(follow_id), "{name}");
        let inner = embedded_activity(&reply);
        assert_eq!(inner.kind, ActivityKind::Follow);
        assert_eq!(inner.actor, BEA);
        assert_eq!(
            inner.object_id(),
            Some("https://mastodon.example/users/ada")
        );
    }

    let misskey = activity("misskey-activity-accept");
    assert_eq!(misskey.kind, ActivityKind::Accept);
    assert_eq!(
        misskey.id.as_deref(),
        Some("https://misskey.example/2c5b8d4e-6a1f-4f3b-9c0d-7e8f9a0b1c2d")
    );
    assert_eq!(misskey.object_id(), Some(ADA_FOLLOW_OF_BEA));
    assert_eq!(
        embedded_activity(&misskey).object_id(),
        Some("https://misskey.example/users/9a0example01")
    );

    let gts = activity("gts-activity-accept");
    assert_eq!(gts.kind, ActivityKind::Accept);
    assert_eq!(gts.to.as_slice(), [BEA], "a string `to` is one addressee");
    assert_eq!(gts.object_id(), Some(ADA_FOLLOW_OF_BEA));
    assert_eq!(embedded_activity(&gts).actor, BEA);

    let bare = activity("generic-activity-accept-bare");
    assert_eq!(bare.kind, ActivityKind::Accept);
    assert_eq!(bare.object_type(), None);
    assert!(bare.object_map().is_none());
    assert_eq!(bare.object_id(), Some(ADA_FOLLOW_OF_BEA));
}

#[test]
fn undo_wraps_a_follow_a_like_or_an_announce() {
    let follow = activity("mastodon-activity-undo-follow");
    assert_eq!(follow.kind, ActivityKind::Undo);
    assert_eq!(follow.object_type(), Some("Follow"));
    assert_eq!(
        follow.object_id(),
        Some("https://mastodon.example/aa3e2c0a-8d0c-4e7a-9b7e-7a5c4b1e0001")
    );
    let inner = embedded_activity(&follow);
    assert_eq!(
        (inner.kind, inner.object_id()),
        (ActivityKind::Follow, Some(BEA))
    );

    let like = activity("misskey-activity-undo-like");
    assert_eq!(like.kind, ActivityKind::Undo);
    assert_eq!(like.object_type(), Some("Like"));
    let inner = embedded_activity(&like);
    assert_eq!(inner.kind, ActivityKind::Like);
    assert_eq!(inner.content.as_deref(), Some("\u{2764}"));
    assert_eq!(inner.object_id(), Some(BEA_POST));

    let announce = activity("gts-activity-undo-announce");
    assert_eq!(announce.kind, ActivityKind::Undo);
    assert_eq!(announce.to.as_slice(), [ap::PUBLIC]);
    assert_eq!(announce.object_type(), Some("Announce"));
    let inner = embedded_activity(&announce);
    assert_eq!(inner.kind, ActivityKind::Announce);
    assert_eq!(
        inner.id.as_deref(),
        Some("https://gts.example/users/ada/statuses/01H0BOOST0000000000000000A")
    );
    assert_eq!(inner.to.as_slice(), [ap::PUBLIC]);
    assert_eq!(inner.cc.len(), 2);
    assert_eq!(inner.object_id(), Some(BEA_POST));
}

#[test]
fn likes_with_and_without_an_emoji_reaction() {
    let plain = activity("mastodon-activity-like");
    assert_eq!(plain.kind, ActivityKind::Like);
    assert_eq!(
        plain.id.as_deref(),
        Some("https://mastodon.example/users/ada#likes/9")
    );
    assert_eq!(plain.object_id(), Some(BEA_POST));
    assert_eq!(plain.content, None);

    let unicode = activity("misskey-activity-like-emoji");
    assert_eq!(unicode.kind, ActivityKind::Like);
    assert_eq!(unicode.content.as_deref(), Some("\u{2764}"));
    assert_eq!(unicode.object_id(), Some(BEA_POST));

    let custom = activity("misskey-activity-like-custom-emoji");
    assert_eq!(custom.content.as_deref(), Some(":ablobcat:"));
    assert_eq!(
        custom.id.as_deref(),
        Some("https://misskey.example/likes/9a0like000002")
    );
}

#[test]
fn announce_block_flag_and_move() {
    let announce = activity("mastodon-activity-announce");
    assert_eq!(announce.kind, ActivityKind::Announce);
    assert_eq!(announce.to.as_slice(), [ap::PUBLIC]);
    assert_eq!(
        announce.cc.as_slice(),
        ["https://mastodon.example/users/ada/followers", BEA]
    );
    assert_eq!(announce.object_id(), Some(BEA_POST));
    assert_eq!(announce.published.as_deref(), Some("2026-11-08T16:00:00Z"));

    let block = activity("mastodon-activity-block");
    assert_eq!(block.kind, ActivityKind::Block);
    assert_eq!(block.object_id(), Some(BEA));

    let flag = activity("mastodon-activity-flag");
    assert_eq!(flag.kind, ActivityKind::Flag);
    assert_eq!(
        flag.actor, "https://mastodon.example/actor",
        "reports come from the instance actor"
    );
    assert_eq!(
        flag.content.as_deref(),
        Some("Spam: advertises a casino in every reply.")
    );
    assert_eq!(flag.object_ids(), [BEA, BEA_POST]);

    let moved = activity("mastodon-activity-move");
    assert_eq!(moved.kind, ActivityKind::Move);
    assert_eq!(
        moved.object_id(),
        Some("https://mastodon.example/users/ada")
    );
    assert_eq!(
        moved.target.as_deref(),
        Some("https://other.example/users/ada")
    );
}

#[test]
fn unknown_activity_types_are_other() {
    let mut value = doc("mastodon-activity-like");
    value["type"] = json!("EmojiReact");
    assert_eq!(
        serde_json::from_value::<Activity>(value.clone())
            .expect("activity")
            .kind,
        ActivityKind::Other
    );
    value["type"] = json!(["Like", "as:Like"]);
    assert_eq!(
        serde_json::from_value::<Activity>(value)
            .expect("activity")
            .kind,
        ActivityKind::Like
    );
}

// ------------------------------------------------------------------ collections

fn page_activity(items: &Collection, at: usize) -> Activity {
    serde_json::from_value(items.page_items()[at].clone()).expect("activity")
}

#[test]
fn collections_point_at_a_first_page_by_url() {
    let mastodon = collection("mastodon-collection-outbox");
    assert_eq!(mastodon.kind, "OrderedCollection");
    assert_eq!(
        mastodon.id.as_deref(),
        Some("https://mastodon.example/users/ada/outbox")
    );
    assert_eq!(mastodon.total_items, Some(128));
    assert_eq!(
        mastodon.first_url(),
        Some("https://mastodon.example/users/ada/outbox?page=true")
    );
    assert!(mastodon.first_page().is_none());
    assert!(mastodon.page_items().is_empty());

    let gts = collection("gts-collection-outbox");
    assert_eq!(gts.total_items, None);
    assert_eq!(
        gts.first_url(),
        Some("https://gts.example/users/ada/outbox?page=true")
    );
}

#[test]
fn a_first_page_may_be_embedded_with_ids_and_objects() {
    let followers = collection("generic-collection-first-embedded");
    assert_eq!(followers.total_items, Some(3));
    assert_eq!(
        followers.first_url(),
        Some("https://mastodon.example/users/ada/followers?page=1")
    );
    let page = followers.first_page().expect("embedded page");
    assert_eq!(page.kind, "OrderedCollectionPage");
    assert_eq!(
        page.part_of.as_deref(),
        Some("https://mastodon.example/users/ada/followers")
    );
    let ids: Vec<Option<&str>> = page.page_items().iter().map(id_of).collect();
    assert_eq!(
        ids,
        [
            Some(BEA),
            Some("https://gts.example/users/ada"),
            Some("https://misskey.example/users/9a0example01"),
        ]
    );
}

#[test]
fn an_outbox_page_of_embedded_activities() {
    let page = collection("mastodon-collection-outbox-page");
    assert_eq!(page.kind, "OrderedCollectionPage");
    assert_eq!(
        page.part_of.as_deref(),
        Some("https://mastodon.example/users/ada/outbox")
    );
    assert_eq!(
        page.next.as_deref(),
        Some("https://mastodon.example/users/ada/outbox?max_id=113200000000000001&page=true")
    );
    assert_eq!(page.page_items().len(), 2);
    let announce = page_activity(&page, 0);
    assert_eq!(
        (announce.kind, announce.object_id()),
        (ActivityKind::Announce, Some(BEA_POST))
    );
    let create = page_activity(&page, 1);
    assert_eq!(create.kind, ActivityKind::Create);
    let note = embedded_note(&create);
    assert_eq!(
        note.id,
        "https://mastodon.example/users/ada/statuses/113200000000000010"
    );
    assert_eq!(
        ap::html_to_text(note.content.as_deref().expect("content")),
        "Outbox item."
    );

    let misskey = collection("misskey-collection-outbox-page");
    assert_eq!(misskey.total_items, Some(2));
    assert_eq!(misskey.page_items().len(), 2);
    assert_eq!(page_activity(&misskey, 0).kind, ActivityKind::Create);
    assert_eq!(
        embedded_note(&page_activity(&misskey, 0))
            .summary
            .as_deref(),
        Some("Mildly spicy opinions")
    );
    assert_eq!(page_activity(&misskey, 1).kind, ActivityKind::Announce);
    assert_eq!(misskey.next, None);
}

#[test]
fn a_gotosocial_outbox_page_names_its_notes_by_url() {
    let page = collection("gts-collection-outbox-page");
    assert_eq!(page.page_items().len(), 2);
    let first = page_activity(&page, 0);
    assert_eq!(first.kind, ActivityKind::Create);
    assert_eq!(
        first.object_type(),
        None,
        "the object is a URL, not an embedded note"
    );
    assert_eq!(
        first.object_id(),
        Some("https://gts.example/users/ada/statuses/01FJC1MKPVX2VMWP2ST93Q90K7")
    );
    assert_eq!(first.to.as_slice(), [ap::PUBLIC]);
    assert_eq!(
        page.next.as_deref(),
        Some("https://gts.example/users/ada/outbox?max_id=01FJC1Q0E3SSQR59TD2M1KP4V8&page=true")
    );
}

#[test]
fn an_item_list_is_ids_or_embedded_notes() {
    let following = collection("gts-collection-following-page");
    assert_eq!(following.total_items, Some(397));
    let ids: Vec<Option<&str>> = following.page_items().iter().map(id_of).collect();
    assert_eq!(
        ids,
        [
            Some("https://gts.example/users/someone_else"),
            Some("https://somewhere.else.example/users/another_account"),
            Some("https://mastodon.example/users/ada"),
        ]
    );
    assert!(following.page_items().iter().all(Value::is_string));

    let featured = collection("mastodon-collection-featured");
    assert_eq!(featured.total_items, Some(1));
    let pinned: Note = serde_json::from_value(featured.page_items()[0].clone()).expect("note");
    assert_eq!(
        pinned.id,
        "https://mastodon.example/users/ada/statuses/113200000000000001"
    );

    let misskey = collection("misskey-collection-featured");
    let pinned: Note = serde_json::from_value(misskey.page_items()[0].clone()).expect("note");
    assert_eq!(pinned.id, "https://misskey.example/notes/9a0noteid001");
}

#[test]
fn unordered_items_are_read_when_there_is_no_ordered_list() {
    let page: Collection = serde_json::from_value(json!({
        "type": "CollectionPage",
        "items": ["https://x.example/a", {"id": "https://x.example/b"}]
    }))
    .expect("collection");
    assert_eq!(page.id, None);
    assert_eq!(page.page_items().len(), 2);
}

// ------------------------------------------------------------------ signatures

fn vector() -> Value {
    doc("mastodon-signature-vector")
}

fn header_of(vector: &Value, name: &str) -> Option<String> {
    vector["headers"][name].as_str().map(str::to_owned)
}

fn at(text: &str) -> time::OffsetDateTime {
    ap::parse_http_date(text).expect("http date")
}

struct Request {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl Request {
    fn of(vector: &Value) -> Self {
        Self {
            method: vector["method"].as_str().expect("method").to_owned(),
            path: vector["path"].as_str().expect("path").to_owned(),
            headers: vector["headers"]
                .as_object()
                .expect("headers")
                .iter()
                .map(|(name, value)| (name.clone(), value.as_str().expect("text").to_owned()))
                .collect(),
            body: vector["body"].as_str().expect("body").as_bytes().to_vec(),
        }
    }

    fn run(&self, vector: &Value, now: time::OffsetDateTime) -> Result<(), SignatureError> {
        let view: Vec<(&str, &str)> = self
            .headers
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str()))
            .collect();
        let parts = RequestParts {
            method: &self.method,
            path_and_query: &self.path,
            headers: &view,
        };
        let signature = parts.header("signature").ok_or(SignatureError::Malformed)?;
        let parsed = ap::parse_signature_header(&signature)?;
        ap::check_request(&parts, &parsed, &self.body, now)?;
        let key = PublicKey::from_pem(vector["publicKeyPem"].as_str().expect("pem")).expect("key");
        ap::verify_signature(&parts, &parsed, &key)
    }

    fn set(&mut self, name: &str, value: &str) {
        for (key, current) in &mut self.headers {
            if key.eq_ignore_ascii_case(name) {
                value.clone_into(current);
            }
        }
    }

    fn remove(&mut self, name: &str) {
        self.headers
            .retain(|(key, _)| !key.eq_ignore_ascii_case(name));
    }
}

fn signed_now(vector: &Value) -> time::OffsetDateTime {
    at(&header_of(vector, "Date").expect("date"))
}

#[test]
fn a_mastodon_signature_made_with_a_fixed_key_verifies() {
    let vector = vector();
    let request = Request::of(&vector);
    let header = header_of(&vector, "Signature").expect("signature");
    let parsed = ap::parse_signature_header(&header).expect("parse");
    assert_eq!(parsed.key_id, "https://mastodon.example/users/ada#main-key");
    assert_eq!(parsed.algorithm.as_deref(), Some("rsa-sha256"));
    assert_eq!(
        parsed.headers,
        ["(request-target)", "host", "date", "digest", "content-type"]
    );
    assert_eq!(parsed.signature.len(), 256, "an RSA-2048 signature");
    assert_eq!(
        ap::digest_header(&request.body),
        header_of(&vector, "Digest").expect("digest")
    );
    request.run(&vector, signed_now(&vector)).expect("verifies");
}

#[test]
fn a_tampered_body_fails_the_digest_and_a_tampered_digest_fails_the_signature() {
    let vector = vector();
    let now = signed_now(&vector);

    let mut body_changed = Request::of(&vector);
    body_changed.body.push(b' ');
    assert_eq!(
        body_changed.run(&vector, now),
        Err(SignatureError::BadDigest)
    );

    let mut digest_changed = Request::of(&vector);
    digest_changed.body.push(b' ');
    let fresh = ap::digest_header(&digest_changed.body);
    digest_changed.set("Digest", &fresh);
    assert_eq!(
        digest_changed.run(&vector, now),
        Err(SignatureError::BadSignature)
    );

    let mut garbage = Request::of(&vector);
    garbage.set("Digest", "SHA-256=AAAA");
    assert_eq!(garbage.run(&vector, now), Err(SignatureError::BadDigest));
}

#[test]
fn a_missing_host_or_date_or_digest_fails() {
    let vector = vector();
    let now = signed_now(&vector);
    for (name, expected) in [("Host", "host"), ("Date", "date"), ("Digest", "digest")] {
        let mut request = Request::of(&vector);
        request.remove(name);
        assert_eq!(
            request.run(&vector, now),
            Err(SignatureError::MissingHeader(expected)),
            "{name}"
        );
    }
}

#[test]
fn a_changed_signed_header_or_target_fails() {
    let vector = vector();
    let now = signed_now(&vector);
    let mut host = Request::of(&vector);
    host.set("Host", "evil.example");
    assert_eq!(host.run(&vector, now), Err(SignatureError::BadSignature));
    let mut kind = Request::of(&vector);
    kind.set("Content-Type", "text/plain");
    assert_eq!(kind.run(&vector, now), Err(SignatureError::BadSignature));
    let mut path = Request::of(&vector);
    path.path = "/users/mallory/inbox".to_owned();
    assert_eq!(path.run(&vector, now), Err(SignatureError::BadSignature));
    let mut method = Request::of(&vector);
    method.method = "GET".to_owned();
    assert_eq!(method.run(&vector, now), Err(SignatureError::BadSignature));
}

#[test]
fn a_date_more_than_an_hour_off_fails() {
    let vector = vector();
    let request = Request::of(&vector);
    let signed = signed_now(&vector);
    assert_eq!(
        request.run(&vector, signed + Duration::minutes(61)),
        Err(SignatureError::StaleDate)
    );
    assert_eq!(
        request.run(&vector, signed - Duration::minutes(61)),
        Err(SignatureError::StaleDate)
    );
    assert_eq!(
        request.run(&vector, signed + Duration::hours(24)),
        Err(SignatureError::StaleDate)
    );
    request
        .run(&vector, signed + Duration::minutes(59))
        .expect("inside the window");
    request
        .run(&vector, signed - Duration::minutes(59))
        .expect("inside the window");

    let mut unreadable = Request::of(&vector);
    unreadable.set("Date", "yesterday-ish");
    assert_eq!(
        unreadable.run(&vector, signed),
        Err(SignatureError::StaleDate)
    );
}

#[test]
fn a_signature_by_another_key_or_over_other_headers_is_refused() {
    let vector = vector();
    let now = signed_now(&vector);
    let other = ap::generate_keypair().expect("keys");
    let mut other_vector = vector.clone();
    other_vector["publicKeyPem"] = json!(other.public_pem);
    assert_eq!(
        Request::of(&other_vector).run(&other_vector, now),
        Err(SignatureError::BadSignature)
    );

    let mut request = Request::of(&vector);
    let header = header_of(&vector, "Signature").expect("signature");
    request.set(
        "Signature",
        &header.replace(
            "(request-target) host date digest content-type",
            "(request-target) host date content-type",
        ),
    );
    assert_eq!(
        request.run(&vector, now),
        Err(SignatureError::Unsigned("digest"))
    );
    request.set(
        "Signature",
        &header.replace(
            "(request-target) host date digest content-type",
            "(request-target) date digest",
        ),
    );
    assert_eq!(
        request.run(&vector, now),
        Err(SignatureError::Unsigned("host"))
    );
    request.set("Signature", &header.replace("rsa-sha256", "hmac-sha256"));
    assert_eq!(
        request.run(&vector, now),
        Err(SignatureError::UnsupportedAlgorithm)
    );
    request.set("Signature", "keyId=\"k\"");
    assert_eq!(request.run(&vector, now), Err(SignatureError::Malformed));
}

#[test]
fn a_request_we_sign_ourselves_verifies_with_the_same_covered_headers() {
    let keys = ap::generate_keypair().expect("keys");
    let private = ap::PrivateKey::from_pem(&keys.private_pem).expect("private");
    let public = PublicKey::from_pem(&keys.public_pem).expect("public");
    let url = url_of("https://social.example/users/bea/inbox");
    let body = br#"{"type":"Follow"}"#;
    let now = at("Mon, 09 Nov 2026 12:00:00 GMT");
    let signed = ap::sign_post(
        &private,
        "https://mastodon.example/users/ada#main-key",
        &url,
        body,
        now,
    )
    .expect("sign");
    let headers: Vec<(&str, String)> = signed
        .iter()
        .map(|(name, value)| (*name, value.clone()))
        .collect();
    let view: Vec<(&str, &str)> = headers
        .iter()
        .map(|(name, value)| (*name, value.as_str()))
        .collect();
    let parts = RequestParts {
        method: "POST",
        path_and_query: "/users/bea/inbox",
        headers: &view,
    };
    let parsed =
        ap::parse_signature_header(&parts.header("signature").expect("header")).expect("parse");
    let mastodon = ap::parse_signature_header(
        vector()["headers"]["Signature"]
            .as_str()
            .expect("signature"),
    )
    .expect("parse");
    assert_eq!(
        parsed.headers, mastodon.headers,
        "we cover what Mastodon covers"
    );
    ap::check_request(&parts, &parsed, body, now).expect("checks");
    ap::verify_signature(&parts, &parsed, &public).expect("verifies");
    assert_eq!(
        ap::check_request(&parts, &parsed, b"{}", now),
        Err(SignatureError::BadDigest)
    );
}

fn url_of(text: &str) -> url::Url {
    url::Url::parse(text).expect("url")
}

// ------------------------------------------------------------------ HTML

#[test]
fn script_and_event_handlers_are_stripped_from_every_fixture_text() {
    let dirty = [
        "<p>hi<script>alert(1)</script></p>",
        "<p>hi<img src=x onerror=alert(1)></p>",
        "<p onclick=\"alert(1)\">hi <a href=\"javascript:alert(1)\" onmouseover=\"alert(1)\">x</a></p>",
        "<p>hi<style>p { display: none }</style><iframe src=\"https://evil.example\"></iframe></p>",
        "<SCRIPT SRC=https://evil.example/x.js></SCRIPT><p>hi</p>",
        "<p>hi<svg onload=alert(1)></svg></p>",
    ];
    for html in dirty {
        let clean = ap::sanitize_html(html);
        for needle in [
            "script",
            "alert",
            "onerror",
            "onclick",
            "onmouseover",
            "onload",
            "javascript:",
            "<img",
            "<iframe",
            "<style",
            "<svg",
            "display",
        ] {
            assert!(
                !clean.to_ascii_lowercase().contains(needle),
                "{needle} survived in {clean:?} (from {html:?})"
            );
        }
        assert!(clean.contains("hi"), "{clean:?}");
    }
    assert_eq!(
        ap::sanitize_html("<p>hi<script>alert(1)</script></p>"),
        "<p>hi</p>"
    );
    assert_eq!(
        ap::html_to_text("<p>hi<img src=x onerror=alert(1)></p>"),
        "hi"
    );

    // Hostile markup put into the content of a real fixture comes out as plain text.
    let mut value = doc("mastodon-note-public");
    value["content"] = json!(
        "<p>safe</p><script>fetch('https://evil.example')</script><img src=x onerror=alert(1)>"
    );
    value["summary"] = json!("cw<script>alert(1)</script>");
    let note: Note = serde_json::from_value(value).expect("note");
    assert_eq!(
        ap::html_to_text(note.content.as_deref().expect("content")),
        "safe"
    );
    assert_eq!(
        ap::html_to_text(note.summary.as_deref().expect("summary")),
        "cw"
    );

    // Nothing any fixture carries as `content`, `summary` or `value` keeps active markup.
    for entry in std::fs::read_dir(dir()).expect("dir") {
        let text = std::fs::read_to_string(entry.expect("entry").path()).expect("read");
        let mut stack = vec![serde_json::from_str::<Value>(&text).expect("json")];
        while let Some(node) = stack.pop() {
            match node {
                Value::Object(map) => {
                    for (key, item) in map {
                        if let (true, Value::String(html)) = (
                            matches!(key.as_str(), "content" | "summary" | "value"),
                            &item,
                        ) {
                            let clean = ap::sanitize_html(html);
                            assert!(
                                !clean.contains("<script") && !clean.contains("onerror"),
                                "{clean}"
                            );
                        }
                        stack.push(item);
                    }
                }
                Value::Array(items) => stack.extend(items),
                _ => {}
            }
        }
    }
}

// ------------------------------------------------------------------ hostile variants

/// `depth` objects nested inside each other under `key`, ending in a string.
fn nested(depth: usize) -> Value {
    let mut value = json!("leaf");
    for _ in 0..depth {
        value = json!({ "a": value });
    }
    value
}

fn bytes_of(value: &Value) -> Vec<u8> {
    serde_json::to_vec(value).expect("json")
}

#[test]
fn nesting_beyond_the_depth_limit_is_refused_at_the_boundary() {
    let mut accepted = None;
    for depth in 0..60 {
        let mut value = doc("mastodon-activity-create");
        value["object"]["extra"] = nested(depth);
        match ap::parse_document(&bytes_of(&value), 1 << 20) {
            Ok(_) => accepted = Some(depth),
            Err(err) => {
                assert_eq!(err, ParseError::TooDeep, "depth {depth}");
                assert!(
                    accepted.expect("some depth passes") + 1 == depth,
                    "the limit is one boundary, not a range"
                );
                // object (2) + extra (3) + `depth` objects + the leaf string.
                assert_eq!(
                    2 + 1 + depth,
                    MAX_DEPTH + 1,
                    "the first refused document is one level past MAX_DEPTH"
                );
                return;
            }
        }
    }
    panic!("nothing was refused");
}

#[test]
fn deeply_nested_documents_are_refused_whatever_the_field() {
    for field in [
        "object",
        "tag",
        "attachment",
        "to",
        "cc",
        "orderedItems",
        "unknownField",
    ] {
        let mut value = doc("mastodon-collection-outbox-page");
        value[field] = nested(100);
        assert_eq!(
            ap::parse_document(&bytes_of(&value), 1 << 20),
            Err(ParseError::TooDeep),
            "{field}"
        );
    }
    let bomb = format!("{{\"a\":{}", "[".repeat(200_000));
    assert_eq!(
        ap::parse_document(bomb.as_bytes(), 1 << 20),
        Err(ParseError::Invalid)
    );
    let arrays = format!(
        "{{\"a\":{}1{}}}",
        "[".repeat(MAX_DEPTH + 5),
        "]".repeat(MAX_DEPTH + 5)
    );
    assert_eq!(
        ap::parse_document(arrays.as_bytes(), 1 << 20),
        Err(ParseError::TooDeep)
    );
}

#[test]
fn arrays_at_the_node_limit_are_refused_and_just_under_it_are_capped_by_the_readers() {
    let mut huge = doc("mastodon-note-public");
    huge["to"] = Value::Array(
        (0..MAX_NODES)
            .map(|n| json!(format!("https://x.example/{n}")))
            .collect(),
    );
    assert_eq!(
        ap::parse_document(&bytes_of(&huge), 1 << 24),
        Err(ParseError::TooComplex)
    );

    let mut wide = doc("mastodon-note-public");
    wide["to"] = Value::Array(
        (0..1_000)
            .map(|n| json!(format!("https://x.example/{n}")))
            .collect(),
    );
    wide["cc"] = wide["to"].clone();
    wide["tag"] = Value::Array(
        (0..1_000)
            .map(|n| json!({"type": "Hashtag", "name": format!("#t{n}")}))
            .collect(),
    );
    wide["attachment"] = Value::Array(
        (0..1_000)
            .map(|n| json!({"type": "Image", "url": format!("https://x.example/{n}.png")}))
            .collect(),
    );
    let parsed = ap::parse_document(&bytes_of(&wide), 1 << 24).expect("under the node limit");
    let note: Note = serde_json::from_value(parsed).expect("note");
    for (name, len) in [
        ("to", note.to.len()),
        ("cc", note.cc.len()),
        ("tag", note.tag.len()),
        ("attachment", note.attachment.len()),
    ] {
        assert_eq!(len, 256, "{name} is capped by OneOrMany");
    }

    let mut collection = doc("gts-collection-following-page");
    collection["orderedItems"] = Value::Array(
        (0..10_000)
            .map(|n| json!(format!("https://x.example/u/{n}")))
            .collect(),
    );
    let parsed = ap::parse_document(&bytes_of(&collection), 1 << 24).expect("under the node limit");
    let page: Collection = serde_json::from_value(parsed).expect("collection");
    assert_eq!(page.page_items().len(), 256);

    let oversized = bytes_of(&wide);
    assert_eq!(
        ap::parse_document(&oversized, oversized.len() - 1),
        Err(ParseError::TooLarge)
    );
}

#[test]
fn hostile_field_types_never_panic_and_never_read_as_something_else() {
    let hostile = [
        json!(null),
        json!(true),
        json!(7),
        json!(-1.5e300),
        json!("text"),
        json!([]),
        json!([[[]]]),
        json!({}),
        json!({"id": 5}),
        json!({"type": ["Note", 3, null], "id": ["https://x.example/a"]}),
    ];
    for fixture in [
        "mastodon-note-public",
        "mastodon-activity-create",
        "mastodon-actor-person",
        "mastodon-collection-outbox-page",
    ] {
        let base = doc(fixture);
        for key in base.as_object().expect("object").keys() {
            for value in &hostile {
                let mut broken = base.clone();
                broken[key.as_str()] = value.clone();
                let _ = serde_json::from_value::<Note>(broken.clone());
                let _ = serde_json::from_value::<Actor>(broken.clone());
                let _ = serde_json::from_value::<Activity>(broken.clone());
                let _ = serde_json::from_value::<Collection>(broken.clone());
                let _ = id_of(&broken[key.as_str()]);
            }
        }
    }
    // A required field of the wrong type is an error, an optional one is simply absent.
    let mut no_id = doc("mastodon-note-public");
    no_id["id"] = json!(5);
    assert!(serde_json::from_value::<Note>(no_id).is_err());
    let mut weird = doc("mastodon-note-public");
    weird["to"] = json!({"id": "https://x.example/a"});
    weird["tag"] = json!(7);
    weird["attachment"] = json!("https://x.example/a.png");
    weird["content"] = json!(["a"]);
    weird["sensitive"] = json!("TRUE");
    weird["published"] = json!(12);
    let note: Note = serde_json::from_value(weird).expect("lenient note");
    assert!(note.to.is_empty() && note.attachment.is_empty());
    assert_eq!(
        note.tag.as_slice(),
        [Tag::Other],
        "a number is a tag we do not use"
    );
    assert_eq!(note.content, None);
    assert!(note.sensitive, "the text \"true\" is a true flag");
    assert_eq!(note.published.as_deref(), Some("12"));
}

#[test]
fn a_document_that_is_not_an_object_is_refused() {
    for body in [
        &b"[]"[..],
        b"\"note\"",
        b"null",
        b"7",
        b"",
        b"{",
        b"{\"a\":}",
    ] {
        assert_eq!(
            ap::parse_document(body, 1 << 20),
            Err(ParseError::Invalid),
            "{body:?}"
        );
    }
}

// ------------------------------------------------------------------ emoji tags

#[test]
fn actors_carry_their_emoji_tags_in_every_shape() {
    let gts = actor("gts-actor-person");
    assert_eq!(
        gts.tag.as_slice(),
        [emoji(
            "https://gts.example/emoji/01F8MH9H8E4VG3KDYJR9EGPXCQ",
            "rainbow",
            "https://gts.example/fileserver/01H0AVATAR0000000000000000/emoji/original/01F8MH9H8E4VG3KDYJR9EGPXCQ.png"
        )],
        "a bare tag object is one tag"
    );
    let mastodon = actor("mastodon-actor-person");
    assert_eq!(
        mastodon.tag.as_slice()[0],
        emoji(
            "https://mastodon.example/emojis/17",
            "blobcat",
            "https://mastodon.example/system/custom_emojis/images/000/000/017/original/blobcat.png"
        )
    );
    assert!(matches!(mastodon.tag.as_slice()[1], Tag::Hashtag { .. }));
    assert_eq!(actor("misskey-actor-person").tag.len(), 2);
}

#[test]
fn unusable_emoji_tags_read_as_other() {
    let read = |value: Value| -> Tag { serde_json::from_value(value).expect("tag") };
    let icon = json!({"type": "Image", "url": "https://x.example/a.png"});
    for bad in [
        json!({"type": "Emoji", "name": "blob", "icon": icon}),
        json!({"type": "Emoji", "name": ":b:", "icon": icon}),
        json!({"type": "Emoji", "name": ":bl-ob:", "icon": icon}),
        json!({"type": "Emoji", "name": ":blob:"}),
        json!({"type": "Emoji", "name": ":blob:", "icon": {"url": "javascript:alert(1)"}}),
        json!({"type": "Emoji", "name": ":blob:", "icon": {"url": "ftp://x.example/a.png"}}),
        json!({"type": "Emoji", "name": 7, "icon": icon}),
    ] {
        assert_eq!(read(bad.clone()), Tag::Other, "{bad}");
    }
}

#[test]
fn an_emoji_without_an_id_is_known_by_its_icon_and_the_tag_round_trips() {
    let tag: Tag = serde_json::from_value(json!({
        "type": "Emoji",
        "name": ":blob:",
        "icon": [{"type": "Image", "mediaType": "image/gif", "url": "https://x.example/blob.gif"}],
    }))
    .expect("tag");
    assert_eq!(
        tag,
        Tag::Emoji {
            id: "https://x.example/blob.gif".to_owned(),
            name: "blob".to_owned(),
            icon: "https://x.example/blob.gif".to_owned(),
            media_type: Some("image/gif".to_owned()),
        }
    );
    let wire = serde_json::to_value(&tag).expect("serialise");
    assert_eq!(
        wire,
        json!({
            "type": "Emoji",
            "id": "https://x.example/blob.gif",
            "name": ":blob:",
            "icon": {"type": "Image", "mediaType": "image/gif", "url": "https://x.example/blob.gif"},
        })
    );
    assert_eq!(serde_json::from_value::<Tag>(wire).expect("back"), tag);
}
