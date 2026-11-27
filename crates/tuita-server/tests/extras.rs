//! Polls, edits, threads, view counts, hidden replies, scheduling, post media and muted words.
#![allow(missing_docs)]

mod common;

use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

use common::app;

/// Inserts an image upload owned by `owner`, as `POST /media` would leave it.
async fn upload(db: &PgPool, owner: Uuid, kind: &str) -> Uuid {
    let id = Uuid::now_v7();
    let ext = if kind == "video" { "mp4" } else { "webp" };
    sqlx::query(
        "insert into media (id, owner_id, kind, mime, path, bytes, width, height)
         values ($1, $2, $3, $4, $5, 10, 4, 3)",
    )
    .bind(id)
    .bind(owner)
    .bind(kind)
    .bind(if kind == "video" {
        "video/mp4"
    } else {
        "image/webp"
    })
    .bind(format!("{id}.{ext}"))
    .execute(db)
    .await
    .expect("insert media");
    id
}

fn id_of(post: &Value) -> String {
    post["id"].as_str().expect("post id").to_owned()
}

#[sqlx::test(migrations = "./migrations")]
async fn a_post_carries_up_to_four_images_in_order(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut ids = Vec::new();
    for _ in 0..5 {
        ids.push(upload(&db, ana.id, "image").await);
    }
    let five: Vec<String> = ids.iter().map(Uuid::to_string).collect();
    ana.browser
        .post("/posts", json!({ "text": "x", "mediaIds": five }))
        .await
        .err(StatusCode::BAD_REQUEST, "You can attach up to 4 images.");

    let four: Vec<String> = ids[..4].iter().rev().map(Uuid::to_string).collect();
    let post = ana
        .browser
        .post("/posts", json!({ "text": "", "mediaIds": four }))
        .await
        .ok();
    let media = post["media"].as_array().expect("media");
    assert_eq!(media.len(), 4);
    assert_eq!(media[0]["id"], four[0]);
    assert_eq!(media[0]["url"], format!("/media/{}.webp", four[0]).as_str());
    assert_eq!(media[0]["kind"], "image");

    // An upload belongs to one post.
    ana.browser
        .post("/posts", json!({ "text": "again", "mediaIds": [four[0]] }))
        .await
        .err(
            StatusCode::BAD_REQUEST,
            "That upload wasn’t found, or it is already attached to a post. Please upload it again.",
        );
    // Not somebody else's.
    let mut ben = app.person(&db, "ben").await;
    ben.browser
        .post("/posts", json!({ "text": "mine now", "mediaIds": [ids[4].to_string()] }))
        .await
        .err(
            StatusCode::BAD_REQUEST,
            "That upload wasn’t found, or it is already attached to a post. Please upload it again.",
        );
    // The media tab and the single post both show it.
    let id = id_of(&post);
    assert_eq!(
        ana.browser.get(&format!("/posts/{id}")).await.ok()["post"]["media"][3]["id"],
        four[3]
    );
    let tab = ana.browser.get("/users/ana/posts?tab=media").await.ok();
    assert_eq!(tab["items"].as_array().expect("items").len(), 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn media_and_polls_do_not_mix_and_a_video_stands_alone(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let image = upload(&db, ana.id, "image").await.to_string();
    let video = upload(&db, ana.id, "video").await.to_string();
    let poll = json!({ "options": ["a", "b"], "durationHours": 1 });
    ana.browser
        .post(
            "/posts",
            json!({ "text": "q", "mediaIds": [image], "poll": poll }),
        )
        .await
        .err(
            StatusCode::BAD_REQUEST,
            "A post can have images or a poll, not both.",
        );
    ana.browser
        .post(
            "/posts",
            json!({ "text": "q", "mediaIds": [video], "poll": poll }),
        )
        .await
        .err(
            StatusCode::BAD_REQUEST,
            "A post can have a video, images or a poll, but only one of them.",
        );
    ana.browser
        .post("/posts", json!({ "text": "q", "mediaIds": [video, image] }))
        .await
        .err(
            StatusCode::BAD_REQUEST,
            "A post can have a video or images, but only one video and not both.",
        );
    let post = ana
        .browser
        .post("/posts", json!({ "text": "v", "mediaIds": [video] }))
        .await
        .ok();
    assert_eq!(post["media"][0]["kind"], "video");
}

#[sqlx::test(migrations = "./migrations")]
async fn a_poll_takes_votes_once_and_ends(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut ben = app.person(&db, "ben").await;
    let mut cy = app.person(&db, "cyrus").await;

    for (poll, message) in [
        (
            json!({ "options": ["only"], "durationHours": 1 }),
            "Polls need 2 to 4 choices.",
        ),
        (
            json!({ "options": ["a", "A"], "durationHours": 1 }),
            "Poll choices must be different.",
        ),
        (
            json!({ "options": ["a", "b"], "durationHours": 169 }),
            "Polls can run from 1 hour to 7 days.",
        ),
    ] {
        ana.browser
            .post("/posts", json!({ "text": "q", "poll": poll }))
            .await
            .err(StatusCode::BAD_REQUEST, message);
    }
    ana.browser
        .post(
            "/posts",
            json!({ "text": "", "poll": { "options": ["a", "b"], "durationHours": 1 } }),
        )
        .await
        .err(StatusCode::BAD_REQUEST, "Ask a question for your poll.");

    let post = ana
        .browser
        .post(
            "/posts",
            json!({ "text": "Tea or coffee?", "poll": { "options": ["Tea", "Coffee", "Water"], "multiple": false, "durationHours": 24 } }),
        )
        .await
        .ok();
    let id = id_of(&post);
    assert_eq!(
        post["poll"]["options"][1],
        json!({ "text": "Coffee", "votes": 0 })
    );
    assert_eq!(post["poll"]["closed"], false);

    ben.browser
        .post(&format!("/posts/{id}/vote"), json!({ "choices": [] }))
        .await
        .err(StatusCode::BAD_REQUEST, "Pick a choice.");
    ben.browser
        .post(&format!("/posts/{id}/vote"), json!({ "choices": [3] }))
        .await
        .err(StatusCode::BAD_REQUEST, "Pick a choice.");
    ben.browser
        .post(&format!("/posts/{id}/vote"), json!({ "choices": [0, 1] }))
        .await
        .err(StatusCode::BAD_REQUEST, "Pick one choice.");
    let voted = ben
        .browser
        .post(&format!("/posts/{id}/vote"), json!({ "choices": [1] }))
        .await
        .ok();
    assert_eq!(voted["poll"]["options"][1]["votes"], 1);
    assert_eq!(voted["poll"]["total"], 1);
    assert_eq!(voted["poll"]["voters"], 1);
    assert_eq!(voted["poll"]["voted"], json!([1]));
    ben.browser
        .post(&format!("/posts/{id}/vote"), json!({ "choices": [0] }))
        .await
        .err(StatusCode::CONFLICT, "You already voted.");

    // Others see the tally, not a vote of their own.
    let seen = cy.browser.get(&format!("/posts/{id}")).await.ok();
    assert_eq!(seen["post"]["poll"]["options"][1]["votes"], 1);
    assert_eq!(seen["post"]["poll"]["voted"], json!([]));

    // Blocked voters are refused.
    sqlx::query("insert into blocks (user_id, target_id) values ($1, $2)")
        .bind(ana.id)
        .bind(cy.id)
        .execute(&db)
        .await
        .expect("block");
    cy.browser
        .post(&format!("/posts/{id}/vote"), json!({ "choices": [0] }))
        .await
        .err(StatusCode::FORBIDDEN, "You can’t vote on this poll.");

    // Ended polls refuse votes.
    sqlx::query("update polls set ends_at = now() - interval '1 minute' where post_id = $1")
        .bind(id.parse::<Uuid>().expect("uuid"))
        .execute(&db)
        .await
        .expect("end poll");
    let plain = ana
        .browser
        .post("/posts", json!({ "text": "no poll" }))
        .await
        .ok();
    let mut dan = app.person(&db, "dan").await;
    dan.browser
        .post(&format!("/posts/{id}/vote"), json!({ "choices": [0] }))
        .await
        .err(StatusCode::BAD_REQUEST, "This poll has ended.");
    dan.browser
        .post(
            &format!("/posts/{}/vote", id_of(&plain)),
            json!({ "choices": [0] }),
        )
        .await
        .err(StatusCode::NOT_FOUND, "This post has no poll.");
    assert_eq!(
        dan.browser.get(&format!("/posts/{id}")).await.ok()["post"]["poll"]["closed"],
        true
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn a_multiple_choice_poll_counts_every_pick(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut ben = app.person(&db, "ben").await;
    let post = ana
        .browser
        .post(
            "/posts",
            json!({ "text": "Pick", "poll": { "options": ["a", "b", "c"], "multiple": true, "durationHours": 2 } }),
        )
        .await
        .ok();
    let voted = ben
        .browser
        .post(
            &format!("/posts/{}/vote", id_of(&post)),
            json!({ "choices": [0, 2, 2] }),
        )
        .await
        .ok();
    assert_eq!(voted["poll"]["total"], 2);
    assert_eq!(voted["poll"]["voters"], 1);
    assert_eq!(voted["poll"]["voted"], json!([0, 2]));
}

#[sqlx::test(migrations = "./migrations")]
async fn an_edit_keeps_history_and_closes_after_30_minutes(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut ben = app.person(&db, "ben").await;
    let post = ana.browser.post_text("first #one").await;
    let id = id_of(&post);

    ben.browser
        .patch(&format!("/posts/{id}"), json!({ "text": "hijack" }))
        .await
        .err(StatusCode::FORBIDDEN, "You can only edit your own posts.");
    ana.browser
        .patch(&format!("/posts/{id}"), json!({ "text": "   " }))
        .await
        .err(StatusCode::BAD_REQUEST, "A post can’t be empty.");

    let edited = ana
        .browser
        .patch(
            &format!("/posts/{id}"),
            json!({ "text": "second #two", "cw": "spoiler" }),
        )
        .await
        .ok();
    assert_eq!(edited["text"], "second #two");
    assert_eq!(edited["cw"], "spoiler");
    assert!(edited["editedAt"].is_string());
    let tags: Vec<String> = sqlx::query_scalar("select tag from post_tags where post_id = $1")
        .bind(id.parse::<Uuid>().expect("uuid"))
        .fetch_all(&db)
        .await
        .expect("tags");
    assert_eq!(tags, ["two"]);

    // No change, no history row.
    ana.browser
        .patch(
            &format!("/posts/{id}"),
            json!({ "text": "second #two", "cw": "spoiler" }),
        )
        .await
        .ok();
    ana.browser
        .patch(&format!("/posts/{id}"), json!({ "text": "third" }))
        .await
        .ok();
    let history = ben.browser.get(&format!("/posts/{id}/edits")).await.ok();
    assert_eq!(history["current"]["text"], "third");
    let edits = history["edits"].as_array().expect("edits");
    assert_eq!(edits.len(), 2);
    assert_eq!(edits[0]["text"], "second #two");
    assert_eq!(edits[1]["text"], "first #one");
    assert_eq!(edits[1]["at"], post["createdAt"]);

    sqlx::query("update posts set created_at = now() - interval '31 minutes' where id = $1")
        .bind(id.parse::<Uuid>().expect("uuid"))
        .execute(&db)
        .await
        .expect("age post");
    ana.browser
        .patch(&format!("/posts/{id}"), json!({ "text": "too late" }))
        .await
        .err(
            StatusCode::FORBIDDEN,
            "Posts can only be edited for 30 minutes after posting.",
        );
}

#[sqlx::test(migrations = "./migrations")]
async fn a_thread_is_a_chain_of_replies_created_together(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut ben = app.person(&db, "ben").await;
    ana.browser
        .post("/posts/thread", json!({ "posts": [{ "text": "only" }] }))
        .await
        .err(StatusCode::BAD_REQUEST, "A thread needs 2 to 25 posts.");
    let target = ben.browser.post_text("hello").await;
    ana.browser
        .post(
            "/posts/thread",
            json!({ "posts": [{ "text": "1" }, { "text": "2", "replyTo": id_of(&target) }] }),
        )
        .await
        .err(
            StatusCode::BAD_REQUEST,
            "Only the first post of a thread can reply to or quote a post.",
        );
    // One bad post rolls everything back.
    ana.browser
        .post(
            "/posts/thread",
            json!({ "posts": [{ "text": "1" }, { "text": "" }] }),
        )
        .await
        .err(StatusCode::BAD_REQUEST, "Write something first.");
    let none: i64 = sqlx::query_scalar("select count(*) from posts where author_id = $1")
        .bind(ana.id)
        .fetch_one(&db)
        .await
        .expect("count");
    assert_eq!(none, 0);

    let made = ana
        .browser
        .post(
            "/posts/thread",
            json!({ "posts": [{ "text": "one" }, { "text": "two" }, { "text": "three" }] }),
        )
        .await
        .ok();
    let posts = made["posts"].as_array().expect("posts");
    assert_eq!(posts.len(), 3);
    assert_eq!(posts[0]["replyTo"], Value::Null);
    assert_eq!(posts[1]["replyTo"]["id"], posts[0]["id"]);
    assert_eq!(posts[2]["replyTo"]["id"], posts[1]["id"]);
    let roots: Vec<Option<Uuid>> =
        sqlx::query_scalar("select root_id from posts where author_id = $1 order by id")
            .bind(ana.id)
            .fetch_all(&db)
            .await
            .expect("roots");
    let first: Uuid = id_of(&posts[0]).parse().expect("uuid");
    assert_eq!(roots, [None, Some(first), Some(first)]);
    let counts: i64 = sqlx::query_scalar("select posts_count::bigint from users where id = $1")
        .bind(ana.id)
        .fetch_one(&db)
        .await
        .expect("posts_count");
    assert_eq!(counts, 3);
    assert_eq!(posts[0]["counts"]["replies"], 1);
    let thread = ben
        .browser
        .get(&format!("/posts/{}", id_of(&posts[2])))
        .await
        .ok();
    assert_eq!(thread["ancestors"].as_array().expect("ancestors").len(), 2);
}

#[sqlx::test(migrations = "./migrations")]
async fn views_count_once_a_day_and_never_for_the_author(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut ben = app.person(&db, "ben").await;
    let post = ana.browser.post_text("look").await;
    let id = id_of(&post);
    ana.browser
        .post("/views", json!({ "ids": [id] }))
        .await
        .ok();
    for _ in 0..3 {
        ben.browser
            .post(
                "/views",
                json!({ "ids": [id, id, Uuid::now_v7().to_string()] }),
            )
            .await
            .ok();
    }
    let views: i64 = sqlx::query_scalar("select views from posts where id = $1")
        .bind(id.parse::<Uuid>().expect("uuid"))
        .fetch_one(&db)
        .await
        .expect("views");
    assert_eq!(views, 1);
    // The next day counts again.
    sqlx::query("update post_views set day = current_date - 1")
        .execute(&db)
        .await
        .expect("age views");
    ben.browser
        .post("/views", json!({ "ids": [id] }))
        .await
        .ok();
    assert_eq!(
        ben.browser.get(&format!("/posts/{id}")).await.ok()["post"]["counts"]["views"],
        2
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn only_the_parent_author_hides_replies(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut ben = app.person(&db, "ben").await;
    let root = ana.browser.post_text("root").await;
    let reply = ben
        .browser
        .post("/posts", json!({ "text": "rude", "replyTo": id_of(&root) }))
        .await
        .ok();
    let rid = id_of(&reply);
    ben.browser
        .post(&format!("/posts/{rid}/hide"), json!({}))
        .await
        .err(
            StatusCode::FORBIDDEN,
            "You can only hide replies to your own posts.",
        );
    ana.browser
        .post(&format!("/posts/{}/hide", id_of(&root)), json!({}))
        .await
        .err(
            StatusCode::FORBIDDEN,
            "You can only hide replies to your own posts.",
        );
    let hidden = ana
        .browser
        .post_empty(&format!("/posts/{rid}/hide"))
        .await
        .ok();
    assert_eq!(hidden["hiddenReply"], true);
    let thread = ben
        .browser
        .get(&format!("/posts/{}", id_of(&root)))
        .await
        .ok();
    assert_eq!(thread["replies"].as_array().expect("replies").len(), 0);
    assert_eq!(thread["hiddenReplies"].as_array().expect("hidden").len(), 1);
    let shown = ana.browser.delete(&format!("/posts/{rid}/hide")).await.ok();
    assert_eq!(shown["hiddenReply"], false);
}

#[sqlx::test(migrations = "./migrations")]
async fn scheduled_posts_publish_as_their_author(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let soon = time::OffsetDateTime::now_utc() + time::Duration::minutes(5);
    let at = |t: time::OffsetDateTime| {
        t.format(&time::format_description::well_known::Rfc3339)
            .expect("format")
    };
    ana.browser
        .post("/scheduled", json!({ "text": "x", "publishAt": at(time::OffsetDateTime::now_utc() + time::Duration::seconds(10)) }))
        .await
        .err(StatusCode::BAD_REQUEST, "Pick a time at least a minute from now.");
    ana.browser
        .post("/scheduled", json!({ "text": "x", "publishAt": at(time::OffsetDateTime::now_utc() + time::Duration::days(91)) }))
        .await
        .err(StatusCode::BAD_REQUEST, "You can schedule up to 90 days ahead.");
    let other = ana.browser.post_text("other").await;
    ana.browser
        .post(
            "/scheduled",
            json!({ "text": "x", "replyTo": id_of(&other), "publishAt": at(soon) }),
        )
        .await
        .err(StatusCode::BAD_REQUEST, "Replies can’t be scheduled.");
    ana.browser
        .post("/scheduled", json!({ "text": "", "publishAt": at(soon) }))
        .await
        .err(StatusCode::BAD_REQUEST, "Write something first.");

    let image = upload(&db, ana.id, "image").await;
    let made = ana
        .browser
        .post(
            "/scheduled",
            json!({ "text": "later #soon", "mediaIds": [image.to_string()], "publishAt": at(soon) }),
        )
        .await
        .ok();
    assert_eq!(made["status"], "pending");
    assert_eq!(made["media"][0]["id"], image.to_string().as_str());
    let sid = made["id"].as_str().expect("id").to_owned();
    ana.browser
        .post(
            "/scheduled",
            json!({ "text": "again", "mediaIds": [image.to_string()], "publishAt": at(soon) }),
        )
        .await
        .err(
            StatusCode::BAD_REQUEST,
            "That upload is already part of another scheduled post.",
        );
    assert_eq!(
        ana.browser
            .get("/scheduled")
            .await
            .ok()
            .as_array()
            .expect("list")
            .len(),
        1
    );

    let moved = ana
        .browser
        .patch(
            &format!("/scheduled/{sid}"),
            json!({ "publishAt": at(soon + time::Duration::hours(1)) }),
        )
        .await
        .ok();
    assert_ne!(moved["publishAt"], made["publishAt"]);
    ana.browser
        .patch(
            &format!("/scheduled/{sid}"),
            json!({ "publishAt": at(time::OffsetDateTime::now_utc()) }),
        )
        .await
        .err(
            StatusCode::BAD_REQUEST,
            "Pick a time between a minute and 90 days from now.",
        );

    // Not due: nothing happens.
    assert_eq!(
        tuita_server::db::scheduled::publish_due(&app.state)
            .await
            .expect("pass"),
        0
    );
    sqlx::query("update scheduled_posts set publish_at = now() - interval '1 second'")
        .execute(&db)
        .await
        .expect("make due");
    assert_eq!(
        tuita_server::db::scheduled::publish_due(&app.state)
            .await
            .expect("pass"),
        1
    );
    let list = ana.browser.get("/scheduled").await.ok();
    assert_eq!(list[0]["status"], "published");
    let post_id = list[0]["postId"].as_str().expect("post id").to_owned();
    let post = ana.browser.get(&format!("/posts/{post_id}")).await.ok();
    assert_eq!(post["post"]["text"], "later #soon");
    assert_eq!(post["post"]["media"][0]["id"], image.to_string().as_str());
    assert_eq!(post["post"]["author"]["handle"], "ana");

    ana.browser
        .delete(&format!("/scheduled/{sid}"))
        .await
        .err(StatusCode::NOT_FOUND, "Scheduled post not found.");
}

#[sqlx::test(migrations = "./migrations")]
async fn publish_now_and_failures(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let at = (time::OffsetDateTime::now_utc() + time::Duration::hours(1))
        .format(&time::format_description::well_known::Rfc3339)
        .expect("format");
    let quoted = ana.browser.post_text("quote me").await;
    let made = ana
        .browser
        .post(
            "/scheduled",
            json!({ "text": "now", "quoteOf": id_of(&quoted), "publishAt": at }),
        )
        .await
        .ok();
    // The quoted post vanishes: the post still goes out, without the quote.
    ana.browser
        .delete(&format!("/posts/{}", id_of(&quoted)))
        .await
        .ok();
    let sid = made["id"].as_str().expect("id");
    let published = ana
        .browser
        .post_empty(&format!("/scheduled/{sid}/publish"))
        .await
        .ok();
    assert_eq!(published["text"], "now");
    assert_eq!(published["quote"], Value::Null);

    let second = ana
        .browser
        .post("/scheduled", json!({ "text": "doomed", "publishAt": at }))
        .await
        .ok();
    sqlx::query("update users set suspended_at = now() where id = $1")
        .bind(ana.id)
        .execute(&db)
        .await
        .expect("suspend");
    sqlx::query(
        "update scheduled_posts set publish_at = now() - interval '1 second' where id = $1",
    )
    .bind(
        second["id"]
            .as_str()
            .expect("id")
            .parse::<Uuid>()
            .expect("uuid"),
    )
    .execute(&db)
    .await
    .expect("make due");
    assert_eq!(
        tuita_server::db::scheduled::publish_due(&app.state)
            .await
            .expect("pass"),
        0
    );
    let (status, error): (String, Option<String>) =
        sqlx::query_as("select status, error from scheduled_posts where id = $1")
            .bind(
                second["id"]
                    .as_str()
                    .expect("id")
                    .parse::<Uuid>()
                    .expect("uuid"),
            )
            .fetch_one(&db)
            .await
            .expect("row");
    assert_eq!(status, "failed");
    assert_eq!(error.as_deref(), Some("Account unavailable"));
}

#[sqlx::test(migrations = "./migrations")]
async fn muted_words_are_phrases_and_hashtags_hide_posts_everywhere(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut ben = app.person(&db, "ben").await;
    ana.browser.post_text("I like concatenate things").await;
    ana.browser.post_text("spoiler alert about the movie").await;
    ana.browser.post_text("plain #secret post").await;
    ana.browser.post_text("fine post").await;
    sqlx::query("insert into follows (follower_id, followee_id) values ($1, $2)")
        .bind(ben.id)
        .bind(ana.id)
        .execute(&db)
        .await
        .expect("follow");
    ben.browser
        .patch(
            "/me",
            json!({ "mutedWords": ["cat", "spoiler alert", "#secret"] }),
        )
        .await
        .ok();
    let texts = |page: &Value| -> Vec<String> {
        page["items"]
            .as_array()
            .expect("items")
            .iter()
            .map(|post| post["text"].as_str().expect("text").to_owned())
            .collect()
    };
    assert_eq!(
        texts(&ben.browser.get("/timelines/following").await.ok()),
        ["fine post"]
    );
    assert_eq!(
        texts(&ben.browser.get("/timelines/foryou").await.ok()),
        ["fine post"]
    );
    assert_eq!(
        texts(&ben.browser.get("/tags/secret/posts").await.ok()),
        Vec::<String>::new()
    );
    let found = ben.browser.get("/search?q=post").await.ok();
    assert_eq!(found["posts"].as_array().expect("posts").len(), 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn the_first_link_is_queued_and_its_finished_card_rides_on_the_post(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut jobs = app.state.take_unfurl_queue().expect("queue");
    let post = ana
        .browser
        .post_text("read https://example.com/a and https://example.com/b")
        .await;
    assert_eq!(post["card"], Value::Null);
    let job = jobs.try_recv().expect("an unfurl job");
    assert_eq!(job.url, "https://example.com/a");
    assert_eq!(job.post_id.to_string(), id_of(&post));
    assert!(jobs.try_recv().is_err(), "only the first link is previewed");

    sqlx::query(
        "insert into link_cards (url, title, description, site, large, ok)
         values ('https://example.com/a', 'Title', 'About', 'example.com', true, true)",
    )
    .execute(&db)
    .await
    .expect("card");
    let seen = ana
        .browser
        .get(&format!("/posts/{}", id_of(&post)))
        .await
        .ok();
    assert_eq!(seen["post"]["card"]["title"], "Title");
    assert_eq!(seen["post"]["card"]["large"], true);
    assert_eq!(seen["post"]["card"]["image"], Value::Null);
}

#[sqlx::test(migrations = "./migrations")]
async fn concurrent_replies_do_not_exhaust_a_tiny_pool(db: PgPool) {
    // A request must hold one connection at a time, or a few of them starve each other.
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .acquire_timeout(std::time::Duration::from_secs(3))
        .connect_with((*db.connect_options()).clone())
        .await
        .expect("small pool");
    let app = app(pool.clone());
    let mut ana = app.person(&db, "ana").await;
    let root = ana.browser.post_text("root").await;
    let root_id = id_of(&root);
    let mut repliers = Vec::new();
    for handle in ["ben", "cyrus", "dan", "eve"] {
        repliers.push(app.person(&db, handle).await);
    }
    let replies = futures_util::future::join_all(repliers.iter_mut().map(|person| {
        let body = json!({ "text": "reply", "replyTo": root_id });
        async move { person.browser.post("/posts", body).await }
    }))
    .await;
    for reply in replies {
        reply.ok();
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn the_31st_edit_in_ten_minutes_is_rate_limited(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let id = id_of(&ana.browser.post_text("v0").await);
    for n in 1..=30 {
        ana.browser
            .patch(&format!("/posts/{id}"), json!({ "text": format!("v{n}") }))
            .await
            .ok();
    }
    let reply = ana
        .browser
        .patch(&format!("/posts/{id}"), json!({ "text": "v31" }))
        .await;
    assert_eq!(
        reply.status,
        StatusCode::TOO_MANY_REQUESTS,
        "{}",
        reply.body
    );
    assert!(!reply.header("retry-after").is_empty());
}
