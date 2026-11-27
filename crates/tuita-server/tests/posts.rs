//! Posts, reactions and threads over HTTP.
#![allow(missing_docs)]

mod common;

use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

use common::{Browser, Person, app};

async fn count(db: &PgPool, sql: &'static str, bind: Uuid) -> i64 {
    sqlx::query_scalar(sql)
        .bind(bind)
        .fetch_one(db)
        .await
        .expect("count query")
}

async fn posts_count(db: &PgPool, user: Uuid) -> i64 {
    count(
        db,
        "select posts_count::bigint from users where id = $1",
        user,
    )
    .await
}

async fn post_count(db: &PgPool, post: &str, column: &'static str) -> i64 {
    let sql = match column {
        "likes" => "select like_count::bigint from posts where id = $1",
        "reposts" => "select repost_count::bigint from posts where id = $1",
        "replies" => "select reply_count::bigint from posts where id = $1",
        "quotes" => "select quote_count::bigint from posts where id = $1",
        _ => "select views from posts where id = $1",
    };
    count(db, sql, post.parse().expect("uuid")).await
}

async fn notifications(db: &PgPool, user: Uuid, kind: &'static str) -> i64 {
    let n: i64 =
        sqlx::query_scalar("select count(*) from notifications where user_id = $1 and kind = $2")
            .bind(user)
            .bind(kind)
            .fetch_one(db)
            .await
            .expect("count notifications");
    n
}

async fn block(db: &PgPool, user: Uuid, target: Uuid) {
    sqlx::query("insert into blocks (user_id, target_id) values ($1, $2)")
        .bind(user)
        .bind(target)
        .execute(db)
        .await
        .expect("block");
}

async fn mute(db: &PgPool, user: Uuid, target: Uuid) {
    sqlx::query("insert into mutes (user_id, target_id) values ($1, $2)")
        .bind(user)
        .bind(target)
        .execute(db)
        .await
        .expect("mute");
}

async fn follow(db: &PgPool, follower: Uuid, followee: Uuid) {
    sqlx::query("insert into follows (follower_id, followee_id) values ($1, $2)")
        .bind(follower)
        .bind(followee)
        .execute(db)
        .await
        .expect("follow");
}

async fn say(browser: &mut Browser, body: Value) -> String {
    let post = browser.post("/posts", body).await.ok();
    post["id"].as_str().expect("post id").to_owned()
}

async fn text(browser: &mut Browser, text: &str) -> String {
    say(browser, json!({ "text": text })).await
}

fn ids(list: &Value) -> Vec<&str> {
    list.as_array()
        .expect("array")
        .iter()
        .map(|post| post["id"].as_str().expect("id"))
        .collect()
}

#[sqlx::test(migrations = "./migrations")]
async fn creating_a_post_stores_tags_mentions_and_counts_link_length(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let bob = app.person(&db, "bob").await;
    let _cara = app.person(&db, "cara").await;

    let post = ana
        .browser
        .post(
            "/posts",
            json!({ "text": "  Hi @bob @BOB @ana @ghost @cara@other.example #Rust #rust  ", "sensitive": true }),
        )
        .await
        .ok();
    assert_eq!(
        post["text"],
        "Hi @bob @BOB @ana @ghost @cara@other.example #Rust #rust"
    );
    assert_eq!(post["sensitive"], true);
    assert_eq!(post["author"]["handle"], "ana");
    assert_eq!(post["replyPolicy"], "everyone");
    assert_eq!(post["counts"]["likes"], 0);
    assert_eq!(post["replyTo"], Value::Null);
    assert_eq!(post["client"], "linux");
    let id: Uuid = post["id"].as_str().expect("id").parse().expect("uuid");

    let tags: Vec<String> =
        sqlx::query_scalar("select tag from post_tags where post_id = $1 order by tag")
            .bind(id)
            .fetch_all(&db)
            .await
            .expect("tags");
    assert_eq!(tags, ["rust"]);
    let mentioned: Vec<Uuid> =
        sqlx::query_scalar("select user_id from mentions where post_id = $1")
            .bind(id)
            .fetch_all(&db)
            .await
            .expect("mentions");
    assert_eq!(mentioned, [bob.id]);
    assert_eq!(notifications(&db, bob.id, "mention").await, 1);
    assert_eq!(posts_count(&db, ana.id).await, 1);

    let url = format!("https://example.com/{}", "a".repeat(200));
    let long = format!("{} {url}", "x".repeat(470));
    let ok = ana
        .browser
        .post("/posts", json!({ "text": long }))
        .await
        .ok();
    assert_eq!(ok["richText"].as_array().map(Vec::len), Some(2));
    assert_eq!(ok["richText"][1]["type"], "link");
}

