// The live stream. `[data-island="stream"][data-handle="<viewer handle>"]` (members only) starts one
// EventSource on /api/v1/stream. Events update `[data-badge]` counters, toast or notify, and reach
// Topcoat as bubbling `tuita-stream` events on every `[data-stream~="<type>"]`.
//
// Client strings used: notify_<kind> (like, repost, reply, mention, quote, follow, follow_request,
// poll, affiliation), notify_message, view, badge_unread (plural), settings_desktop_state,
// settings_desktop_allowed, settings_desktop_blocked, settings_desktop_not_asked,
// settings_desktop_blocked_toast.

/** Wait before reconnect attempt 1, 2, 3...; the last value repeats. */
export const BACKOFF_MS = [1000, 2000, 5000, 10_000, 30_000];

export const STREAM_EVENTS = ['badges', 'notification', 'message', 'read', 'card'];

/**
 * Keeps one `EventSource` alive. After any error the source is closed and reopened with the
 * backoff above; a successful open resets the delay. Returns the function that stops it.
 */
export function openStream(url, deps) {
	let source;
	let cancel;
	let attempt = 0;
	let stopped = false;

	const connect = () => {
		const current = deps.open(url);
		source = current;
		current.onopen = () => (attempt = 0);
		for (const type of STREAM_EVENTS) {
			current.addEventListener(type, (message) => {
				try {
					deps.onEvent(JSON.parse(message.data));
				} catch {
					// A malformed frame is dropped; the next one is independent.
				}
			});
		}
		current.onerror = () => {
			current.close();
			if (stopped || source !== current) return;
			const wait = BACKOFF_MS[Math.min(attempt, BACKOFF_MS.length - 1)];
			attempt++;
			cancel = deps.later(connect, wait);
		};
	};

	connect();
	return () => {
		stopped = true;
		cancel?.();
		source?.close();
	};
}

/** Fills `{name}` placeholders; a plural entry picks the sample in the same category as `vars.count`. */
export function text(entry, vars = {}, lang = document.documentElement.lang) {
	let template = entry;
	if (entry && typeof entry === 'object') {
		const rules = new Intl.PluralRules(lang);
		const want = rules.select(vars.count);
		const samples = Object.entries(entry);
		template = (samples.find(([sample]) => rules.select(Number(sample)) === want) ?? samples.at(-1))[1];
	}
	return String(template).replace(/\{(\w+)\}/g, (hole, name) =>
		name in vars ? (typeof vars[name] === 'number' ? new Intl.NumberFormat(lang).format(vars[name]) : String(vars[name])) : hole
	);
}

/** `@b liked your post`. */
export const notificationText = (n, strings, lang) => text(strings[`notify_${n.kind}`], { handle: n.actor.handle }, lang);

/** Where a notification leads: the post it is about, else the actor. */
export const notificationPath = (n, me) =>
	n.post ? `/@${me}/${n.post.id}` : `/@${n.actor.handle}${n.actor.domain ? `@${n.actor.domain}` : ''}`;

/** The shown count of a badge: capped at the badge's display limit. */
export const badgeText = (n, max = 99) => (n > max ? `${max}+` : String(n));

const acctOf = (who) => (who.domain ? `${who.handle}@${who.domain}` : who.handle);

/**
 * Applies one stream event. `deps`: `strings`, `lang`, `me` (viewer handle), `here()`, `hidden()`,
 * `canNotify()`, `desktop(title, body, onclick)`, `toast(message, {label, href})`,
 * `navigate(path)`, `setBadge(kind, n)`, `emit(type, data)`, `card(postId)`.
 */
export function applyEvent(event, deps) {
	const { strings, lang } = deps;
	const view = { label: strings.view };
	switch (event.type) {
		case 'badges':
			deps.setBadge('notifications', Number(event.data.notifications));
			deps.setBadge('messages', Number(event.data.messages));
			deps.emit('badges', event.data);
			return;
		case 'read':
			deps.setBadge(event.data.kind, 0);
			deps.emit('read', event.data);
			return;
		case 'notification': {
			if (!deps.me) return;
			const note = event.data;
			const message = notificationText(note, strings, lang);
			const path = notificationPath(note, deps.me);
			deps.emit('notification', note);
			if (deps.hidden() && deps.canNotify()) {
				deps.desktop(message, note.post?.cw || note.post?.text || '', () => deps.navigate(path));
			} else {
				deps.toast(message, { ...view, href: path });
			}
			return;
		}
		case 'message': {
			const peer = acctOf(event.data.from);
			deps.emit('message', event.data);
			if (deps.here() === `/messages/${peer}`) return;
			const path = `/messages/${peer}`;
			const message = text(strings.notify_message, { handle: event.data.from.handle }, lang);
			// A sealed message has no preview to show: only the browser can read it, and only inside the thread.
			const body = event.data.encrypted ? '' : event.data.text;
			if (deps.hidden() && deps.canNotify()) deps.desktop(message, body, () => deps.navigate(path));
			else deps.toast(message, { ...view, href: path });
			return;
		}
		case 'card':
			deps.emit('card', event.data);
			deps.card(event.data.postId);
			return;
	}
}

