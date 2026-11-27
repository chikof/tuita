//! The sentence of a notification and where it leads.

use tuita_server::dto::{NotificationGroupDto, NotificationKindDto};

use topcoat::context::Cx;

use crate::{
    icons::Icon,
    l10n::{strip_isolates, t},
    paths,
    post::{post_path, profile_path},
};

/// A run of a notification sentence: plain text, or the name of the actor at that index of
/// `group.actors` (user content, drawn with that account's emotes).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Part {
    Text(String),
    Name(usize),
}

/// Kinds that show on the Mentions tab and draw a full post card.
pub(crate) fn is_mention(kind: NotificationKindDto) -> bool {
    matches!(
        kind,
        NotificationKindDto::Reply | NotificationKindDto::Mention | NotificationKindDto::Quote
    )
}

/// The icon in front of a row.
pub(crate) const fn icon_of(kind: NotificationKindDto) -> Icon {
    match kind {
        NotificationKindDto::Like => Icon::Heart,
        NotificationKindDto::Repost => Icon::Repeat,
        NotificationKindDto::Follow | NotificationKindDto::FollowRequest => Icon::UserPlus,
        NotificationKindDto::Reply | NotificationKindDto::Mention | NotificationKindDto::Quote => {
            Icon::Message
        }
        NotificationKindDto::Poll | NotificationKindDto::Affiliation => Icon::InfoBox,
    }
}

/// Stand-in for a name while a message is filled in, so user text never reaches the splitter.
fn slot(index: usize) -> String {
    format!("\u{241f}{index}\u{241f}")
}

/// `count` is 1 or 2 (many): enough for languages whose verb agrees with the number of people.
fn verb(cx: &Cx, kind: NotificationKindDto, count: u32) -> String {
    let text = match kind {
        NotificationKindDto::Like => t!(notif_verb_like, count: count).resolve(cx),
        NotificationKindDto::Repost => t!(notif_verb_repost, count: count).resolve(cx),
        NotificationKindDto::Follow => t!(notif_verb_follow, count: count).resolve(cx),
        NotificationKindDto::FollowRequest => {
            t!(notif_verb_follow_request, count: count).resolve(cx)
        }
        NotificationKindDto::Reply => t!(notif_verb_reply, count: count).resolve(cx),
        NotificationKindDto::Mention => t!(notif_verb_mention, count: count).resolve(cx),
        NotificationKindDto::Quote => t!(notif_verb_quote, count: count).resolve(cx),
        NotificationKindDto::Poll => t!(notif_verb_poll, count: count).resolve(cx),
        NotificationKindDto::Affiliation => t!(notif_verb_affiliation, count: count).resolve(cx),
    };
    strip_isolates(&text)
}

/// Splits `text` at the slots: the pieces between them are text, a slot is the name at its index.
fn split_slots(text: &str) -> Vec<Part> {
    let mut parts = Vec::new();
    let mut plain = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{241f}' {
            let mut ahead = chars.clone();
            if let (Some(digit), Some('\u{241f}')) = (ahead.next(), ahead.next())
                && let Some(index) = digit.to_digit(10)
            {
                if !plain.is_empty() {
                    parts.push(Part::Text(std::mem::take(&mut plain)));
                }
                parts.push(Part::Name(index as usize));
                chars = ahead;
                continue;
            }
        }
        plain.push(c);
    }
    if !plain.is_empty() {
        parts.push(Part::Text(plain));
    }
    parts
}

/// `Ana, Ben and 3 others liked your post`, split so the names can be styled.
pub(crate) fn parts(cx: &Cx, group: &NotificationGroupDto) -> Vec<Part> {
    let others = i64::from(group.actor_count) - 2;
    let who = if group.actors.len() < 2 {
        slot(0)
    } else if others <= 0 {
        strip_isolates(&t!(notif_who_two, first: slot(0), second: slot(1)).resolve(cx))
    } else {
        strip_isolates(
            &t!(notif_who_many, first: slot(0), second: slot(1), count: others).resolve(cx),
        )
    };
    let sentence = format!("{who} {}", verb(cx, group.kind, group.actor_count.min(2)));
    split_slots(&sentence)
        .into_iter()
        .filter(|part| match part {
            Part::Text(text) => !text.is_empty(),
            Part::Name(index) => group.actors.get(*index).is_some(),
        })
        .collect()
}

