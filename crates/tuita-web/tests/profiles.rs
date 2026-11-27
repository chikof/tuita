//! Profiles, follow lists, explore and notifications through the whole app, in-process.
//!
//! Each `#[sqlx::test]` gets a fresh database; `DATABASE_URL` must point at a running Postgres.
//! Run `topcoat asset bundle --package tuita` first (or point `TUITA_TEST_ASSETS` at a bundle).

mod common;

use axum::http::{Method, StatusCode};
use common::{Browser, Reply, app};
use serde_json::{Value, json};

/// Creates a post as the signed-in `who` and returns its id.
async fn post(who: &mut Browser, text: &str) -> String {
    let reply = who
        .request(
            Method::POST,
            "/api/v1/posts",
            Some(json!({"text": text, "cw": "", "sensitive": false, "mediaIds": []})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "post: {}", reply.body);
    let made: Value = serde_json::from_str(&reply.body).expect("post json");
    made["id"].as_str().expect("post id").to_owned()
}

/// `POST path` as the signed-in `who`.
async fn act(who: &mut Browser, path: &str) {
    let reply = who
        .request(Method::POST, &format!("/api/v1{path}"), None)
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{path}: {}", reply.body);
}

/// `GET path` as a fragment fetch, the way `infinite.js` asks for the next page.
async fn fragment(who: &mut Browser, path: &str) -> Reply {
    who.headers
        .push(("x-tuita-fragment".to_owned(), "1".to_owned()));
    let reply = who.get(path).await;
    who.headers.clear();
    reply
}

/// The `href` of the `data-more` link in `html`, entities undone.
fn more_link(html: &str) -> String {
    let at = html.find("data-more").expect("a link to more");
    let rest = &html[at..];
    let start = rest.find("href=\"").expect("an href") + 6;
    let end = start + rest[start..].find('"').expect("the href ends");
    rest[start..end].replace("&amp;", "&")
}

mod profile {
    use sqlx::PgPool;

    use super::*;

    #[sqlx::test(migrations = "../tuita-server/migrations")]
    async fn a_profile_shows_its_header_tabs_and_posts(db: PgPool) {
        let router = app(db);
        let mut ana = Browser::new(&router);
        ana.signup("ana").await;
        post(&mut ana, "An ordinary note").await;
        let mut bob = Browser::new(&router);
        bob.signup("bob").await;

        let reply = bob.get("/@ana").await;
        assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
        let html = &reply.body;
        assert!(html.contains("<h1>Name of ana</h1>"), "{html}");
        assert!(html.contains("@ana"), "{html}");
        assert!(html.contains(r#"role="tablist""#), "{html}");
        for tab in ["Posts", "Replies", "Media", "Likes"] {
            assert!(html.contains(&format!(">{tab}")), "{tab}: {html}");
        }
        assert!(html.contains("An ordinary note"), "{html}");
        assert!(html.contains("data-follow"), "{html}");
        assert!(
            html.contains("Name of ana (@ana) · tuita</title>"),
            "{html}"
        );
        assert!(html.contains("Joined"), "{html}");
    }

    #[sqlx::test(migrations = "../tuita-server/migrations")]
    async fn a_pinned_post_shows_once_and_the_page_loads(db: PgPool) {
        let mut ana = Browser::new(&app(db));
        ana.signup("ana").await;
        post(&mut ana, "first note").await;
        let pinned = post(&mut ana, "the pinned note").await;
        post(&mut ana, "last note").await;
        act(&mut ana, &format!("/posts/{pinned}/pin")).await;

        let reply = ana.get("/@ana").await;
        assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
        assert_eq!(
            reply
                .body
                .matches(&format!("data-post=\"{pinned}\""))
                .count(),
            1,
            "the pinned post is on the page once"
        );
        assert!(reply.body.contains("first note"), "{}", reply.body);
        assert!(reply.body.contains("last note"), "{}", reply.body);
        let pin = reply.body.find("the pinned note").expect("pinned");
        let first = reply.body.find("first note").expect("first");
        assert!(pin < first, "the pinned post comes first");
    }

    #[sqlx::test(migrations = "../tuita-server/migrations")]
    async fn the_tab_and_media_filter_are_read_from_the_query(db: PgPool) {
        let mut ana = Browser::new(&app(db));
        ana.signup("ana").await;
        let replies = ana.get("/@ana?tab=replies").await;
        assert!(replies.body.contains("No replies yet"), "{}", replies.body);
        assert!(replies.body.contains(r#"aria-selected="true""#));
        let videos = ana.get("/@ana?tab=media&media=video").await;
        assert!(videos.body.contains("No videos yet"), "{}", videos.body);
        assert!(videos.body.contains("Media type"), "{}", videos.body);
        assert!(videos.body.contains(r#"aria-checked="true""#));
        let own = ana.get("/@ana").await;
        assert!(own.body.contains("You haven’t posted yet"), "{}", own.body);
        assert!(own.body.contains("Edit profile"), "{}", own.body);
    }

    #[sqlx::test(migrations = "../tuita-server/migrations")]
    async fn more_posts_come_as_a_fragment_of_items_and_the_next_link(db: PgPool) {
        let mut ana = Browser::new(&app(db));
        ana.signup("ana").await;
        for i in 0..22 {
            post(&mut ana, &format!("note number {i:02}")).await;
        }
        let page = ana.get("/@ana").await;
        assert!(page.body.contains("note number 21"), "{}", page.body);
        assert!(!page.body.contains("note number 00"));
        let next = more_link(&page.body);
        assert!(next.starts_with("/@ana?"), "{next}");
        assert!(next.contains("before="), "{next}");

        let more = fragment(&mut ana, &next).await;
        assert_eq!(more.status, StatusCode::OK);
        assert!(!more.body.contains("<html"), "{}", more.body);
        assert!(!more.body.contains("<h1"), "no page chrome");
        assert!(more.body.contains("note number 00"), "{}", more.body);
        assert!(more.body.contains("note number 01"), "{}", more.body);
        assert!(!more.body.contains("data-more"), "that was the last page");
    }

    #[sqlx::test(migrations = "../tuita-server/migrations")]
    async fn an_unknown_account_is_the_not_found_page(db: PgPool) {
        let mut ana = Browser::new(&app(db));
        ana.signup("ana").await;
        let reply = ana.get("/@ghost").await;
        assert_eq!(reply.status, StatusCode::NOT_FOUND, "{}", reply.body);
        assert!(reply.body.contains("Nothing here"), "{}", reply.body);
    }

    #[sqlx::test(migrations = "../tuita-server/migrations")]
    async fn an_unknown_remote_account_says_so(db: PgPool) {
        let mut ana = Browser::new(&app(db));
        ana.signup("ana").await;
        let reply = ana.get("/@mara@localhost:1").await;
        assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
        assert!(reply.body.contains("@mara@localhost:1"), "{}", reply.body);
        assert!(reply.body.contains("No such account"), "{}", reply.body);
    }

    #[sqlx::test(migrations = "../tuita-server/migrations")]
    async fn a_blocked_account_shows_the_blocked_state_with_unblock(db: PgPool) {
        let router = app(db);
        let mut ana = Browser::new(&router);
        ana.signup("ana").await;
        let mut bob = Browser::new(&router);
        bob.signup("bob").await;
        act(&mut bob, "/users/ana/block").await;
        let reply = bob.get("/@ana").await;
        assert!(reply.body.contains("You blocked @ana"), "{}", reply.body);
        assert!(
            reply.body.contains(r#"data-profile-act="unblock""#),
            "{}",
            reply.body
        );
        assert!(
            !reply.body.contains("Profile sections"),
            "no tabs for a blocked account"
        );

        let theirs = ana.get("/@bob").await;
        assert!(
            theirs.body.contains("You can’t see these posts"),
            "{}",
            theirs.body
        );
    }

    #[sqlx::test(migrations = "../tuita-server/migrations")]
    async fn the_follow_lists_show_people_with_follow_buttons(db: PgPool) {
        let router = app(db);
        let mut ana = Browser::new(&router);
        ana.signup("ana").await;
        let mut bob = Browser::new(&router);
        bob.signup("bob").await;
        act(&mut bob, "/users/ana/follow").await;

        let followers = ana.get("/@ana/followers").await;
        assert_eq!(followers.status, StatusCode::OK, "{}", followers.body);
        assert!(followers.body.contains("Name of bob"), "{}", followers.body);
        assert!(followers.body.contains(r#"aria-label="Followers""#));
        assert!(followers.body.contains("data-follow"), "{}", followers.body);
        assert!(
            followers
                .body
                .contains("Name of ana (@ana) · tuita</title>")
        );

        let following = bob.get("/@bob/following").await;
        assert!(following.body.contains("Name of ana"), "{}", following.body);
        let empty = ana.get("/@ana/following").await;
        assert!(
            empty.body.contains("Not following anyone"),
            "{}",
            empty.body
        );
    }
}

mod explore {
    use sqlx::PgPool;

    use super::*;

    #[sqlx::test(migrations = "../tuita-server/migrations")]
    async fn the_home_lists_trends_and_suggestions(db: PgPool) {
        let router = app(db);
        let mut ana = Browser::new(&router);
        ana.signup("ana").await;
        post(&mut ana, "talking about #pixels today").await;
        let mut bob = Browser::new(&router);
        bob.signup("bob").await;

        let reply = bob.get("/explore").await;
        assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
        let html = &reply.body;
        assert!(html.contains(">Trending<"), "{html}");
        assert!(html.contains("#"), "{html}");
        assert!(html.contains("pixels"), "{html}");
        assert!(html.contains(">Who to follow<"), "{html}");
        assert!(html.contains("Name of ana"), "{html}");
        assert!(html.contains(r#"data-island="recent-searches""#), "{html}");
        assert!(html.contains(r#"role="combobox""#), "{html}");
        for tab in ["For you", "Trending", "People"] {
            assert!(html.contains(tab), "{tab}");
        }
    }

    #[sqlx::test(migrations = "../tuita-server/migrations")]
    async fn a_query_searches_posts_by_default_and_people_on_the_people_tab(db: PgPool) {
        let router = app(db);
        let mut ana = Browser::new(&router);
        ana.signup("ana").await;
        post(&mut ana, "a note about narwhals").await;
        let mut bob = Browser::new(&router);
        bob.signup("bob").await;

        let posts = bob.get("/explore?q=narwhals").await;
        assert!(
            posts.body.contains("a note about narwhals"),
            "{}",
            posts.body
        );
        assert!(
            posts.body.contains(r#"aria-label="Posts""#),
            "{}",
            posts.body
        );
        assert!(posts.body.contains(r#"value="narwhals""#), "{}", posts.body);

        let people = bob.get("/explore?q=ana&type=people").await;
        assert!(people.body.contains("Name of ana"), "{}", people.body);
        assert!(people.body.contains("data-follow"), "{}", people.body);
        assert!(
            people.body.contains(r#"aria-label="People""#),
            "{}",
            people.body
        );

        let none = bob.get("/explore?q=zzzzqqqq").await;
        assert!(none.body.contains("No posts for"), "{}", none.body);
        let nobody = bob.get("/explore?q=zzzzqqqq&type=people").await;
        assert!(nobody.body.contains("No people for"), "{}", nobody.body);
        let media = bob.get("/explore?q=zzzzqqqq&type=media").await;
        assert!(media.body.contains("No images for"), "{}", media.body);
    }

    #[sqlx::test(migrations = "../tuita-server/migrations")]
    async fn a_lookup_query_explains_an_empty_answer(db: PgPool) {
        let mut ana = Browser::new(&app(db));
        ana.signup("ana").await;
        let reply = ana
            .get("/explore?q=%40mara%40localhost%3A1&type=people")
            .await;
        assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
        assert!(reply.body.contains("No people for"), "{}", reply.body);
        assert!(reply.body.contains("wasn’t found"), "{}", reply.body);
    }
}

mod notifications {
    use sqlx::PgPool;

    use super::*;

    #[sqlx::test(migrations = "../tuita-server/migrations")]
    async fn the_list_shows_what_happened_and_is_empty_for_others(db: PgPool) {
        let router = app(db);
        let mut ana = Browser::new(&router);
        ana.signup("ana").await;
        let mut bob = Browser::new(&router);
        bob.signup("bob").await;
        let id = post(&mut ana, "like me please").await;
        act(&mut bob, "/users/ana/follow").await;
        act(&mut bob, &format!("/posts/{id}/like")).await;

        let reply = ana.get("/notifications").await;
        assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
        let html = &reply.body;
        assert!(html.contains(r#"data-island="notifications""#), "{html}");
        assert!(html.contains(r#"data-stream="notification""#), "{html}");
        assert!(html.contains("Name of bob"), "{html}");
        assert!(html.contains("followed you"), "{html}");
        assert!(html.contains("liked your post"), "{html}");
        assert!(html.contains("unread"), "{html}");
        assert!(
            html.contains(r#"aria-label="Notification filter""#),
            "{html}"
        );

        let empty = bob.get("/notifications").await;
        assert!(
            empty.body.contains("No notifications yet"),
            "{}",
            empty.body
        );
    }

    #[sqlx::test(migrations = "../tuita-server/migrations")]
    async fn the_mentions_tab_lists_replies_as_cards(db: PgPool) {
        let router = app(db);
        let mut ana = Browser::new(&router);
        ana.signup("ana").await;
        let mut bob = Browser::new(&router);
        bob.signup("bob").await;
        let id = post(&mut ana, "a post to answer").await;
        act(&mut bob, "/users/ana/follow").await;
        let reply = bob
            .request(
                Method::POST,
                "/api/v1/posts",
                Some(json!({
                    "text": "@ana an answer", "cw": "", "sensitive": false,
                    "mediaIds": [], "replyTo": id
                })),
            )
            .await;
        assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);

        let mentions = ana.get("/notifications?filter=mentions").await;
        assert!(mentions.body.contains("an answer"), "{}", mentions.body);
        assert!(mentions.body.contains("<article"), "{}", mentions.body);
        assert_eq!(
            mentions
                .body
                .matches(r#"data-c="notification-row""#)
                .count(),
            1,
            "only the reply"
        );
        let none = bob.get("/notifications?filter=mentions").await;
        assert!(none.body.contains("No mentions yet"), "{}", none.body);
    }

    #[sqlx::test(migrations = "../tuita-server/migrations")]
    async fn a_fragment_is_only_the_entries(db: PgPool) {
        let router = app(db);
        let mut ana = Browser::new(&router);
        ana.signup("ana").await;
        let mut bob = Browser::new(&router);
        bob.signup("bob").await;
        act(&mut bob, "/users/ana/follow").await;

        let reply = fragment(&mut ana, "/notifications").await;
        assert_eq!(reply.status, StatusCode::OK);
        assert!(!reply.body.contains("<html"), "{}", reply.body);
        assert!(reply.body.contains("followed you"), "{}", reply.body);
        assert!(!reply.body.contains("<h1"), "no page chrome");
    }
}