#[sqlx::test(migrations = "./migrations")]
async fn client_is_hidden_when_the_author_turns_it_off(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    sqlx::query("update users set show_client = false where id = $1")
        .bind(ana.id)
        .execute(&db)
        .await
        .expect("update");
    let post = ana
        .browser
        .post("/posts", json!({ "text": "hi" }))
        .await
        .ok();
    assert_eq!(post["client"], Value::Null);
}

#[sqlx::test(migrations = "./migrations")]
async fn rejects_invalid_post_bodies(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let b = &mut ana.browser;

    b.post("/posts", json!({ "text": "x".repeat(501) }))
        .await
        .err(
            StatusCode::BAD_REQUEST,
            "Post must be at most 500 characters.",
        );
    b.post("/posts", json!({ "text": "x".repeat(500) }))
        .await
        .ok();
    b.post("/posts", json!({ "text": "hi", "cw": "c".repeat(101) }))
        .await
        .err(
            StatusCode::BAD_REQUEST,
            "Content warning must be at most 100 characters.",
        );
    let post = b
        .post(
            "/posts",
            json!({ "text": "hi", "cw": format!("  {}  ", "c".repeat(100)) }),
        )
        .await
        .ok();
    assert_eq!(post["cw"], "c".repeat(100));
    b.post("/posts", json!({ "text": "   " }))
        .await
        .err(StatusCode::BAD_REQUEST, "Write something first.");
    b.post("/posts", json!({}))
        .await
        .err(StatusCode::BAD_REQUEST, "Write something first.");
    b.post(
        "/posts",
        json!({ "text": "hi", "mediaIds": [Uuid::now_v7().to_string()] }),
    )
    .await
    .err(
        StatusCode::BAD_REQUEST,
        "That upload wasn’t found, or it is already attached to a post. Please upload it again.",
    );
    b.post("/posts", json!({ "text": "hi", "replyPolicy": "nobody" }))
        .await
        .err(StatusCode::BAD_REQUEST, "Invalid JSON.");
    b.post(
        "/posts",
        json!({ "text": "hi", "replyTo": Uuid::now_v7().to_string() }),
    )
    .await
    .err(StatusCode::NOT_FOUND, "Post not found.");
    b.post(
        "/posts",
        json!({ "text": "hi", "quoteOf": Uuid::now_v7().to_string() }),
    )
    .await
    .err(StatusCode::NOT_FOUND, "Post not found.");
    assert_eq!(posts_count(&db, ana.id).await, 2);
}