/// The text of a name part: the display name, else the handle.
pub(crate) fn name_of(group: &NotificationGroupDto, index: usize) -> &str {
    group.actors.get(index).map_or("", |actor| {
        if actor.display_name.is_empty() {
            &actor.handle
        } else {
            &actor.display_name
        }
    })
}

/// Where a row leads: the post, else the follower list, the affiliation settings or the actor.
pub(crate) fn group_path(group: &NotificationGroupDto, me: &str) -> String {
    if let Some(post) = &group.post {
        return post_path(&post.author, post.id);
    }
    if group.kind == NotificationKindDto::Affiliation {
        return "/settings/affiliation".to_owned();
    }
    if group.kind == NotificationKindDto::Follow && group.actor_count > 1 {
        return paths::followers(me);
    }
    group
        .actors
        .first()
        .map_or_else(|| "/notifications".to_owned(), profile_path)
}

#[cfg(test)]
mod test {
    use time::macros::datetime;
    use uuid::Uuid;

    use super::*;
    use crate::{post::testing::sample_user, testing::context};

    fn group(kind: NotificationKindDto, names: &[&str], count: u32) -> NotificationGroupDto {
        NotificationGroupDto {
            id: Uuid::from_u128(7),
            kind,
            read: false,
            created_at: datetime!(2026-11-01 00:00 UTC),
            actors: names
                .iter()
                .map(|name| {
                    let mut user = sample_user(&name.to_lowercase());
                    user.display_name = (*name).to_owned();
                    user
                })
                .collect(),
            actor_count: count,
            post: None,
        }
    }

    /// The sentence as a reader sees it.
    fn sentence(group: &NotificationGroupDto) -> String {
        let cx = context("en");
        parts(&cx, group)
            .iter()
            .map(|part| match part {
                Part::Text(text) => text.clone(),
                Part::Name(index) => name_of(group, *index).to_owned(),
            })
            .collect()
    }

    #[test]
    fn it_names_one_two_or_many_actors_and_marks_only_the_names() {
        let cx = context("en");
        let one = group(NotificationKindDto::Like, &["Ana"], 1);
        assert_eq!(
            parts(&cx, &one),
            vec![Part::Name(0), Part::Text(" liked your post".to_owned())]
        );
        assert_eq!(
            sentence(&group(NotificationKindDto::Repost, &["Ana", "Ben"], 2)),
            "Ana and Ben reposted your post"
        );
        assert_eq!(
            sentence(&group(
                NotificationKindDto::Follow,
                &["Ana", "Ben", "Cy"],
                3
            )),
            "Ana, Ben and 1 other followed you"
        );
        assert_eq!(
            sentence(&group(NotificationKindDto::Like, &["Ana", "Ben", "Cy"], 5)),
            "Ana, Ben and 3 others liked your post"
        );
    }

    #[test]
    fn a_name_that_looks_like_a_slot_is_not_markup() {
        let cx = context("en");
        let odd = group(NotificationKindDto::Like, &["\u{241f}1\u{241f}"], 1);
        assert_eq!(parts(&cx, &odd)[0], Part::Name(0));
        assert_eq!(name_of(&odd, 0), "\u{241f}1\u{241f}");
    }

    #[test]
    fn a_row_leads_to_the_actor_or_to_the_follower_list_of_a_group() {
        let one = group(NotificationKindDto::Follow, &["Ana"], 1);
        assert_eq!(group_path(&one, "me"), "/@ana");
        let many = group(NotificationKindDto::Follow, &["Ana", "Ben"], 4);
        assert_eq!(group_path(&many, "me"), "/@me/followers");
        let affiliation = group(NotificationKindDto::Affiliation, &["Org"], 1);
        assert_eq!(group_path(&affiliation, "me"), "/settings/affiliation");
    }

    #[test]
    fn a_row_with_a_post_leads_to_the_post() {
        let mut liked = group(NotificationKindDto::Like, &["Ana"], 1);
        let post = crate::post::testing::sample_post();
        liked.post = Some(post.clone());
        assert_eq!(group_path(&liked, "me"), post_path(&post.author, post.id));
    }

    #[test]
    fn replies_mentions_and_quotes_are_the_mention_kinds() {
        assert!(is_mention(NotificationKindDto::Reply));
        assert!(is_mention(NotificationKindDto::Mention));
        assert!(is_mention(NotificationKindDto::Quote));
        assert!(!is_mention(NotificationKindDto::Like));
        assert!(!is_mention(NotificationKindDto::Follow));
    }
}
