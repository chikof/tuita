// Known-answer vectors for assets/js/stream.js.
const assert = (ok, what) => {
	if (!ok) throw new Error(`assertion failed: ${what}`);
};
const eq = (actual, expected, what) => {
	if (JSON.stringify(actual) !== JSON.stringify(expected)) {
		throw new Error(`${what}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
	}
};

class FakeSource {
	onopen = null;
	onerror = null;
	closed = false;
	listeners = new Map();
	addEventListener(type, listener) {
		this.listeners.set(type, listener);
	}
	close() {
		this.closed = true;
	}
	emit(type, data) {
		this.listeners.get(type)?.({ data: JSON.stringify(data) });
	}
}

const strings = {
	notify_like: '@{handle} liked your post',
	notify_follow: '@{handle} followed you',
	notify_message: '@{handle} sent you a message',
	view: 'View',
	badge_unread: { 0: '{count} unread', 1: '{count} unread', 2: '{count} unread' }
};

const actor = { id: 'b', handle: 'b', domain: null, displayName: 'Bee' };

const note = (over = {}) => ({
	type: 'notification',
	data: {
		id: 'n1',
		kind: 'like',
		actor,
		post: { id: 'p1', text: 'hello', cw: '' },
		createdAt: '2026-11-09T10:00:00Z',
		...over
	}
});

const message = (encrypted) => ({
	type: 'message',
	data: { from: actor, text: encrypted ? '' : 'lunch at noon?', encrypted, createdAt: '2026-11-09T10:00:00Z' }
});

function harness({ hidden = false, canNotify = true, here = '/', me = 'a' } = {}) {
	const log = { toasts: [], desktops: [], navigated: [], badges: {}, emitted: [], cards: [] };
	const deps = {
		strings,
		lang: 'en',
		me,
		here: () => here,
		hidden: () => hidden,
		canNotify: () => canNotify,
		desktop: (title, body, onclick) => log.desktops.push({ title, body, onclick }),
		toast: (text, action) => log.toasts.push({ text, action }),
		navigate: (path) => log.navigated.push(path),
		setBadge: (kind, n) => (log.badges[kind] = n),
		emit: (type, data) => log.emitted.push(type),
		card: (id) => log.cards.push(id)
	};
	return { deps, log };
}

export async function run(mod) {
	// Reconnects with 1, 2, 5, 10, 30, 30 s and resets after a successful open.
	{
		const sources = [];
		const waits = [];
		let pending;
		const stop = mod.openStream('/api/v1/stream', {
			open: () => {
				const source = new FakeSource();
				sources.push(source);
				return source;
			},
			later: (run, ms) => {
				waits.push(ms);
				pending = run;
				return () => (pending = undefined);
			},
			onEvent: () => {}
		});
		for (let i = 0; i < 6; i++) {
			sources.at(-1).onerror?.({});
			pending?.();
		}
		eq(waits, [1000, 2000, 5000, 10000, 30000, 30000], 'backoff');
		eq(mod.BACKOFF_MS.at(-1), 30000, 'the last delay repeats');
		assert(sources.slice(0, -1).every((s) => s.closed), 'failed sources are closed');
		sources.at(-1).onopen?.({});
		sources.at(-1).onerror?.({});
		eq(waits.at(-1), 1000, 'a successful open resets the delay');
		stop();
		assert(pending === undefined, 'stop cancels the pending reconnect');
	}

	// Frames become events; malformed ones are dropped.
	{
		const seen = [];
		let source;
		mod.openStream('/s', { open: () => (source = new FakeSource()), later: () => () => {}, onEvent: (e) => seen.push(e) });
		source.emit('badges', { type: 'badges', data: { notifications: 2, messages: 0, reports: null } });
		source.listeners.get('badges')?.({ data: '{nope' });
		eq(seen, [{ type: 'badges', data: { notifications: 2, messages: 0, reports: null } }], 'parsed frames');
		eq([...source.listeners.keys()], ['badges', 'notification', 'message', 'read', 'card'], 'listens to every event name');
	}

	// A toast "@b liked your post" that opens the post.
	{
		const { deps, log } = harness();
		mod.applyEvent(note(), deps);
		eq(log.toasts.map((t) => t.text), ['@b liked your post'], 'like toast');
		eq(log.toasts[0].action, { label: 'View', href: '/@a/p1' }, 'the toast opens the post');
		eq(log.desktops, [], 'no desktop notification while visible');
		eq(log.emitted, ['notification'], 'the event reaches Topcoat');
	}

	// A desktop notification when the tab is hidden and permitted.
	{
		const { deps, log } = harness({ hidden: true });
		mod.applyEvent(note({ kind: 'follow', post: null }), deps);
		eq(log.toasts, [], 'no toast');
		eq([log.desktops[0].title, log.desktops[0].body], ['@b followed you', ''], 'desktop text');
		log.desktops[0].onclick();
		eq(log.navigated, ['/@b'], 'a follow leads to the actor');
	}

	// Hidden without permission falls back to a toast.
	{
		const { deps, log } = harness({ hidden: true, canNotify: false });
		mod.applyEvent(note(), deps);
		eq(log.toasts.length, 1, 'toast fallback');
	}

	// The body of a desktop notification prefers the content warning.
	{
		const { deps, log } = harness({ hidden: true });
		mod.applyEvent(note({ post: { id: 'p1', text: 'secret', cw: 'spoilers' } }), deps);
		eq(log.desktops[0].body, 'spoilers', 'cw replaces text');
	}

	// A signed-out page ignores notifications.
	{
		const { deps, log } = harness({ me: '' });
		mod.applyEvent(note(), deps);
		eq([log.toasts, log.desktops, log.emitted], [[], [], []], 'no viewer, no reaction');
	}

	// Message previews: plaintext kept, sealed never shown.
	{
		const { deps, log } = harness({ hidden: true });
		mod.applyEvent(message(false), deps);
		eq([log.desktops[0].title, log.desktops[0].body], ['@b sent you a message', 'lunch at noon?'], 'plaintext preview');
		log.desktops[0].onclick();
		eq(log.navigated, ['/messages/b'], 'opens the thread');
	}
	{
		const { deps, log } = harness({ hidden: true });
		mod.applyEvent(message(true), deps);
		eq([log.desktops[0].title, log.desktops[0].body], ['@b sent you a message', ''], 'sealed message has no body');
	}
	{
		const { deps, log } = harness();
		mod.applyEvent(message(true), deps);
		eq(log.toasts.map((t) => t.text), ['@b sent you a message'], 'sealed toast names only the sender');
	}

	// The conversation already on screen gets no toast, but the event still reaches Topcoat.
	{
		const { deps, log } = harness({ here: '/messages/b' });
		mod.applyEvent(message(false), deps);
		eq([log.toasts, log.emitted], [[], ['message']], 'open thread is silent');
	}

	// Badges and read.
	{
		const { deps, log } = harness();
		mod.applyEvent({ type: 'badges', data: { notifications: 3, messages: 1, reports: null } }, deps);
		eq(log.badges, { notifications: 3, messages: 1 }, 'badges set');
		mod.applyEvent({ type: 'read', data: { kind: 'notifications' } }, deps);
		eq(log.badges, { notifications: 0, messages: 1 }, 'read clears one badge');
		eq(log.emitted, ['badges', 'read'], 'both reach Topcoat');
	}

	// Cards.
	{
		const { deps, log } = harness();
		mod.applyEvent({ type: 'card', data: { postId: 'p9', card: {} } }, deps);
		eq(log.cards, ['p9'], 'card loads the fragment');
	}

	// Pure helpers.
	eq(mod.notificationText({ kind: 'like', actor }, strings, 'en'), '@b liked your post', 'notificationText');
	eq(mod.notificationPath({ post: null, actor: { handle: 'bo', domain: 'x.example' } }, 'a'), '/@bo@x.example', 'remote actor path');
	eq([mod.badgeText(0), mod.badgeText(99), mod.badgeText(100)], ['0', '99', '99+'], 'badge cap');
}
