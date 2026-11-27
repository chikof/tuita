//! Open Graph and preview tags for a page head, so link previews work for crawlers.

use std::borrow::Cow;
use std::fmt::Write as _;

use url::form_urlencoded;

use crate::db::settings;
use crate::state::AppState;

/// Characters of a post that go into its preview description.
const POST_DESCRIPTION_CHARS: usize = 200;
/// `type` of the `<link rel="alternate">` that points federation clients at the actor or note.
const ACTIVITY_JSON: &str = "application/activity+json";
/// `type` of the `<link rel="alternate">` that points embedders at `/oembed`.
const OEMBED_JSON: &str = "application/json+oembed";

/// What a preview shows for one URL.
struct Preview {
    kind: &'static str,
    title: String,
    description: String,
    card: &'static str,
    /// `<link rel="alternate">` targets as `(type, href)`.
    alternates: Vec<(&'static str, String)>,
}

/// Meta tags for `path`: a profile or post preview when the URL names a visible one, else the
/// instance's name and description. Deleted, suspended and unknown targets get the generic tags.
pub async fn head_tags(state: &AppState, path: &str) -> String {
    let config = state.config();
    let url = format!("{}{path}", config.public_url);
    let instance = match settings::load(state.db()).await {
        Ok(instance) => instance,
        Err(err) => {
            tracing::warn!(error = %err, "cannot read instance settings for the page head");
            settings::InstanceSettings::defaults()
        }
    };
    let preview = match preview_for(state, path, &instance.name).await {
        Ok(preview) => preview,
        Err(err) => {
            tracing::warn!(error = %err, "cannot build a preview");
            None
        }
    };
    let mut preview = preview.unwrap_or(Preview {
        kind: "website",
        title: instance.name.clone(),
        description: instance.description.clone(),
        card: "summary",
        alternates: Vec::new(),
    });
    if preview.description.trim().is_empty() {
        preview.description.clone_from(&instance.description);
    }
    let mut head = String::with_capacity(512);
    let mut meta = |property: &str, content: &str| {
        let _ = writeln!(
            head,
            r#"<meta property="{property}" content="{}">"#,
            html_escape(content)
        );
    };
    meta("og:site_name", &instance.name);
    meta("og:type", preview.kind);
    meta("og:title", &preview.title);
    meta("og:description", &preview.description);
    meta("og:url", &url);
    let _ = writeln!(
        head,
        r#"<meta name="description" content="{}">"#,
        html_escape(&preview.description)
    );
    let _ = writeln!(
        head,
        r#"<meta name="twitter:card" content="{}">"#,
        preview.card
    );
    for (kind, href) in &preview.alternates {
        let _ = writeln!(
            head,
            r#"<link rel="alternate" type="{kind}" href="{}">"#,
            html_escape(href)
        );
    }
    head
}

/// The preview of `/@handle` and `/@handle/{post id}`; `None` for every other path and for
/// targets the public may not see.
async fn preview_for(
    state: &AppState,
    path: &str,
    instance_name: &str,
) -> Result<Option<Preview>, sqlx::Error> {
    let Some(rest) = path.strip_prefix("/@") else {
        return Ok(None);
    };
    let (handle, post) = match rest.split_once('/') {
        None => (rest, None),
        Some((handle, id)) => match uuid::Uuid::parse_str(id) {
            Ok(id) => (handle, Some(id)),
            Err(_) => return Ok(None),
        },
    };
    let valid = (1..=15).contains(&handle.len())
        && handle
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_');
    if !valid {
        return Ok(None);
    }
    let Some((id, handle, name, bio)) = sqlx::query_as::<_, (uuid::Uuid, String, String, String)>(
        "select id, handle, display_name, bio from users
         where lower(handle) = lower($1) and domain is null and lower(handle) <> 'tuita'
           and suspended_at is null and deleted_at is null",
    )
    .bind(handle)
    .fetch_optional(state.db())
    .await?
    else {
        return Ok(None);
    };
    let public = &state.config().public_url;
    let Some(post) = post else {
        return Ok(Some(Preview {
            kind: "profile",
            title: format!("{name} (@{handle})"),
            description: bio,
            card: "summary",
            alternates: vec![(ACTIVITY_JSON, format!("{public}/users/{handle}"))],
        }));
    };
    let found: Option<PostFacts> = sqlx::query_as(
        "select text, cw, visibility, like_count, repost_count + quote_count as reposts,
                reply_count, coalesce(views, 0) as views
           from posts where id = $1 and author_id = $2 and deleted_at is null
            and visibility in ('public', 'unlisted')",
    )
    .bind(post)
    .bind(id)
    .fetch_optional(state.db())
    .await?;
    Ok(found.map(|facts| {
        let mut description: String = (if facts.cw.is_empty() {
            &facts.text
        } else {
            &facts.cw
        })
        .chars()
        .take(POST_DESCRIPTION_CHARS)
        .collect();
        let stats = stats_line(&facts);
        if !stats.is_empty() {
            if !description.is_empty() {
                description.push_str("\n\n");
            }
            description.push_str(&stats);
        }
        let mut alternates = vec![(
            ACTIVITY_JSON,
            format!("{public}/users/{handle}/statuses/{post}"),
        )];
        if facts.visibility == "public" {
            let page: String =
                form_urlencoded::byte_serialize(format!("{public}/@{handle}/{post}").as_bytes())
                    .collect();
            alternates.push((OEMBED_JSON, format!("{public}/oembed?url={page}")));
        }
        Preview {
            kind: "article",
            title: format!("{name} on {instance_name}"),
            description,
            card: "summary",
            alternates,
        }
    }))
}

/// What a post preview reads from the post row.
#[derive(sqlx::FromRow)]
struct PostFacts {
    text: String,
    cw: String,
    visibility: String,
    like_count: i32,
    reposts: i32,
    reply_count: i32,
    views: i64,
}

/// A short number: `950`, `1.2K`, `3.4M`.
fn compact(count: i64) -> String {
    match count {
        ..1_000 => count.to_string(),
        1_000..1_000_000 => format!("{:.1}K", count as f64 / 1_000.0).replace(".0K", "K"),
        _ => format!("{:.1}M", count as f64 / 1_000_000.0).replace(".0M", "M"),
    }
}

/// The counters of a post as text, nonzero ones only: `💬 2   🔁 1   ❤️ 3   👁 1.2K`. Crawlers show the
/// description, not the page, so this is where the view count appears.
fn stats_line(facts: &PostFacts) -> String {
    [
        ("💬", i64::from(facts.reply_count)),
        ("🔁", i64::from(facts.reposts)),
        ("❤️", i64::from(facts.like_count)),
        ("👁", facts.views),
    ]
    .into_iter()
    .filter(|(_, count)| *count > 0)
    .map(|(icon, count)| format!("{icon} {}", compact(count)))
    .collect::<Vec<_>>()
    .join("   ")
}

/// Escapes `& < > " '` for HTML text and attribute values.
#[must_use]
pub fn html_escape(text: &str) -> Cow<'_, str> {
    if !text.contains(['&', '<', '>', '"', '\'']) {
        return Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len() + 16);
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    Cow::Owned(out)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn escapes_all_five_characters() {
        assert_eq!(
            html_escape(r#"<a href="x">Tom & 'Jerry'</a>"#),
            "&lt;a href=&quot;x&quot;&gt;Tom &amp; &#39;Jerry&#39;&lt;/a&gt;"
        );
        assert!(matches!(html_escape("plain"), Cow::Borrowed("plain")));
    }
}
