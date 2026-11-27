//! The numbers of a poll.

use time::OffsetDateTime;
use tuita_server::dto::PollDto;

/// Percentage of all votes per answer, rounded; all zero before the first vote.
#[must_use]
pub(crate) fn shares(poll: &PollDto) -> Vec<u32> {
    poll.options
        .iter()
        .map(|option| {
            if poll.total > 0 {
                (f64::from(option.votes) / f64::from(poll.total) * 100.0).round() as u32
            } else {
                0
            }
        })
        .collect()
}

/// Indexes of the answers with the most votes; empty when nobody voted.
#[must_use]
pub(crate) fn leaders(poll: &PollDto) -> Vec<usize> {
    let top = poll
        .options
        .iter()
        .map(|option| option.votes)
        .max()
        .unwrap_or(0)
        .max(0);
    if top == 0 {
        return Vec::new();
    }
    poll.options
        .iter()
        .enumerate()
        .filter(|(_, option)| option.votes == top)
        .map(|(index, _)| index)
        .collect()
}

/// A time span in the coarsest unit that is not zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Left {
    Minutes(i64),
    Hours(i64),
    Days(i64),
}

/// Time until `ends_at`, in the coarsest unit that is not zero.
#[must_use]
pub(crate) fn time_left(ends_at: OffsetDateTime, now: OffsetDateTime) -> Left {
    let seconds = (ends_at - now).as_seconds_f64().max(0.0);
    if seconds < 3600.0 {
        Left::Minutes(((seconds / 60.0).round() as i64).max(1))
    } else if seconds < 86_400.0 {
        Left::Hours((seconds / 3600.0).round() as i64)
    } else {
        Left::Days((seconds / 86_400.0).round() as i64)
    }
}

/// Whether results show instead of buttons: ended, already voted, or the author's own poll.
#[must_use]
pub(crate) fn shows_results(poll: &PollDto, mine: bool) -> bool {
    poll.closed || !poll.voted.is_empty() || mine
}

#[cfg(test)]
mod test {
    use time::macros::datetime;
    use tuita_server::dto::PollOptionDto;

    use super::*;

    fn poll(votes: &[i32], total: i32) -> PollDto {
        PollDto {
            options: votes
                .iter()
                .map(|votes| PollOptionDto {
                    text: "x".to_owned(),
                    votes: *votes,
                })
                .collect(),
            total,
            voters: total,
            multiple: false,
            ends_at: datetime!(2026-11-09 10:00 UTC),
            closed: false,
            voted: Vec::new(),
        }
    }

    #[test]
    fn shares_round_and_survive_an_empty_poll() {
        assert_eq!(shares(&poll(&[3, 1, 3], 7)), [43, 14, 43]);
        assert_eq!(shares(&poll(&[0], 0)), [0]);
    }

    #[test]
    fn every_leader_is_found_and_none_without_votes() {
        assert_eq!(leaders(&poll(&[3, 1, 3], 7)), [0, 2]);
        assert!(leaders(&poll(&[0], 0)).is_empty());
    }

    #[test]
    fn the_coarsest_unit_wins() {
        let now = datetime!(2026-11-09 10:00 UTC);
        assert_eq!(
            time_left(datetime!(2026-11-09 10:00:20 UTC), now),
            Left::Minutes(1)
        );
        assert_eq!(
            time_left(datetime!(2026-11-09 12:00 UTC), now),
            Left::Hours(2)
        );
        assert_eq!(
            time_left(datetime!(2026-11-12 10:00 UTC), now),
            Left::Days(3)
        );
        assert_eq!(
            time_left(datetime!(2026-11-09 09:00 UTC), now),
            Left::Minutes(1)
        );
    }

    #[test]
    fn results_show_once_closed_voted_or_authored() {
        let mut open = poll(&[1], 1);
        assert!(!shows_results(&open, false));
        assert!(shows_results(&open, true));
        open.voted = vec![1];
        assert!(shows_results(&open, false));
        open.voted.clear();
        open.closed = true;
        assert!(shows_results(&open, false));
    }
}