function mount(selector, init) {
	const live = new Map();
	const scan = (root) => {
		if (root.nodeType !== 1) return;
		const found = root.matches(selector) ? [root, ...root.querySelectorAll(selector)] : root.querySelectorAll(selector);
		for (const el of found) if (!live.has(el)) live.set(el, init(el) ?? null);
	};
	const sweep = () => { for (const [el, stop] of live) if (!el.isConnected) { stop?.(); live.delete(el); } };
	scan(document.documentElement);
	new MutationObserver((records) => {
		for (const r of records) for (const n of r.addedNodes) scan(n);
		sweep();
	}).observe(document.documentElement, { childList: true, subtree: true });
}

const clientStrings = () => JSON.parse(document.getElementById('tuita-l10n')?.textContent ?? '{}');

/** A message, or a message with an action button (`{label, href}`): the two shapes toast.js takes. */
function toast(message, action) {
	document.dispatchEvent(new CustomEvent('tuita-toast', { detail: action ? { message, ...action } : message }));
}

/** Topcoat reads event payloads from `ev.data`. */
function emit(type, data) {
	const detail = { type, data };
	for (const el of document.querySelectorAll(`[data-stream~="${type}"]`)) {
		const ev = new CustomEvent('tuita-stream', { bubbles: true, detail });
		ev.data = JSON.stringify(detail);
		el.dispatchEvent(ev);
	}
}

function setBadge(kind, n) {
	const strings = clientStrings();
	for (const badge of document.querySelectorAll(`[data-badge="${kind}"]`)) {
		badge.hidden = n <= 0;
		for (const count of badge.querySelectorAll('[data-count]')) count.textContent = badgeText(n);
		for (const label of badge.querySelectorAll('[data-label]')) label.textContent = text(strings.badge_unread, { count: n });
	}
}

async function loadCard(postId) {
	try {
		const response = await fetch(`/_tuita/card/${encodeURIComponent(postId)}`, { credentials: 'same-origin' });
		if (!response.ok) return;
		const html = await response.text();
		if (!html.trim()) return;
		for (const slot of document.querySelectorAll(`[data-post-id="${CSS.escape(postId)}"] [data-slot="card"]`)) {
			const page = document.createElement('template');
			page.innerHTML = html;
			slot.replaceChildren(page.content);
		}
	} catch {
		// The preview stays absent; the post is complete without it.
	}
}

function liveDeps(me) {
	return {
		strings: clientStrings(),
		lang: document.documentElement.lang,
		me,
		here: () => location.pathname,
		hidden: () => document.hidden,
		canNotify: () => typeof Notification !== 'undefined' && Notification.permission === 'granted',
		desktop: (title, body, onclick) => {
			const popup = new Notification(title, { body, icon: '/icon-192.png' });
			popup.onclick = () => {
				window.focus();
				onclick();
				popup.close();
			};
		},
		toast,
		navigate: (path) => location.assign(path),
		setBadge,
		emit,
		card: loadCard
	};
}

const PERMISSION_LABEL = {
	granted: 'settings_desktop_allowed',
	denied: 'settings_desktop_blocked',
	default: 'settings_desktop_not_asked'
};

/**
 * `[data-island="desktop-notifications"]`: the root, or a wrapper holding `[data-action="enable"]`,
 * reflects `Notification.permission` in `data-state` (`default`, `granted`, `denied`, `unsupported`).
 * The enable button is hidden when granted. Descendants with `data-when="<states>"` show only in
 * those states, and `[data-state-label]` receives the localized "Permission: …" text.
 */
function desktopNotifications(root) {
	const strings = clientStrings();
	const button = root.matches('button') ? root : root.querySelector('[data-action="enable"]');
	const state = () => (typeof Notification === 'undefined' ? 'unsupported' : Notification.permission);
	const sync = () => {
		const now = state();
		root.dataset.state = now;
		if (button) button.hidden = now === 'granted' || now === 'unsupported';
		for (const el of root.querySelectorAll('[data-when]')) el.hidden = !el.dataset.when.split(/\s+/).includes(now);
		for (const el of root.querySelectorAll('[data-state-label]')) {
			el.textContent = text(strings.settings_desktop_state, { state: text(strings[PERMISSION_LABEL[now]] ?? '') });
		}
	};
	const enable = async () => {
		if (state() === 'unsupported') return;
		const permission = await Notification.requestPermission();
		sync();
		if (permission === 'denied') toast(strings.settings_desktop_blocked_toast);
	};
	button?.addEventListener('click', enable);
	window.addEventListener('focus', sync);
	sync();
	return () => window.removeEventListener('focus', sync);
}

if (typeof document !== 'undefined' && !document.documentElement.dataset.streamBound) {
	document.documentElement.dataset.streamBound = '';
	let stop = null;
	mount('[data-island="stream"]', (root) => {
		if (stop) return undefined;
		const deps = liveDeps(root.dataset.handle ?? '');
		const start = () =>
			openStream('/api/v1/stream', {
				open: (url) => new EventSource(url, { withCredentials: true }),
				later: (run, ms) => {
					const timer = setTimeout(run, ms);
					return () => clearTimeout(timer);
				},
				onEvent: (event) => applyEvent(event, deps)
			});
		stop = start();
		return () => {
			stop?.();
			stop = null;
		};
	});
	// A page that goes into the back/forward cache must not keep its stream open: the browser caps connections per
	// host, so a few cached pages would block every new request. Coming back, the page is reloaded for fresh data.
	addEventListener('pagehide', () => {
		stop?.();
		stop = null;
	});
	addEventListener('pageshow', (event) => {
		if (event.persisted) location.reload();
	});
	mount('[data-island="desktop-notifications"]', desktopNotifications);
}