#[sqlx::test(migrations = "./migrations")]
async fn replying_updates_counters_and_notifies_once(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut bob = app.person(&db, "bob").await;
    let root = text(&mut ana.browser, "root").await;

    let reply = bob
        .browser
        .post("/posts", json!({ "text": "re @ana", "replyTo": root }))
        .await
        .ok();
    assert_eq!(reply["replyTo"]["id"], root);
    assert_eq!(reply["replyTo"]["handle"], "ana");
    let reply_id = reply["id"].as_str().expect("id").to_owned();
    assert_eq!(post_count(&db, &root, "replies").await, 1);
    assert_eq!(notifications(&db, ana.id, "reply").await, 1);
    assert_eq!(notifications(&db, ana.id, "mention").await, 0);

    let deep = say(
        &mut ana.browser,
        json!({ "text": "deep", "replyTo": reply_id }),
    )
    .await;
    let root_id: Uuid = root.parse().expect("uuid");
    let stored: Option<Uuid> = sqlx::query_scalar("select root_id from posts where id = $1")
        .bind(deep.parse::<Uuid>().expect("uuid"))
        .fetch_one(&db)
        .await
        .expect("root");
    assert_eq!(stored, Some(root_id));
    assert_eq!(notifications(&db, bob.id, "reply").await, 1);
    assert_eq!(post_count(&db, &root, "replies").await, 1);
    assert_eq!(post_count(&db, &reply_id, "replies").await, 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn quoting_embeds_the_post_and_counts_it(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut bob = app.person(&db, "bob").await;
    let original = text(&mut ana.browser, "original").await;

    let quote = bob
        .browser
        .post(
            "/posts",
            json!({ "text": "look @ana", "quoteOf": original }),
        )
        .await
        .ok();
    assert_eq!(quote["quote"]["id"], original);
    assert_eq!(quote["quote"]["available"], true);
    assert_eq!(quote["quote"]["text"], "original");
    assert_eq!(quote["quote"]["author"]["handle"], "ana");
    assert_eq!(post_count(&db, &original, "quotes").await, 1);
    assert_eq!(notifications(&db, ana.id, "quote").await, 1);
    assert_eq!(notifications(&db, ana.id, "mention").await, 0);
    assert_eq!(posts_count(&db, bob.id).await, 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn reply_policies_limit_who_can_answer(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut bob = app.person(&db, "bob").await;
    let mut cara = app.person(&db, "cara").await;
    let limited = "The author limited who can reply to this post.";

    let following = say(
        &mut ana.browser,
        json!({ "text": "followers only", "replyPolicy": "following" }),
    )
    .await;
    let mentioned = say(
        &mut ana.browser,
        json!({ "text": "hey @cara", "replyPolicy": "mentioned" }),
    )
    .await;

    bob.browser
        .post("/posts", json!({ "text": "x", "replyTo": following }))
        .await
        .err(StatusCode::FORBIDDEN, limited);
    follow(&db, ana.id, bob.id).await;
    bob.browser
        .post("/posts", json!({ "text": "x", "replyTo": following }))
        .await
        .ok();
    ana.browser
        .post("/posts", json!({ "text": "mine", "replyTo": following }))
        .await
        .ok();

    bob.browser
        .post("/posts", json!({ "text": "x", "replyTo": mentioned }))
        .await
        .err(StatusCode::FORBIDDEN, limited);
    let view = bob.browser.get(&format!("/posts/{mentioned}")).await.ok();
    assert_eq!(view["post"]["canReply"], false);
    let reply = cara
        .browser
        .post("/posts", json!({ "text": "x", "replyTo": mentioned }))
        .await
        .ok();
    assert_eq!(reply["replyTo"]["id"], mentioned);
    assert_eq!(post_count(&db, &following, "replies").await, 2);
    assert_eq!(post_count(&db, &mentioned, "replies").await, 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn blocked_users_cannot_reply_quote_or_react(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut bob = app.person(&db, "bob").await;
    let post = text(&mut ana.browser, "hello").await;
    block(&db, ana.id, bob.id).await;
    let msg = "You can’t interact with this post.";

    bob.browser
        .post("/posts", json!({ "text": "x", "replyTo": post }))
        .await
        .err(StatusCode::FORBIDDEN, msg);
    bob.browser
        .post("/posts", json!({ "text": "x", "quoteOf": post }))
        .await
        .err(StatusCode::FORBIDDEN, msg);
    bob.browser
        .post_empty(&format!("/posts/{post}/like"))
        .await
        .err(StatusCode::FORBIDDEN, msg);
    bob.browser
        .post_empty(&format!("/posts/{post}/repost"))
        .await
        .err(StatusCode::FORBIDDEN, msg);
    bob.browser
        .get(&format!("/posts/{post}"))
        .await
        .err(StatusCode::NOT_FOUND, "This post is unavailable.");
    assert_eq!(post_count(&db, &post, "likes").await, 0);
    assert_eq!(posts_count(&db, bob.id).await, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn thread_orders_replies_and_splits_hidden_ones(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut bob = app.person(&db, "bob").await;
    let mut cara = app.person(&db, "cara").await;
    let mut dan = app.person(&db, "dan").await;

    let grand = text(&mut ana.browser, "grand").await;
    let post = say(
        &mut bob.browser,
        json!({ "text": "post", "replyTo": grand }),
    )
    .await;
    let b1 = say(&mut bob.browser, json!({ "text": "b1", "replyTo": post })).await;
    let c1 = say(&mut cara.browser, json!({ "text": "c1", "replyTo": post })).await;
    let a1 = say(&mut ana.browser, json!({ "text": "a1", "replyTo": post })).await;
    let d1 = say(&mut dan.browser, json!({ "text": "d1", "replyTo": post })).await;
    let gone = say(&mut dan.browser, json!({ "text": "gone", "replyTo": post })).await;
    sqlx::query("update posts set like_count = 5 where id = $1")
        .bind(c1.parse::<Uuid>().expect("uuid"))
        .execute(&db)
        .await
        .expect("likes");
    sqlx::query("update posts set hidden_reply = true where id = $1")
        .bind(d1.parse::<Uuid>().expect("uuid"))
        .execute(&db)
        .await
        .expect("hide");
    dan.browser.delete(&format!("/posts/{gone}")).await.ok();

    let thread = cara.browser.get(&format!("/posts/{post}")).await.ok();
    assert_eq!(thread["post"]["id"], post);
    assert_eq!(thread["post"]["author"]["handle"], "bob");
    assert_eq!(ids(&thread["ancestors"]), [grand.as_str()]);
    assert_eq!(
        ids(&thread["replies"]),
        [b1.as_str(), c1.as_str(), a1.as_str()]
    );
    assert_eq!(ids(&thread["hiddenReplies"]), [d1.as_str()]);
    assert_eq!(thread["hiddenReplies"][0]["hiddenReply"], true);

    let nested = cara.browser.get(&format!("/posts/{b1}")).await.ok();
    assert_eq!(ids(&nested["ancestors"]), [grand.as_str(), post.as_str()]);
    assert_eq!(nested["replies"], json!([]));

    // Opening a thread must not count a view: views feed the For you ranking, and counting
    // them needs the deduplicated, rate-limited `POST /views` that arrives with M2.
    assert_eq!(post_count(&db, &post, "views").await, 0);
    bob.browser.get(&format!("/posts/{post}")).await.ok();
    assert_eq!(post_count(&db, &post, "views").await, 0);

    cara.browser
        .get(&format!("/posts/{}", Uuid::now_v7()))
        .await
        .err(StatusCode::NOT_FOUND, "Post not found.");
    cara.browser
        .get("/posts/not-a-uuid")
        .await
        .err(StatusCode::NOT_FOUND, "Post not found.");
    cara.browser
        .get(&format!("/posts/{gone}"))
        .await
        .err(StatusCode::NOT_FOUND, "Post not found.");
}

#[sqlx::test(migrations = "./migrations")]
async fn a_muted_author_still_opens_but_a_suspended_one_does_not(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut bob = app.person(&db, "bob").await;
    let post = text(&mut ana.browser, "hello").await;
    mute(&db, bob.id, ana.id).await;

    let thread = bob.browser.get(&format!("/posts/{post}")).await.ok();
    assert_eq!(thread["post"]["id"], post);

    sqlx::query("update users set suspended_at = now() where id = $1")
        .bind(ana.id)
        .execute(&db)
        .await
        .expect("suspend");
    bob.browser
        .get(&format!("/posts/{post}"))
        .await
        .err(StatusCode::NOT_FOUND, "This post is unavailable.");
}

#[sqlx::test(migrations = "./migrations")]
async fn deleting_clears_everything_attached(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut bob = app.person(&db, "bob").await;
    let mut cara = app.person(&db, "cara").await;

    let parent = text(&mut bob.browser, "parent").await;
    let quoted = text(&mut bob.browser, "quoted").await;
    let post = say(
        &mut ana.browser,
        json!({ "text": "mine #tag @bob", "replyTo": parent, "quoteOf": quoted }),
    )
    .await;
    ana.browser
        .post_empty(&format!("/posts/{post}/pin"))
        .await
        .ok();
    cara.browser
        .post_empty(&format!("/posts/{post}/like"))
        .await
        .ok();
    cara.browser
        .post_empty(&format!("/posts/{post}/repost"))
        .await
        .ok();
    cara.browser
        .post_empty(&format!("/posts/{post}/bookmark"))
        .await
        .ok();
    assert_eq!(post_count(&db, &parent, "replies").await, 1);
    assert_eq!(post_count(&db, &quoted, "quotes").await, 1);

    cara.browser
        .delete(&format!("/posts/{post}"))
        .await
        .err(StatusCode::FORBIDDEN, "You can only delete your own posts.");
    assert_eq!(
        ana.browser.delete(&format!("/posts/{post}")).await.ok(),
        json!({ "ok": true })
    );

    let pid: Uuid = post.parse().expect("uuid");
    assert_eq!(posts_count(&db, ana.id).await, 0);
    assert_eq!(post_count(&db, &parent, "replies").await, 0);
    assert_eq!(post_count(&db, &quoted, "quotes").await, 0);
    for sql in [
        "select count(*) from likes where post_id = $1",
        "select count(*) from reposts where post_id = $1",
        "select count(*) from bookmarks where post_id = $1",
        "select count(*) from notifications where post_id = $1",
        "select count(*) from post_tags where post_id = $1",
        "select count(*) from mentions where post_id = $1",
    ] {
        assert_eq!(count(&db, sql, pid).await, 0, "{sql}");
    }
    let pinned: Option<Uuid> = sqlx::query_scalar("select pinned_post_id from users where id = $1")
        .bind(ana.id)
        .fetch_one(&db)
        .await
        .expect("pin");
    assert_eq!(pinned, None);
    let (text, deleted): (String, bool) =
        sqlx::query_as("select text, deleted_at is not null from posts where id = $1")
            .bind(pid)
            .fetch_one(&db)
            .await
            .expect("row");
    assert_eq!((text.as_str(), deleted), ("", true));

    ana.browser
        .delete(&format!("/posts/{post}"))
        .await
        .err(StatusCode::NOT_FOUND, "Post not found.");
    assert_eq!(posts_count(&db, ana.id).await, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn staff_may_delete_any_post(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut mod_ = app.person(&db, "mody").await;
    sqlx::query("update users set role = 'moderator' where id = $1")
        .bind(mod_.id)
        .execute(&db)
        .await
        .expect("role");
    let post = text(&mut ana.browser, "bad").await;
    // The role is read per request, so the already signed-in session sees it.
    mod_.browser.delete(&format!("/posts/{post}")).await.ok();
    assert_eq!(posts_count(&db, ana.id).await, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn reactions_are_idempotent_and_counted_exactly(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut bob = app.person(&db, "bob").await;
    let post = text(&mut ana.browser, "hello").await;

    for kind in ["like", "repost", "bookmark"] {
        let url = format!("/posts/{post}/{kind}");
        let first = bob.browser.post_empty(&url).await.ok();
        let again = bob.browser.post_empty(&url).await.ok();
        assert_eq!(first, again, "{kind}");
    }
    let view = bob.browser.get(&format!("/posts/{post}")).await.ok();
    assert_eq!(
        view["post"]["viewer"],
        json!({ "liked": true, "reposted": true, "bookmarked": true })
    );
    assert_eq!(post_count(&db, &post, "likes").await, 1);
    assert_eq!(post_count(&db, &post, "reposts").await, 1);
    assert_eq!(notifications(&db, ana.id, "like").await, 1);
    assert_eq!(notifications(&db, ana.id, "repost").await, 1);

    let liked = bob
        .browser
        .post_empty(&format!("/posts/{post}/like"))
        .await
        .ok();
    assert_eq!(liked["counts"]["likes"], 1);
    assert_eq!(liked["viewer"]["liked"], true);

    for kind in ["like", "repost", "bookmark"] {
        let url = format!("/posts/{post}/{kind}");
        bob.browser.delete(&url).await.ok();
        let again = bob.browser.delete(&url).await.ok();
        assert_eq!(again["id"], post, "{kind}");
    }
    assert_eq!(post_count(&db, &post, "likes").await, 0);
    assert_eq!(post_count(&db, &post, "reposts").await, 0);
    let after = bob.browser.get(&format!("/posts/{post}")).await.ok();
    assert_eq!(
        after["post"]["viewer"],
        json!({ "liked": false, "reposted": false, "bookmarked": false })
    );

    // Unlike then like again keeps a single notification.
    bob.browser
        .post_empty(&format!("/posts/{post}/like"))
        .await
        .ok();
    assert_eq!(post_count(&db, &post, "likes").await, 1);
    assert_eq!(notifications(&db, ana.id, "like").await, 1);

    let missing = Uuid::now_v7();
    for method in ["post", "delete"] {
        for kind in ["like", "repost", "bookmark"] {
            let url = format!("/posts/{missing}/{kind}");
            let reply = if method == "post" {
                bob.browser.post_empty(&url).await
            } else {
                bob.browser.delete(&url).await
            };
            reply.err(StatusCode::NOT_FOUND, "Post not found.");
        }
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn no_notification_for_self_or_muted_actors(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut bob = app.person(&db, "bob").await;
    let post = text(&mut ana.browser, "hello").await;

    ana.browser
        .post_empty(&format!("/posts/{post}/like"))
        .await
        .ok();
    mute(&db, ana.id, bob.id).await;
    let liked = bob
        .browser
        .post_empty(&format!("/posts/{post}/like"))
        .await
        .ok();
    assert_eq!(liked["counts"]["likes"], 2);
    assert_eq!(notifications(&db, ana.id, "like").await, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn pinning_follows_the_rules(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut bob = app.person(&db, "bob").await;
    let one = text(&mut ana.browser, "one").await;
    let two = text(&mut ana.browser, "two").await;

    bob.browser
        .post_empty(&format!("/posts/{one}/pin"))
        .await
        .err(StatusCode::FORBIDDEN, "You can only pin your own posts.");
    ana.browser
        .post_empty(&format!("/posts/{}/pin", Uuid::now_v7()))
        .await
        .err(StatusCode::NOT_FOUND, "Post not found.");
    let pinned = |db: PgPool, user: Uuid| async move {
        sqlx::query_scalar::<_, Option<Uuid>>("select pinned_post_id from users where id = $1")
            .bind(user)
            .fetch_one(&db)
            .await
            .expect("pin")
    };
    assert_eq!(
        ana.browser
            .post_empty(&format!("/posts/{one}/pin"))
            .await
            .ok(),
        json!({ "ok": true })
    );
    assert_eq!(
        pinned(db.clone(), ana.id).await,
        Some(one.parse().expect("uuid"))
    );
    ana.browser
        .post_empty(&format!("/posts/{two}/pin"))
        .await
        .ok();
    assert_eq!(
        pinned(db.clone(), ana.id).await,
        Some(two.parse().expect("uuid"))
    );
    assert_eq!(
        ana.browser.delete(&format!("/posts/{one}/pin")).await.ok(),
        json!({ "ok": true })
    );
    assert_eq!(pinned(db.clone(), ana.id).await, None);
    assert_eq!(
        bob.browser.delete(&format!("/posts/{one}/pin")).await.ok(),
        json!({ "ok": true })
    );
}

async fn react_as(people: &mut [&mut Person], post: &str, kind: &str) {
    for person in people {
        person
            .browser
            .post_empty(&format!("/posts/{post}/{kind}"))
            .await
            .ok();
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn likes_and_reposts_lists_page_newest_first_and_hide_blocked(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut bob = app.person(&db, "bob").await;
    let mut cara = app.person(&db, "cara").await;
    let mut dan = app.person(&db, "dan").await;
    let mut eve = app.person(&db, "eve").await;
    let post = text(&mut ana.browser, "hello").await;
    react_as(
        &mut [&mut bob, &mut cara, &mut dan, &mut eve],
        &post,
        "like",
    )
    .await;
    react_as(&mut [&mut bob, &mut cara], &post, "repost").await;
    mute(&db, ana.id, eve.id).await;

    let first = ana
        .browser
        .get(&format!("/posts/{post}/likes?limit=2"))
        .await
        .ok();
    let handles = |page: &Value| -> Vec<String> {
        page["items"]
            .as_array()
            .expect("items")
            .iter()
            .map(|u| u["handle"].as_str().expect("handle").to_owned())
            .collect()
    };
    assert_eq!(handles(&first), ["dan", "cara"]);
    let next = first["next"].as_str().expect("next").to_owned();
    let second = ana
        .browser
        .get(&format!("/posts/{post}/likes?limit=2&before={next}"))
        .await
        .ok();
    assert_eq!(handles(&second), ["bob"]);
    assert_eq!(second["next"], Value::Null);

    let reposts = ana
        .browser
        .get(&format!("/posts/{post}/reposts"))
        .await
        .ok();
    assert_eq!(handles(&reposts), ["cara", "bob"]);

    block(&db, dan.id, ana.id).await;
    let hidden = ana.browser.get(&format!("/posts/{post}/likes")).await.ok();
    assert_eq!(handles(&hidden), ["cara", "bob"]);

    ana.browser
        .get(&format!("/posts/{}/likes", Uuid::now_v7()))
        .await
        .err(StatusCode::NOT_FOUND, "Post not found.");
    ana.browser
        .get(&format!("/posts/{post}/reposts?before=nope"))
        .await
        .err(StatusCode::BAD_REQUEST, "Invalid cursor.");
}

#[sqlx::test(migrations = "./migrations")]
async fn quotes_list_pages_by_id(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut bob = app.person(&db, "bob").await;
    let post = text(&mut ana.browser, "original").await;
    let q1 = say(&mut bob.browser, json!({ "text": "q1", "quoteOf": post })).await;
    let q2 = say(&mut bob.browser, json!({ "text": "q2", "quoteOf": post })).await;
    let q3 = say(&mut bob.browser, json!({ "text": "q3", "quoteOf": post })).await;
    let q4 = say(&mut bob.browser, json!({ "text": "q4", "quoteOf": post })).await;
    bob.browser.delete(&format!("/posts/{q4}")).await.ok();

    let first = ana
        .browser
        .get(&format!("/posts/{post}/quotes?limit=2"))
        .await
        .ok();
    assert_eq!(ids(&first["items"]), [q3.as_str(), q2.as_str()]);
    let next = first["next"].as_str().expect("next");
    assert_eq!(next, q2);
    let second = ana
        .browser
        .get(&format!("/posts/{post}/quotes?limit=2&before={next}"))
        .await
        .ok();
    assert_eq!(ids(&second["items"]), [q1.as_str()]);
    assert_eq!(second["next"], Value::Null);

    ana.browser
        .get(&format!("/posts/{}/quotes", Uuid::now_v7()))
        .await
        .err(StatusCode::NOT_FOUND, "Post not found.");
}

#[sqlx::test(migrations = "./migrations")]
async fn the_61st_post_in_a_window_is_rate_limited(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    for i in 0..60 {
        ana.browser
            .post("/posts", json!({ "text": format!("p{i}") }))
            .await
            .ok();
    }
    let reply = ana
        .browser
        .post("/posts", json!({ "text": "one more" }))
        .await;
    reply.err(
        StatusCode::TOO_MANY_REQUESTS,
        "Too many requests. Try again in a few minutes.",
    );
    assert!(!reply.header("retry-after").is_empty());
    assert_eq!(posts_count(&db, ana.id).await, 60);
}

#[sqlx::test(migrations = "./migrations")]
async fn every_route_needs_a_session(db: PgPool) {
    let app = app(db.clone());
    let mut anon = app.browser();
    let id = Uuid::now_v7();
    anon.post("/posts", json!({ "text": "hi" }))
        .await
        .err(StatusCode::UNAUTHORIZED, "sign in required");
    anon.get(&format!("/posts/{id}"))
        .await
        .err(StatusCode::UNAUTHORIZED, "sign in required");
    anon.delete(&format!("/posts/{id}"))
        .await
        .err(StatusCode::UNAUTHORIZED, "sign in required");
    anon.post_empty(&format!("/posts/{id}/like"))
        .await
        .err(StatusCode::UNAUTHORIZED, "sign in required");
    anon.get(&format!("/posts/{id}/quotes"))
        .await
        .err(StatusCode::UNAUTHORIZED, "sign in required");
}

#[sqlx::test(migrations = "./migrations")]
async fn reacting_to_a_muted_authors_post_returns_the_post(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut bob = app.person(&db, "bob").await;
    let post = text(&mut ana.browser, "hello cats").await;
    mute(&db, bob.id, ana.id).await;
    sqlx::query("update users set muted_words = array['cats'] where id = $1")
        .bind(bob.id)
        .execute(&db)
        .await
        .expect("muted words");

    for kind in ["like", "repost", "bookmark"] {
        let done = bob
            .browser
            .post_empty(&format!("/posts/{post}/{kind}"))
            .await
            .ok();
        assert_eq!(done["id"], post.as_str(), "{kind}");
        let flag = match kind {
            "like" => "liked",
            "repost" => "reposted",
            _ => "bookmarked",
        };
        assert_eq!(done["viewer"][flag], true, "{kind}");
        let undone = bob
            .browser
            .delete(&format!("/posts/{post}/{kind}"))
            .await
            .ok();
        assert_eq!(undone["viewer"][flag], false, "{kind}");
    }
    assert_eq!(post_count(&db, &post, "likes").await, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_like_racing_a_delete_leaves_no_trace(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut bob = app.person(&db, "bob").await;
    let post = text(&mut ana.browser, "doomed").await;
    let id: Uuid = post.parse().expect("uuid");

    // The delete is in flight (uncommitted) while bob's like arrives.
    let mut tx = db.begin().await.expect("begin");
    sqlx::query("update posts set deleted_at = now(), text = '' where id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await
        .expect("delete");
    let like =
        tokio::spawn(async move { bob.browser.post_empty(&format!("/posts/{post}/like")).await });
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    tx.commit().await.expect("commit");
    like.await
        .expect("task")
        .err(StatusCode::NOT_FOUND, "Post not found.");

    assert_eq!(post_count(&db, &id.to_string(), "likes").await, 0);
    assert_eq!(notifications(&db, ana.id, "like").await, 0);
    let rows: i64 = sqlx::query_scalar("select count(*) from likes where post_id = $1")
        .bind(id)
        .fetch_one(&db)
        .await
        .expect("count");
    assert_eq!(rows, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_reply_racing_a_delete_is_refused(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut bob = app.person(&db, "bob").await;
    let post = text(&mut ana.browser, "doomed").await;
    let id: Uuid = post.parse().expect("uuid");

    let mut tx = db.begin().await.expect("begin");
    sqlx::query("update posts set deleted_at = now(), text = '' where id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await
        .expect("delete");
    let reply = tokio::spawn(async move {
        bob.browser
            .post("/posts", json!({ "text": "too late", "replyTo": post }))
            .await
    });
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    tx.commit().await.expect("commit");
    reply
        .await
        .expect("task")
        .err(StatusCode::NOT_FOUND, "Post not found.");

    let replies: i64 = sqlx::query_scalar("select count(*) from posts where reply_to_id = $1")
        .bind(id)
        .fetch_one(&db)
        .await
        .expect("count");
    assert_eq!(replies, 0);
    assert_eq!(notifications(&db, ana.id, "reply").await, 0);
    assert_eq!(posts_count(&db, bob.id).await, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_remote_post_the_viewer_may_not_read_is_not_found_for_every_interaction(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let remote = Uuid::now_v7();
    sqlx::query(
        "insert into users (id, handle, domain, ap_id) values ($1, 'eve', 'b.example', 'https://b.example/users/eve')",
    )
    .bind(remote)
    .execute(&db)
    .await
    .expect("remote user");
    let mut secrets = Vec::new();
    for visibility in ["followers", "direct"] {
        let post = Uuid::now_v7();
        sqlx::query(
            "insert into posts (id, author_id, text, visibility, ap_id) values ($1, $2, 'secret', $3, $4)",
        )
        .bind(post)
        .bind(remote)
        .bind(visibility)
        .bind(format!("https://b.example/notes/{visibility}"))
        .execute(&db)
        .await
        .expect("remote post");
        sqlx::query(
            "insert into polls (post_id, options, multiple, ends_at, counts)
             values ($1, array['a', 'b'], false, now() + interval '1 day', array[0, 0])",
        )
        .bind(post)
        .execute(&db)
        .await
        .expect("poll");
        secrets.push(post);
    }

    for post in &secrets {
        for kind in ["like", "repost", "bookmark"] {
            ana.browser
                .post_empty(&format!("/posts/{post}/{kind}"))
                .await
                .err(StatusCode::NOT_FOUND, "Post not found.");
        }
        for body in [
            json!({ "text": "r", "replyTo": post }),
            json!({ "text": "q", "quoteOf": post }),
        ] {
            ana.browser
                .post("/posts", body)
                .await
                .err(StatusCode::NOT_FOUND, "Post not found.");
        }
        ana.browser
            .post(&format!("/posts/{post}/vote"), json!({ "choices": [0] }))
            .await
            .err(StatusCode::NOT_FOUND, "This post has no poll.");
        for list in ["likes", "reposts", "quotes"] {
            ana.browser
                .get(&format!("/posts/{post}/{list}"))
                .await
                .err(StatusCode::NOT_FOUND, "Post not found.");
        }
        ana.browser
            .get(&format!("/posts/{post}/edits"))
            .await
            .err(StatusCode::NOT_FOUND, "Post not found.");
    }
    let reactions: i64 = sqlx::query_scalar(
        "select (select count(*) from likes) + (select count(*) from reposts)
              + (select count(*) from bookmarks) + (select count(*) from poll_votes)",
    )
    .fetch_one(&db)
    .await
    .expect("count");
    assert_eq!(reactions, 0);
    assert_eq!(posts_count(&db, ana.id).await, 0);

    follow(&db, ana.id, remote).await;
    let followers_only = secrets[0].to_string();
    ana.browser
        .post_empty(&format!("/posts/{followers_only}/like"))
        .await
        .ok();
    ana.browser
        .get(&format!("/posts/{followers_only}/likes"))
        .await
        .ok();
    ana.browser
        .post("/posts", json!({ "text": "r", "replyTo": followers_only }))
        .await
        .ok();
    let direct = secrets[1].to_string();
    ana.browser
        .post_empty(&format!("/posts/{direct}/like"))
        .await
        .err(StatusCode::NOT_FOUND, "Post not found.");
}
