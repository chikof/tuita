//! How a thread's messages group into bubbles. `messages.js` runs the same rules when messages arrive later.

use time::Duration;
use tuita_server::dto::MessageDto;
use uuid::Uuid;

/// Messages of one sender at most this far apart share a run.
const GROUP: Duration = Duration::minutes(3);

/// One message with its place in a run.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Bubble<'a> {
    pub(crate) message: &'a MessageDto,
    /// First bubble of a run by one sender.
    pub(crate) first: bool,
    /// Last bubble of the run: carries the time.
    pub(crate) last: bool,
}

/// Runs of one sender with each message within 3 minutes of the one before. `messages` is oldest
/// first.
pub(crate) fn group_bubbles(messages: &[MessageDto]) -> Vec<Bubble<'_>> {
    let joined =
        |a: &MessageDto, b: &MessageDto| a.mine == b.mine && b.created_at - a.created_at <= GROUP;
    messages
        .iter()
        .enumerate()
        .map(|(i, message)| Bubble {
            message,
            first: i == 0 || !joined(&messages[i - 1], message),
            last: messages
                .get(i + 1)
                .is_none_or(|next| !joined(message, next)),
        })
        .collect()
}

/// Id of the last own message the other side has read.
pub(crate) fn seen_id(messages: &[MessageDto]) -> Option<Uuid> {
    messages
        .iter()
        .rev()
        .find(|message| message.mine && message.read_at.is_some())
        .map(|message| message.id)
}

#[cfg(test)]
mod test {
    use time::{OffsetDateTime, format_description::well_known::Rfc3339};
    use uuid::Uuid;

    use super::*;

    fn list(rows: &[(u128, &str, bool, bool)]) -> Vec<MessageDto> {
        rows.iter()
            .map(|&(n, clock, mine, read)| {
                let created_at = OffsetDateTime::parse(&format!("2026-01-01T{clock}Z"), &Rfc3339)
                    .expect("a valid time");
                MessageDto {
                    id: Uuid::from_u128(n),
                    text: n.to_string(),
                    ciphertext: None,
                    created_at,
                    mine,
                    read_at: read.then_some(created_at),
                    emojis: Vec::new(),
                }
            })
            .collect()
    }

    #[test]
    fn a_run_is_one_sender_within_three_minutes() {
        let messages = list(&[
            (1, "10:00:00", false, false),
            (2, "10:02:59", false, false),
            (3, "10:06:00", false, false),
            (4, "10:06:10", true, false),
        ]);
        let flags: Vec<_> = group_bubbles(&messages)
            .iter()
            .map(|b| (b.first, b.last))
            .collect();
        assert_eq!(
            flags,
            [(true, false), (false, true), (true, true), (true, true)]
        );
    }

    #[test]
    fn the_seen_message_is_the_last_own_read_one() {
        let messages = list(&[
            (1, "10:00:00", true, true),
            (2, "10:01:00", true, true),
            (3, "10:02:00", true, false),
            (4, "10:03:00", false, true),
        ]);
        assert_eq!(seen_id(&messages), Some(Uuid::from_u128(2)));
        assert_eq!(seen_id(&list(&[(1, "10:00:00", false, false)])), None);
    }
}
