//! The Mastodon routes clients call at start-up and on common screens (preferences, blocks,
//! mutes, filters, announcements, trends, suggestions, familiar followers, rules, grouped
//! notifications) and CORS on the discovery documents.
#![allow(missing_docs)]

mod common;
#[path = "common/oauth_client.rs"]
mod oauth_client;

use axum::body::Body;
use axum::http::{Method, StatusCode};
use common::{Person, RawReply};
use oauth_client::{Api, Env, sign_in};
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

/// The path and query of the `rel` link of a reply.
fn link(reply: &RawReply, rel: &str) -> Option<String> {
    reply
        .header("link")
        .split(", ")
        .find(|part| part.ends_with(&format!("rel=\"{rel}\"")))
        .map(|part| {
            let url = part
                .split(';')
                .next()
                .unwrap_or_default()
                .trim_matches(['<', '>']);
            url.strip_prefix(common::ORIGIN).unwrap_or(url).to_owned()
        })
}

fn ids(list: &Value) -> Vec<String> {
    list.as_array()
        .expect("array")
        .iter()
        .map(|item| item["id"].as_str().expect("id").to_owned())
        .collect()
}

async fn status(api: &mut Api, text: &str) -> Value {
    api.post_json("/api/v1/statuses", &json!({"status": text}))
        .await
        .ok()
}

/// A client with no token, on another origin.
async fn anonymous(app: &common::App, uri: &str) -> RawReply {
    let mut browser = app.browser_at([203, 0, 113, 90]);
    browser.origin = Some(oauth_client::CLIENT_ORIGIN.to_owned());
    browser.send_raw(Method::GET, uri, &[], Body::empty()).await
}

struct World {
    env: Env,
    db: PgPool,
    ana: Person,
    bo: Person,
    cy: Person,
    dee: Person,
    api: Api,
}

/// `ana` holds a `read write follow` token; `bo`, `cy` and `dee` have sessions only.
async fn world(db: PgPool) -> World {
    let env = Env::new(db.clone());
    let mut ana = env.app.person(&db, "ana").await;
    let bo = env.app.person(&db, "bo").await;
    let cy = env.app.person(&db, "cy").await;
    let dee = env.app.person(&db, "dee").await;
    let api = sign_in(&env.app, &mut ana, "read write follow", [203, 0, 113, 10]).await;
    World {
        env,
        db,
        ana,
        bo,
        cy,
        dee,
        api,
    }
}

const V1_ROUTES: [&str; 14] = [
    "/api/v1/preferences",
    "/api/v1/followed_tags",
    "/api/v1/follow_requests",
    "/api/v1/endorsements",
    "/api/v1/domain_blocks",
    "/api/v1/blocks",
    "/api/v1/mutes",
    "/api/v1/filters",
    "/api/v1/trends/tags",
    "/api/v1/trends/statuses",
    "/api/v1/trends/links",
    "/api/v1/accounts/familiar_followers?id[]=0198f3b2-0000-7000-8000-000000000001",
    "/api/v1/instance/rules",
    "/api/v1/suggestions",
];
const V2_ROUTES: [&str; 3] = [
    "/api/v2/filters",
    "/api/v2/suggestions",
    "/api/v2/notifications",
];

// ------------------------------------------------------------------ authorisation

#[sqlx::test(migrations = "./migrations")]
async fn every_route_needs_a_sign_in_and_the_read_scope(db: PgPool) {
    let mut w = world(db).await;
    for uri in V1_ROUTES.iter().chain(&V2_ROUTES) {
        let reply = anonymous(&w.env.app, uri).await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "{uri} anonymous");
        assert_eq!(w.api.get(uri).await.status, StatusCode::OK, "{uri} read");
    }
    // The announcements route is tuita's own for everyone but an OAuth token, and tuita's
    // announcements are public.
    let public = anonymous(&w.env.app, "/api/v1/announcements").await;
    assert_eq!(public.status, StatusCode::OK);
    assert_eq!(
        w.api.get("/api/v1/announcements").await.status,
        StatusCode::OK
    );

    let mut writer = sign_in(&w.env.app, &mut w.ana, "write", [203, 0, 113, 11]).await;
    for uri in V1_ROUTES
        .iter()
        .chain(&V2_ROUTES)
        .chain(&["/api/v1/announcements"])
    {
        assert_eq!(
            writer.get(uri).await.status,
            StatusCode::FORBIDDEN,
            "{uri} write-only"
        );
    }
}

