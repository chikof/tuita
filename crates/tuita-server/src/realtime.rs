//! Live events: one broadcast channel per signed-in user, fed by handlers and read by
//! `GET /api/v1/stream`.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::dto::StreamEvent;

/// Events a slow reader may fall behind before it loses some.
const CHANNEL_CAPACITY: usize = 64;

/// Open event streams one user may hold; opening one more ends the oldest.
pub const MAX_STREAMS_PER_USER: usize = 12;

/// An event as it travels through the hub: shared, so each reader clones a pointer.
pub type Event = Arc<StreamEvent>;

/// Fan-out of [`StreamEvent`]s to the open streams of each user.
// shortcut: in-process fan-out, single replica only; move to Postgres LISTEN/NOTIFY before running 2+ replicas
#[derive(Debug, Default)]
pub struct Hub {
    senders: Mutex<HashMap<Uuid, broadcast::Sender<Event>>>,
    streams: Mutex<HashMap<Uuid, Vec<(u64, CancellationToken)>>>,
    next_stream: AtomicU64,
}

impl Hub {
    /// Creates a hub with no channels.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts listening for `user`'s events.
    pub fn subscribe(&self, user: Uuid) -> broadcast::Receiver<Event> {
        let mut senders = self.senders.lock().unwrap_or_else(PoisonError::into_inner);
        senders
            .entry(user)
            .or_insert_with(|| broadcast::channel(CHANNEL_CAPACITY).0)
            .subscribe()
    }

    /// Registers an open stream of `user`. Returns its id and a token that is cancelled when the
    /// stream must end because the user opened more than [`MAX_STREAMS_PER_USER`].
    pub fn open_stream(&self, user: Uuid) -> (u64, CancellationToken) {
        let id = self.next_stream.fetch_add(1, Ordering::Relaxed);
        let token = CancellationToken::new();
        let mut streams = self.streams.lock().unwrap_or_else(PoisonError::into_inner);
        let open = streams.entry(user).or_default();
        open.push((id, token.clone()));
        while open.len() > MAX_STREAMS_PER_USER {
            let (_, oldest) = open.remove(0);
            oldest.cancel();
        }
        (id, token)
    }

    /// Forgets a stream registered with [`Self::open_stream`].
    pub fn close_stream(&self, user: Uuid, id: u64) {
        let mut streams = self.streams.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(open) = streams.get_mut(&user) {
            open.retain(|(open_id, _)| *open_id != id);
            if open.is_empty() {
                streams.remove(&user);
            }
        }
    }

    /// Whether `user` has an open stream. Lets callers skip building an event nobody reads.
    #[must_use]
    pub fn is_listening(&self, user: Uuid) -> bool {
        let senders = self.senders.lock().unwrap_or_else(PoisonError::into_inner);
        senders
            .get(&user)
            .is_some_and(|sender| sender.receiver_count() > 0)
    }

    /// The users with an open stream. Lets a publisher with a large audience (followers of an
    /// author) intersect it with the people who are online instead of loading them all.
    #[must_use]
    pub fn listening(&self) -> Vec<Uuid> {
        let senders = self.senders.lock().unwrap_or_else(PoisonError::into_inner);
        senders
            .iter()
            .filter(|(_, sender)| sender.receiver_count() > 0)
            .map(|(user, _)| *user)
            .collect()
    }

    /// Sends `event` to every open stream of `user`. Does nothing when there is none.
    pub fn publish(&self, user: Uuid, event: StreamEvent) {
        let senders = self.senders.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(sender) = senders.get(&user) {
            // An error only means every receiver is gone; the sweep removes the sender.
            let _ = sender.send(Arc::new(event));
        }
    }

    /// Drops the channels nobody listens to. Returns how many it removed.
    pub fn sweep(&self) -> usize {
        let mut senders = self.senders.lock().unwrap_or_else(PoisonError::into_inner);
        let before = senders.len();
        senders.retain(|_, sender| sender.receiver_count() > 0);
        before - senders.len()
    }

    /// How many users have a channel.
    #[must_use]
    pub fn channels(&self) -> usize {
        self.senders
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .len()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::dto::{ReadEventDto, ReadKind};

    fn read() -> StreamEvent {
        StreamEvent::Read(ReadEventDto {
            kind: ReadKind::Notifications,
        })
    }

    #[tokio::test]
    async fn delivers_to_every_stream_of_the_user_only() {
        let hub = Hub::new();
        let (ana, ben) = (Uuid::now_v7(), Uuid::now_v7());
        let mut first = hub.subscribe(ana);
        let mut second = hub.subscribe(ana);
        let mut other = hub.subscribe(ben);
        hub.publish(ana, read());
        assert_eq!(*first.recv().await.expect("event"), read());
        assert_eq!(*second.recv().await.expect("event"), read());
        assert!(other.try_recv().is_err());
    }

    #[test]
    fn publish_without_listener_is_a_no_op_and_sweep_frees_channels() {
        let hub = Hub::new();
        let ana = Uuid::now_v7();
        hub.publish(ana, read());
        assert_eq!(hub.channels(), 0);
        let rx = hub.subscribe(ana);
        assert!(hub.is_listening(ana));
        assert_eq!(hub.sweep(), 0);
        drop(rx);
        assert!(!hub.is_listening(ana));
        assert_eq!(hub.sweep(), 1);
        assert_eq!(hub.channels(), 0);
    }

    #[test]
    fn listening_lists_only_users_with_a_reader() {
        let hub = Hub::new();
        let (ana, ben) = (Uuid::now_v7(), Uuid::now_v7());
        let _ana_rx = hub.subscribe(ana);
        drop(hub.subscribe(ben));
        assert_eq!(hub.listening(), vec![ana]);
    }
}
