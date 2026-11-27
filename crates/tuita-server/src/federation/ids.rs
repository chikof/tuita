//! The ids and URLs of local objects, and the parsers that recognise them again. Pure functions
//! of `PUBLIC_URL`; a parser returns `None` for any id that is not ours.

use uuid::Uuid;

use crate::config::Config;

/// `{PUBLIC_URL}/users/{handle}`: the actor id.
#[must_use]
pub fn actor_id(config: &Config, handle: &str) -> String {
    format!("{}/users/{handle}", config.public_url)
}

/// The actor's personal inbox.
#[must_use]
pub fn inbox_url(config: &Config, handle: &str) -> String {
    format!("{}/inbox", actor_id(config, handle))
}

/// The instance's shared inbox.
#[must_use]
pub fn shared_inbox_url(config: &Config) -> String {
    format!("{}/inbox", config.public_url)
}

/// The actor's outbox.
#[must_use]
pub fn outbox_url(config: &Config, handle: &str) -> String {
    format!("{}/outbox", actor_id(config, handle))
}

/// The actor's followers collection.
#[must_use]
pub fn followers_url(config: &Config, handle: &str) -> String {
    format!("{}/followers", actor_id(config, handle))
}

/// The actor's following collection.
#[must_use]
pub fn following_url(config: &Config, handle: &str) -> String {
    format!("{}/following", actor_id(config, handle))
}

/// The actor's collection of pinned posts.
#[must_use]
pub fn featured_url(config: &Config, handle: &str) -> String {
    format!("{}/featured", actor_id(config, handle))
}

/// The HTML profile page, `{PUBLIC_URL}/@{handle}`.
#[must_use]
pub fn profile_url(config: &Config, handle: &str) -> String {
    format!("{}/@{handle}", config.public_url)
}

/// The id of a post's `Note`.
#[must_use]
pub fn note_id(config: &Config, handle: &str, post: Uuid) -> String {
    format!("{}/statuses/{post}", actor_id(config, handle))
}

/// The HTML page of a post, `{PUBLIC_URL}/@{handle}/{post}`.
#[must_use]
pub fn note_url(config: &Config, handle: &str, post: Uuid) -> String {
    format!("{}/@{handle}/{post}", config.public_url)
}

/// An activity id that no object names: `{PUBLIC_URL}/activities/{id}`.
#[must_use]
pub fn activity_id(config: &Config, id: Uuid) -> String {
    format!("{}/activities/{id}", config.public_url)
}

/// The instance actor, which signs fetches.
#[must_use]
pub fn instance_actor_id(config: &Config) -> String {
    format!("{}/actor", config.public_url)
}

/// The id of the key of `actor_id`, which a signature names as `keyId`.
#[must_use]
pub fn key_id(actor_id: &str) -> String {
    format!("{actor_id}#main-key")
}

/// The handle of the local actor `id` names, or `None` when it is not one of ours.
///
/// ```
/// # use tuita_server::federation::ids::local_handle_from_actor_id;
/// # let config = tuita_server::Config::from_lookup(|v| match v {
/// #     "DATABASE_URL" => Some("postgres://x".to_owned()),
/// #     "PUBLIC_URL" => Some("https://a.example".to_owned()),
/// #     _ => None,
/// # }).unwrap();
/// assert_eq!(local_handle_from_actor_id(&config, "https://a.example/users/ada"), Some("ada"));
/// assert_eq!(local_handle_from_actor_id(&config, "https://b.example/users/ada"), None);
/// ```
#[must_use]
pub fn local_handle_from_actor_id<'a>(config: &Config, id: &'a str) -> Option<&'a str> {
    let handle = id
        .strip_prefix(&*config.public_url)?
        .strip_prefix("/users/")?;
    plain_segment(handle)
}

/// The author handle and post id of a local `Note` id, or `None`.
#[must_use]
pub fn local_post_from_note_id<'a>(config: &Config, id: &'a str) -> Option<(&'a str, Uuid)> {
    let rest = id
        .strip_prefix(&*config.public_url)?
        .strip_prefix("/users/")?;
    let (handle, post) = rest.split_once("/statuses/")?;
    Some((plain_segment(handle)?, plain_segment(post)?.parse().ok()?))
}

/// `segment` when it is one non-empty path segment without query, fragment, escapes or control
/// characters (a NUL cannot be looked up).
fn plain_segment(segment: &str) -> Option<&str> {
    (!segment.is_empty()
        && !segment.contains(['/', '?', '#', '%', '\\'])
        && !segment.chars().any(char::is_control))
    .then_some(segment)
}

#[cfg(test)]
mod test {
    use super::*;

    fn config() -> Config {
        Config::from_lookup(|var| match var {
            "DATABASE_URL" => Some("postgres://x".to_owned()),
            "PUBLIC_URL" => Some("http://127.0.0.1:8081".to_owned()),
            _ => None,
        })
        .expect("config")
    }

    #[test]
    fn ids_follow_the_scheme_and_parse_back() {
        let config = config();
        let post = Uuid::now_v7();
        assert_eq!(actor_id(&config, "ada"), "http://127.0.0.1:8081/users/ada");
        assert_eq!(
            inbox_url(&config, "ada"),
            "http://127.0.0.1:8081/users/ada/inbox"
        );
        assert_eq!(shared_inbox_url(&config), "http://127.0.0.1:8081/inbox");
        assert_eq!(profile_url(&config, "ada"), "http://127.0.0.1:8081/@ada");
        assert_eq!(instance_actor_id(&config), "http://127.0.0.1:8081/actor");
        assert_eq!(
            key_id(&actor_id(&config, "ada")),
            "http://127.0.0.1:8081/users/ada#main-key"
        );
        let note = note_id(&config, "ada", post);
        assert_eq!(local_post_from_note_id(&config, &note), Some(("ada", post)));
        assert_eq!(
            note_url(&config, "ada", post),
            format!("http://127.0.0.1:8081/@ada/{post}")
        );
        assert_eq!(
            activity_id(&config, post),
            format!("http://127.0.0.1:8081/activities/{post}")
        );
    }

    #[test]
    fn foreign_and_odd_ids_do_not_parse() {
        let config = config();
        for id in [
            "http://127.0.0.1:8082/users/ada",
            "http://127.0.0.1:8081/users/",
            "http://127.0.0.1:8081/users/ada/inbox",
            "http://127.0.0.1:8081/users/ada#main-key",
            "http://127.0.0.1:80810/users/ada",
            "http://127.0.0.1:8081/users/a%2Fb",
        ] {
            assert_eq!(local_handle_from_actor_id(&config, id), None, "{id}");
        }
        assert_eq!(
            local_post_from_note_id(&config, "http://127.0.0.1:8081/users/ada/statuses/nope"),
            None
        );
    }
}