// ------------------------------------------------------------------ constant answers

#[sqlx::test(migrations = "./migrations")]
async fn preferences_and_the_lists_tuita_has_no_data_for(db: PgPool) {
    let mut w = world(db).await;
    assert_eq!(
        w.api.get("/api/v1/preferences").await.ok(),
        json!({
            "posting:default:visibility": "public",
            "posting:default:sensitive": false,
            "posting:default:language": null,
            "reading:expand:media": "default",
            "reading:expand:spoilers": false,
        })
    );
    for uri in [
        "/api/v1/followed_tags",
        "/api/v1/follow_requests",
        "/api/v1/endorsements",
        "/api/v1/domain_blocks",
        "/api/v1/trends/statuses",
        "/api/v1/trends/links",
        "/api/v1/filters",
        "/api/v2/filters",
        "/api/v1/announcements",
    ] {
        let reply = w.api.get(uri).await;
        assert_eq!(reply.ok(), json!([]), "{uri}");
        assert_eq!(reply.header("link"), "", "{uri}");
    }
    // Nobody is locked: a follow of a local account is accepted at once, so nothing waits.
    w.bo.browser.post_empty("/users/ana/follow").await.ok();
    assert_eq!(w.api.get("/api/v1/follow_requests").await.ok(), json!([]));
    let me = w.api.get("/api/v1/accounts/verify_credentials").await.ok();
    assert_eq!(me["locked"], false);
    assert_eq!(me["source"]["follow_requests_count"], 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn rules_are_the_instance_rules_as_objects(db: PgPool) {
    let mut w = world(db).await;
    let defaults = w.api.get("/api/v1/instance/rules").await.ok();
    assert_eq!(
        defaults.as_array().expect("array").len(),
        5,
        "the built-in rules"
    );
    assert_eq!(defaults[0]["id"], "1");
    sqlx::query(
        "insert into instance_settings (key, value) values ('rules', $1)
         on conflict (key) do update set value = excluded.value",
    )
    .bind(json!(["Be kind.", "No spam."]))
    .execute(&w.db)
    .await
    .expect("rules");
    assert_eq!(
        w.api.get("/api/v1/instance/rules").await.ok(),
        json!([
            {"id": "1", "text": "Be kind.", "hint": ""},
            {"id": "2", "text": "No spam.", "hint": ""},
        ])
    );
}

// ------------------------------------------------------------------ blocks, mutes, filters

#[sqlx::test(migrations = "./migrations")]
async fn blocks_and_mutes_list_the_viewers_own_with_link_paging(db: PgPool) {
    let mut w = world(db).await;
    for handle in ["bo", "cy", "dee"] {
        w.ana
            .browser
            .post_empty(&format!("/users/{handle}/block"))
            .await
            .ok();
        w.ana
            .browser
            .post_empty(&format!("/users/{handle}/mute"))
            .await
            .ok();
    }
    // Another account's lists never show.
    w.bo.browser.post_empty("/users/ana/block").await.ok();

    let first = w.api.get("/api/v1/blocks?limit=2").await;
    let page = first.ok();
    assert_eq!(
        ids(&page),
        [w.dee.id.to_string(), w.cy.id.to_string()],
        "newest block first"
    );
    assert_eq!(page[0]["acct"], "dee");
    assert!(page[0].get("mute_expires_at").is_none());
    let next = link(&first, "next").expect("next link");
    assert!(
        next.starts_with("/api/v1/blocks?") && next.contains("limit=2"),
        "{next}"
    );
    let second = w.api.get(&next).await;
    assert_eq!(ids(&second.ok()), [w.bo.id.to_string()]);
    assert!(link(&second, "next").is_none());
    assert!(link(&second, "prev").is_some());

    let muted = w.api.get("/api/v1/mutes").await.ok();
    assert_eq!(
        ids(&muted),
        [
            w.dee.id.to_string(),
            w.cy.id.to_string(),
            w.bo.id.to_string()
        ]
    );
    assert_eq!(muted[0]["username"], "dee");
    assert!(muted[0]["mute_expires_at"].is_null());
    let paged = w.api.get("/api/v1/mutes?limit=1").await;
    let next = link(&paged, "next").expect("next");
    assert!(next.starts_with("/api/v1/mutes?"), "{next}");
    assert_eq!(
        ids(&w.api.get(&next).await.ok()),
        [w.cy.id.to_string()],
        "the next page starts after the cursor account"
    );

    w.ana.browser.delete("/users/cy/mute").await.ok();
    w.ana.browser.delete("/users/dee/block").await.ok();
    assert_eq!(
        ids(&w.api.get("/api/v1/mutes").await.ok()),
        [w.dee.id.to_string(), w.bo.id.to_string()]
    );
    assert_eq!(
        ids(&w.api.get("/api/v1/blocks").await.ok()),
        [w.cy.id.to_string(), w.bo.id.to_string()]
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn filters_are_the_viewers_muted_words(db: PgPool) {
    let mut w = world(db).await;
    w.ana
        .browser
        .patch("/me", json!({"mutedWords": ["Spoilers", "#cats"]}))
        .await
        .ok();
    assert_eq!(
        w.api.get("/api/v1/filters").await.ok(),
        json!([
            {"id": "1", "phrase": "spoilers", "context": ["home", "notifications", "public", "thread", "account"],
             "expires_at": null, "irreversible": true, "whole_word": false},
            {"id": "2", "phrase": "#cats", "context": ["home", "notifications", "public", "thread", "account"],
             "expires_at": null, "irreversible": true, "whole_word": false},
        ])
    );
    assert_eq!(
        w.api.get("/api/v2/filters").await.ok(),
        json!([{
            "id": "1",
            "title": "Muted words",
            "context": ["home", "notifications", "public", "thread", "account"],
            "expires_at": null,
            "filter_action": "hide",
            "keywords": [
                {"id": "1", "keyword": "spoilers", "whole_word": false},
                {"id": "2", "keyword": "#cats", "whole_word": false},
            ],
            "statuses": [],
        }])
    );
}

// ------------------------------------------------------------------ announcements

#[sqlx::test(migrations = "./migrations")]
async fn announcements_are_mapped_for_oauth_tokens_and_stay_tuitas_for_everyone_else(db: PgPool) {
    let mut w = world(db).await;
    let shown = Uuid::now_v7();
    for (id, text, tone, link, active, expired) in [
        (
            shown,
            "Maintenance <b>tonight</b>",
            "warning",
            "/status",
            true,
            false,
        ),
        (Uuid::now_v7(), "ended", "info", "", false, false),
        (Uuid::now_v7(), "expired", "info", "", true, true),
    ] {
        sqlx::query(
            "insert into announcements (id, text, tone, link, active, expires_at)
             values ($1, $2, $3, $4, $5,
                     case when $6 then now() - interval '1 hour' end)",
        )
        .bind(id)
        .bind(text)
        .bind(tone)
        .bind(link)
        .bind(active)
        .bind(expired)
        .execute(&w.db)
        .await
        .expect("announcement");
    }
    let list = w.api.get("/api/v1/announcements").await.ok();
    assert_eq!(list.as_array().expect("array").len(), 1);
    let item = &list[0];
    assert_eq!(item["id"], shown.to_string());
    assert_eq!(
        item["content"],
        "<p>Maintenance &lt;b&gt;tonight&lt;/b&gt;</p>\
         <p><a href=\"https://social.example/status\" target=\"_blank\" \
         rel=\"nofollow noopener noreferrer\">https://social.example/status</a></p>"
    );
    assert_eq!(item["published"], true);
    assert_eq!(item["all_day"], false);
    assert_eq!(item["read"], false);
    assert!(item["starts_at"].is_null() && item["ends_at"].is_null());
    for key in ["mentions", "statuses", "tags", "emojis", "reactions"] {
        assert_eq!(item[key], json!([]), "{key}");
    }
    let published = item["published_at"].as_str().expect("time");
    assert!(
        published.ends_with('Z') && published.contains('.'),
        "{published}"
    );
    assert_eq!(item["updated_at"], item["published_at"]);

    // A session (the web client) keeps tuita's own shape, signed in or not.
    let own = w.ana.browser.get("/announcements").await.ok();
    assert_eq!(own[0]["tone"], "warning");
    assert!(own[0].get("content").is_none());
    let public = anonymous(&w.env.app, "/api/v1/announcements").await.ok();
    assert_eq!(public[0]["text"], "Maintenance <b>tonight</b>");
}

// ------------------------------------------------------------------ trends

#[sqlx::test(migrations = "./migrations")]
async fn trending_tags_carry_a_week_of_daily_use(db: PgPool) {
    let mut w = world(db).await;
    w.bo.browser
        .post("/posts", json!({"text": "learning #rust today"}))
        .await
        .ok();
    w.cy.browser
        .post("/posts", json!({"text": "more #Rust"}))
        .await
        .ok();
    w.cy.browser
        .post("/posts", json!({"text": "again #rust"}))
        .await
        .ok();
    let old = w
        .dee
        .browser
        .post("/posts", json!({"text": "a #gopher story"}))
        .await
        .ok();
    sqlx::query("update posts set created_at = now() - interval '3 days' where id = $1")
        .bind(
            old["id"]
                .as_str()
                .expect("id")
                .parse::<Uuid>()
                .expect("uuid"),
        )
        .execute(&w.db)
        .await
        .expect("backdate");

    let trends = w.api.get("/api/v1/trends/tags").await.ok();
    let names: Vec<&str> = trends
        .as_array()
        .expect("array")
        .iter()
        .map(|tag| tag["name"].as_str().expect("name"))
        .collect();
    assert_eq!(names, ["rust", "gopher"]);
    let rust = &trends[0];
    assert_eq!(rust["url"], "https://social.example/tags/rust");
    assert_eq!(rust["following"], false);
    let history = rust["history"].as_array().expect("history");
    assert_eq!(history.len(), 7);
    assert_eq!(history[0]["uses"], "3");
    assert_eq!(history[0]["accounts"], "2");
    for day in &history[1..] {
        assert_eq!(day["uses"], "0");
        assert_eq!(day["accounts"], "0");
    }
    let days: Vec<i64> = history
        .iter()
        .map(|day| day["day"].as_str().expect("day").parse().expect("number"))
        .collect();
    assert_eq!(days[0] % 86_400, 0, "UTC midnight");
    assert!(days.windows(2).all(|pair| pair[0] - pair[1] == 86_400));
    let gopher = trends[1]["history"].as_array().expect("history");
    assert_eq!(gopher[0]["uses"], "0");
    assert_eq!(gopher[3]["uses"], "1");
    assert_eq!(gopher[3]["accounts"], "1");

    let limited = w.api.get("/api/v1/trends/tags?limit=1&offset=1").await.ok();
    assert_eq!(limited.as_array().expect("array").len(), 1);
    assert_eq!(limited[0]["name"], "gopher");
}

// ------------------------------------------------------------------ suggestions, familiar followers

#[sqlx::test(migrations = "./migrations")]
async fn suggestions_are_tuitas_in_both_shapes(db: PgPool) {
    let mut w = world(db).await;
    w.ana.browser.post_empty("/users/bo/follow").await.ok();
    let (cy, dee) = (w.cy.id.to_string(), w.dee.id.to_string());

    let accounts = w.api.get("/api/v1/suggestions").await.ok();
    assert_eq!(
        ids(&accounts),
        [cy.clone(), dee.clone()],
        "not self, not followed"
    );
    assert_eq!(accounts[0]["acct"], "cy");
    assert!(accounts[0]["followers_count"].is_number());

    let suggestions = w.api.get("/api/v2/suggestions").await.ok();
    let suggestions = suggestions.as_array().expect("array");
    assert_eq!(suggestions.len(), 2);
    assert_eq!(suggestions[0]["source"], "global");
    assert_eq!(suggestions[0]["sources"], json!(["most_followed"]));
    assert_eq!(suggestions[0]["account"]["id"], cy);
    assert_eq!(suggestions[1]["account"]["id"], dee);
    let one = w.api.get("/api/v2/suggestions?limit=1").await.ok();
    assert_eq!(one.as_array().expect("array").len(), 1);

    // A session keeps tuita's own shape on the shared path.
    let own = w.ana.browser.get("/suggestions").await.ok();
    assert_eq!(own[0]["handle"], "cy");
}

#[sqlx::test(migrations = "./migrations")]
async fn familiar_followers_are_the_followers_the_viewer_follows(db: PgPool) {
    let mut w = world(db).await;
    let eve = w.env.app.person(&w.db, "eve").await;
    let mut eve = eve;
    w.ana.browser.post_empty("/users/bo/follow").await.ok();
    w.ana.browser.post_empty("/users/cy/follow").await.ok();
    for who in [&mut w.bo, &mut w.cy] {
        who.browser.post_empty("/users/dee/follow").await.ok();
    }
    eve.browser.post_empty("/users/dee/follow").await.ok();

    let unknown = Uuid::now_v7();
    let reply = w
        .api
        .get(&format!(
            "/api/v1/accounts/familiar_followers?id[]={}&id[]={}&id[]={unknown}&id[]=junk&id[]={}",
            w.dee.id, w.ana.id, w.dee.id
        ))
        .await
        .ok();
    let entries = reply.as_array().expect("array");
    assert_eq!(
        entries.len(),
        2,
        "unknown and malformed ids are left out: {reply}"
    );
    assert_eq!(entries[0]["id"], w.dee.id.to_string());
    let mut familiar = ids(&entries[0]["accounts"]);
    familiar.sort();
    let mut expected = vec![w.bo.id.to_string(), w.cy.id.to_string()];
    expected.sort();
    assert_eq!(familiar, expected);
    assert_eq!(
        entries[0]["accounts"][0]["acct"].as_str().map(str::len),
        Some(2)
    );
    assert_eq!(
        entries[1],
        json!({"id": w.ana.id.to_string(), "accounts": []})
    );

    assert_eq!(
        w.api.get("/api/v1/accounts/familiar_followers").await.ok(),
        json!([])
    );
}

// ------------------------------------------------------------------ grouped notifications

#[sqlx::test(migrations = "./migrations")]
async fn grouped_notifications_list_every_notification_as_a_group(db: PgPool) {
    let mut w = world(db).await;
    let mut bo = sign_in(
        &w.env.app,
        &mut w.bo,
        "read write follow",
        [203, 0, 113, 12],
    )
    .await;
    w.bo.browser.post_empty("/users/ana/follow").await.ok();
    let post = status(&mut w.api, "hello").await;
    let post_id = post["id"].as_str().expect("id").to_owned();
    bo.post(&format!("/api/v1/statuses/{post_id}/favourite"))
        .await
        .ok();

    let v1 = w.api.get("/api/v1/notifications").await.ok();
    let v1_ids = ids(&v1);
    assert_eq!(v1_ids.len(), 2);

    let reply = w.api.get("/api/v2/notifications").await;
    let body = reply.ok();
    let groups = body["notification_groups"].as_array().expect("groups");
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0]["type"], "favourite");
    assert_eq!(groups[0]["status_id"], post_id);
    assert_eq!(groups[0]["notifications_count"], 1);
    assert_eq!(groups[0]["most_recent_notification_id"], v1_ids[0]);
    assert_eq!(groups[0]["page_min_id"], v1_ids[0]);
    assert_eq!(groups[0]["page_max_id"], v1_ids[0]);
    assert_eq!(groups[0]["group_key"], format!("ungrouped-{}", v1_ids[0]));
    assert_eq!(
        groups[0]["sample_account_ids"],
        json!([w.bo.id.to_string()])
    );
    assert_eq!(
        groups[0]["latest_page_notification_at"],
        v1[0]["created_at"]
    );
    assert_eq!(groups[1]["type"], "follow");
    assert!(groups[1].get("status_id").is_none());
    let accounts = body["accounts"].as_array().expect("accounts");
    assert_eq!(accounts.len(), 1, "one account, listed once");
    assert_eq!(accounts[0]["acct"], "bo");
    let statuses = body["statuses"].as_array().expect("statuses");
    assert_eq!(statuses.len(), 1);
    assert_eq!(statuses[0]["id"], post_id);
    assert_eq!(statuses[0]["favourites_count"], 1);

    let paged = w.api.get("/api/v2/notifications?limit=1").await;
    let next = link(&paged, "next").expect("next link");
    assert!(next.starts_with("/api/v2/notifications?"), "{next}");
    let second = w.api.get(&next).await.ok();
    assert_eq!(second["notification_groups"][0]["type"], "follow");
    assert_eq!(
        second["notification_groups"]
            .as_array()
            .expect("groups")
            .len(),
        1
    );

    let only = w.api.get("/api/v2/notifications?types[]=follow").await.ok();
    assert_eq!(
        only["notification_groups"]
            .as_array()
            .expect("groups")
            .len(),
        1
    );
    assert_eq!(only["statuses"], json!([]));
    let empty = w
        .api
        .get("/api/v2/notifications?exclude_types[]=follow&exclude_types[]=favourite")
        .await
        .ok();
    assert_eq!(
        empty,
        json!({"accounts": [], "statuses": [], "notification_groups": []})
    );
}

// ------------------------------------------------------------------ CORS

#[sqlx::test(migrations = "./migrations")]
async fn discovery_documents_answer_cors_preflights_and_carry_the_origin_header(db: PgPool) {
    let app = common::app(db.clone());
    app.person(&db, "ana").await;
    let mut web_client = app.browser_at([203, 0, 113, 20]);
    web_client.origin = Some("https://phanpy.social".to_owned());
    let preflight = [("access-control-request-method", "GET")];
    for uri in [
        "/.well-known/host-meta",
        "/.well-known/webfinger",
        "/.well-known/nodeinfo",
        "/nodeinfo/2.0",
        "/nodeinfo/2.1",
    ] {
        let reply = web_client
            .send_raw(Method::OPTIONS, uri, &preflight, Body::empty())
            .await;
        assert!(reply.status.is_success(), "{uri}: {}", reply.status);
        assert_eq!(reply.header("access-control-allow-origin"), "*", "{uri}");
        assert!(
            reply.header("access-control-allow-methods").contains("GET"),
            "{uri}"
        );
        assert!(
            reply
                .headers
                .get("access-control-allow-credentials")
                .is_none(),
            "{uri}"
        );
    }
    for (uri, status) in [
        ("/.well-known/host-meta", StatusCode::OK),
        (
            "/.well-known/webfinger?resource=acct:ana@social.example",
            StatusCode::OK,
        ),
        (
            "/.well-known/webfinger?resource=acct:nobody@social.example",
            StatusCode::NOT_FOUND,
        ),
        ("/.well-known/webfinger", StatusCode::BAD_REQUEST),
        ("/.well-known/nodeinfo", StatusCode::OK),
        ("/nodeinfo/2.0", StatusCode::OK),
        ("/nodeinfo/2.1", StatusCode::OK),
    ] {
        let reply = web_client
            .send_raw(Method::GET, uri, &[], Body::empty())
            .await;
        assert_eq!(reply.status, status, "{uri}");
        assert_eq!(reply.header("access-control-allow-origin"), "*", "{uri}");
        assert!(
            reply
                .headers
                .get("access-control-allow-credentials")
                .is_none(),
            "{uri}"
        );
    }
    // The CORS layer is only on the discovery routes: the actor document is not opened up.
    let actor = web_client
        .send_raw(
            Method::GET,
            "/users/ana",
            &[("accept", "application/activity+json")],
            Body::empty(),
        )
        .await;
    assert_eq!(actor.status, StatusCode::OK);
    assert!(actor.headers.get("access-control-allow-origin").is_none());
}

#[sqlx::test(migrations = "./migrations")]
async fn nodeinfo_is_served_as_2_0_and_2_1_and_both_are_linked(db: PgPool) {
    let app = common::app(db.clone());
    app.person(&db, "ana").await;
    let mut visitor = app.browser();
    let links = visitor
        .send_raw(Method::GET, "/.well-known/nodeinfo", &[], Body::empty())
        .await
        .ok();
    assert_eq!(
        links,
        json!({"links": [
            {"rel": "http://nodeinfo.diaspora.software/ns/schema/2.0",
             "href": "https://social.example/nodeinfo/2.0"},
            {"rel": "http://nodeinfo.diaspora.software/ns/schema/2.1",
             "href": "https://social.example/nodeinfo/2.1"},
        ]})
    );
    for version in ["2.0", "2.1"] {
        let reply = visitor
            .send_raw(
                Method::GET,
                &format!("/nodeinfo/{version}"),
                &[],
                Body::empty(),
            )
            .await;
        assert_eq!(
            reply.header("content-type"),
            format!(
                "application/json; profile=\"http://nodeinfo.diaspora.software/ns/schema/{version}#\""
            )
        );
        let info = reply.ok();
        assert_eq!(info["version"], version);
        assert_eq!(info["software"]["name"], "tuita");
        assert_eq!(info["usage"]["users"]["total"], 1);
    }
}
