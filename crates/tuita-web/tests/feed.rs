//! Home, tag, bookmark, scheduled and list pages: the feed cookie, the pagination fragment
//! contract (`?before=` plus `x-tuita-fragment: 1` answers items and the next link only) and the
//! owner-only controls.
//!
//! Each `#[sqlx::test]` gets a fresh database; `DATABASE_URL` must point at a running Postgres.
//! Run `topcoat asset bundle --package tuita` first (or point `TUITA_TEST_ASSETS` at a bundle).

mod common;

mod feed {
    use axum::http::{Method, StatusCode};
    use serde_json::{Value, json};
    use sqlx::PgPool;

    use super::common::{Browser, Reply, app};

    const FRAGMENT: &str = "x-tuita-fragment";

    async fn api(member: &mut Browser, method: Method, path: &str, body: Value) -> Value {
        let reply = member
            .request(method, &format!("/api/v1{path}"), Some(body))
            .await;
        assert!(reply.status.is_success(), "{path}: {}", reply.body);
        serde_json::from_str(&reply.body).unwrap_or(Value::Null)
    }

    async fn post(member: &mut Browser, text: &str) -> Value {
        api(
            member,
            Method::POST,
            "/posts",
            json!({"text": text, "cw": "", "sensitive": false, "mediaIds": []}),
        )
        .await
    }

    async fn fragment(member: &mut Browser, path: &str) -> Reply {
        member.headers.push((FRAGMENT.to_owned(), "1".to_owned()));
        let reply = member.get(path).await;
        member.headers.retain(|(name, _)| name != FRAGMENT);
        reply
    }

    /// The `href` of the sentinel link in `html`.
    fn next_href(html: &str) -> String {
        let at = html.find("data-more=\"\"").expect("a sentinel");
        let rest = &html[at..];
        let start = rest.find("href=\"").expect("an href") + 6;
        let end = rest[start..].find('"').expect("it closes");
        rest[start..start + end].replace("&amp;", "&")
    }

    #[sqlx::test(migrations = "../tuita-server/migrations")]
    async fn a_visit_with_a_feed_sets_the_cookie_and_a_bare_visit_reads_it(db: PgPool) {
        let mut member = Browser::new(&app(db));
        member.signup("ada").await;

        let first = member.get("/").await;
        assert_eq!(first.status, StatusCode::OK, "{}", first.body);
        assert!(
            first.body.contains(r#"data-feed="foryou""#),
            "{}",
            first.body
        );
        assert!(
            !first.header_all("set-cookie").contains("tuita_feed"),
            "a bare visit stores nothing"
        );

        let following = member.get("/?feed=following").await;
        assert!(
            following.body.contains(r#"data-feed="following""#),
            "{}",
            following.body
        );
        let cookie = following.header_all("set-cookie");
        assert!(
            cookie.contains("tuita_feed=following")
                && cookie.contains("HttpOnly")
                && cookie.contains("SameSite=Lax")
                && cookie.contains("Path=/")
                && !cookie.contains("Max-Age"),
            "a session cookie: {cookie}"
        );

        let again = member.get("/").await;
        assert!(
            again.body.contains(r#"data-feed="following""#),
            "the cookie decides: {}",
            again.body
        );
        let empty = &again.body;
        assert!(empty.contains("Your circle is quiet"), "{empty}");
        assert!(empty.contains(r#"href="/explore""#), "{empty}");

        let back = member.get("/?feed=foryou").await;
        assert!(back.body.contains(r#"data-feed="foryou""#), "{}", back.body);
        assert!(back.header_all("set-cookie").contains("tuita_feed=foryou"));
        let junk = member.get("/?feed=popular").await;
        assert!(
            !junk.header_all("set-cookie").contains("tuita_feed=popular"),
            "an unknown feed is never stored"
        );
    }

    #[sqlx::test(migrations = "../tuita-server/migrations")]
    async fn the_home_page_has_both_tabs_the_post_box_and_the_pill_slot(db: PgPool) {
        let mut member = Browser::new(&app(db));
        member.signup("ada").await;
        let html = member.get("/?feed=following").await.body;
        assert!(html.contains(r#"role="tablist""#) && html.contains(r#"aria-label="Timeline""#));
        assert!(html.contains(r#"href="/?feed=foryou""#), "{html}");
        assert!(html.contains(r#"href="/?feed=following""#), "{html}");
        assert!(html.contains(r#"data-slot="pill""#), "{html}");
        assert!(html.contains("data-feed-body"), "{html}");
        assert!(html.contains(r#"data-island="composer""#), "{html}");
        assert!(html.contains("<title>Home · tuita</title>"), "{html}");
    }

    #[sqlx::test(migrations = "../tuita-server/migrations")]
    async fn a_fragment_of_the_first_page_is_one_feed_body(db: PgPool) {
        let mut member = Browser::new(&app(db));
        member.signup("ada").await;
        let reply = fragment(&mut member, "/?feed=foryou").await;
        assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
        let html = reply.body.trim_start();
        assert!(html.starts_with("<div data-feed-body"), "{html}");
        assert!(
            html.contains(r#"data-since="01"#),
            "the pill counts from the moment the list was built (a v7 id): {html}"
        );
        assert!(!html.contains("<html") && !html.contains("data-c=\"page-bar\""));
        assert!(
            !reply.header_all("set-cookie").contains("tuita_feed"),
            "a fragment does not touch the cookie"
        );
    }

    #[sqlx::test(migrations = "../tuita-server/migrations")]
    async fn a_tag_page_ends_in_a_sentinel_and_the_next_page_is_only_items(db: PgPool) {
        let mut member = Browser::new(&app(db));
        member.signup("ada").await;
        for n in 0..21 {
            post(&mut member, &format!("number {n} #paged")).await;
        }

        let page = member.get("/tags/Paged").await;
        assert_eq!(page.status, StatusCode::OK, "{}", page.body);
        assert!(page.body.contains("Posts tagged"), "{}", page.body);
        assert!(
            page.body.contains("<title>#paged · tuita</title>"),
            "{}",
            page.body
        );
        assert_eq!(page.body.matches("<article").count(), 20, "one page of 20");
        let href = next_href(&page.body);
        assert!(href.starts_with("/tags/Paged?before="), "{href}");

        let more = fragment(&mut member, &href).await;
        assert_eq!(more.status, StatusCode::OK, "{}", more.body);
        assert_eq!(more.body.matches("<article").count(), 1, "{}", more.body);
        assert!(!more.body.contains("<html") && !more.body.contains("data-more"));
        assert!(more.body.contains(r#"role="listitem""#));

        let bare = member.get(&href).await;
        assert!(bare.body.contains("<html") && bare.body.contains("data-c=\"page-bar\""));
    }

    #[sqlx::test(migrations = "../tuita-server/migrations")]
    async fn a_tag_nobody_used_shows_the_empty_state(db: PgPool) {
        let mut member = Browser::new(&app(db));
        member.signup("ada").await;
        let html = member.get("/tags/nothing").await.body;
        assert!(html.contains("No posts with"), "{html}");
        assert!(html.contains("Be the first to use it."), "{html}");
        assert!(!html.contains("data-more"), "{html}");
    }

    #[sqlx::test(migrations = "../tuita-server/migrations")]
    async fn bookmarks_has_two_tabs_and_lists_the_saved_posts(db: PgPool) {
        let mut member = Browser::new(&app(db));
        member.signup("ada").await;
        let made = post(&mut member, "keep this one").await;
        let id = made["id"].as_str().expect("an id").to_owned();
        api(
            &mut member,
            Method::POST,
            &format!("/posts/{id}/bookmark"),
            json!({}),
        )
        .await;

        let saved = member.get("/bookmarks").await.body;
        assert!(saved.contains("keep this one"), "{saved}");
        assert!(
            saved.contains(r#"aria-label="History sections""#),
            "{saved}"
        );
        assert!(saved.contains(r#"href="/bookmarks?tab=likes""#), "{saved}");

        let likes = member.get("/bookmarks?tab=likes").await.body;
        assert!(!likes.contains("keep this one"), "{likes}");
        assert!(likes.contains("No likes yet"), "{likes}");
    }

    #[sqlx::test(migrations = "../tuita-server/migrations")]
    async fn the_scheduled_page_lists_a_post_with_its_actions(db: PgPool) {
        let mut member = Browser::new(&app(db));
        member.signup("ada").await;
        let empty = member.get("/scheduled").await.body;
        assert!(empty.contains("Nothing scheduled"), "{empty}");

        let at = (time::OffsetDateTime::now_utc() + time::Duration::hours(2))
            .format(&time::format_description::well_known::Rfc3339)
            .expect("a timestamp");
        api(
            &mut member,
            Method::POST,
            "/scheduled",
            json!({"text": "Out later", "cw": "", "sensitive": false, "mediaIds": [], "publishAt": at}),
        )
        .await;

        let html = member.get("/scheduled").await.body;
        assert!(html.contains("Out later"), "{html}");
        assert!(html.contains(r#"aria-label="Scheduled posts""#), "{html}");
        assert!(html.contains(r#"data-act="sched-publish""#), "{html}");
        assert!(html.contains(r#"id="reschedule""#), "{html}");
        assert!(html.contains(r#"id="confirm-scheduled-cancel""#), "{html}");

        let part = fragment(&mut member, "/scheduled").await.body;
        assert!(
            part.trim_start().starts_with("<div data-scheduled-body"),
            "{part}"
        );
        assert!(
            !part.contains("<html") && !part.contains("reschedule-form"),
            "{part}"
        );
    }

    #[sqlx::test(migrations = "../tuita-server/migrations")]
    async fn lists_are_created_shown_and_managed_by_their_owner(db: PgPool) {
        let router = app(db);
        let mut owner = Browser::new(&router);
        owner.signup("ada").await;
        let mut other = Browser::new(&router);
        other.signup("bob").await;

        let none = owner.get("/lists").await.body;
        assert!(none.contains("No lists yet"), "{none}");
        assert!(none.contains(r#"data-act="list-new""#), "{none}");

        let made = api(
            &mut owner,
            Method::POST,
            "/lists",
            json!({"name": "Friends", "description": "Close ones", "private": false}),
        )
        .await;
        let id = made["id"].as_str().expect("an id").to_owned();
        api(
            &mut owner,
            Method::POST,
            &format!("/lists/{id}/members/bob"),
            json!({}),
        )
        .await;

        let index = owner.get("/lists").await.body;
        assert!(
            index.contains("Friends") && index.contains(&format!("/lists/{id}")),
            "{index}"
        );

        let page = owner.get(&format!("/lists/{id}")).await;
        assert_eq!(page.status, StatusCode::OK, "{}", page.body);
        assert!(
            page.body.contains("<title>Friends · tuita</title>"),
            "{}",
            page.body
        );
        assert!(
            page.body.contains(r#"data-act="list-edit""#),
            "{}",
            page.body
        );
        assert!(page.body.contains("Nothing here yet"), "{}", page.body);

        let members = owner.get(&format!("/lists/{id}/members")).await.body;
        assert!(members.contains("@bob"), "{members}");
        assert!(
            members.contains(r#"data-act="list-remove-member""#),
            "{members}"
        );
        assert!(members.contains(r#"data-count="1""#), "{members}");

        let seen = other.get(&format!("/lists/{id}/members")).await.body;
        assert!(seen.contains("@bob"), "{seen}");
        assert!(!seen.contains("list-remove-member"), "{seen}");
        let theirs = other.get(&format!("/lists/{id}")).await.body;
        assert!(
            !theirs.contains("list-edit") && !theirs.contains("list-delete"),
            "{theirs}"
        );
    }

    #[sqlx::test(migrations = "../tuita-server/migrations")]
    async fn a_list_that_does_not_exist_says_so_under_a_back_bar(db: PgPool) {
        let mut member = Browser::new(&app(db));
        member.signup("ada").await;
        let reply = member
            .get("/lists/00000000-0000-0000-0000-0000000000aa")
            .await;
        assert_eq!(reply.status, StatusCode::NOT_FOUND, "{}", reply.body);
        assert!(reply.body.contains("open this list"), "{}", reply.body);
    }

    #[sqlx::test(migrations = "../tuita-server/migrations")]
    async fn the_editor_dialog_is_a_fragment_for_a_new_and_an_existing_list(db: PgPool) {
        let router = app(db);
        let mut member = Browser::new(&router);
        member.signup("ada").await;
        let blank = member.get("/_tuita/dialog/list-editor").await;
        assert_eq!(blank.status, StatusCode::OK, "{}", blank.body);
        assert!(blank.body.starts_with("<dialog") && blank.body.contains(">New list<"));
        let made = api(
            &mut member,
            Method::POST,
            "/lists",
            json!({"name": "Mine", "description": "", "private": true}),
        )
        .await;
        let id = made["id"].as_str().expect("an id");
        let edit = member
            .get(&format!("/_tuita/dialog/list-editor/{id}"))
            .await;
        assert!(
            edit.body.contains(r#"value="Mine""#) && edit.body.contains(r#"aria-checked="true""#),
            "{}",
            edit.body
        );

        let mut guest = Browser::new(&router);
        let denied = guest.get("/_tuita/dialog/list-editor").await;
        assert_eq!(denied.status, StatusCode::UNAUTHORIZED);
        assert!(denied.body.is_empty());
    }

    #[sqlx::test(migrations = "../tuita-server/migrations")]
    async fn the_new_posts_pill_is_empty_for_guests_and_nothing_new(db: PgPool) {
        let router = app(db);
        let mut guest = Browser::new(&router);
        let denied = guest.get("/_tuita/new-posts?feed=foryou").await;
        assert_eq!(denied.status, StatusCode::UNAUTHORIZED);

        let mut member = Browser::new(&router);
        member.signup("ada").await;
        let none = member
            .get("/_tuita/new-posts?feed=following&since=00000000-0000-0000-0000-000000000000")
            .await;
        assert_eq!(none.status, StatusCode::NO_CONTENT, "{}", none.body);
        let bad = member
            .get("/_tuita/new-posts?feed=following&since=nope")
            .await;
        assert_eq!(bad.status, StatusCode::BAD_REQUEST);
    }

    #[sqlx::test(migrations = "../tuita-server/migrations")]
    async fn the_pill_counts_what_followed_accounts_posted(db: PgPool) {
        let router = app(db);
        let mut reader = Browser::new(&router);
        reader.signup("ada").await;
        let mut writer = Browser::new(&router);
        writer.signup("bob").await;
        api(&mut reader, Method::POST, "/users/bob/follow", json!({})).await;
        post(&mut writer, "first").await;
        post(&mut writer, "second").await;
        let pill = reader
            .get("/_tuita/new-posts?feed=following&since=00000000-0000-0000-0000-000000000000")
            .await;
        assert_eq!(pill.status, StatusCode::OK, "{}", pill.body);
        assert!(
            pill.body.contains("Show") && pill.body.contains("new posts"),
            "{}",
            pill.body
        );
        assert!(pill.body.contains(r#"class="pill""#), "{}", pill.body);
    }
}
