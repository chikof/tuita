// The notification list, `[data-island="notifications"]` (also `data-stream="notification"` and
// `data-filter="all|mentions"`): it marks the notifications read once the page shows, and on a live
// `notification` event fetches the first page again as a fragment and swaps the list.

/** Kinds the Mentions tab lists. */
export const MENTION_KINDS = ['reply', 'mention', 'quote'];

/** Whether a notification with `kind` changes the list of `filter`. */
export const affects = (filter, kind) => filter !== 'mentions' || MENTION_KINDS.includes(kind);

/** Wait after the last notification before refreshing: a burst makes one fetch. */
export const DEBOUNCE_MS = 400;

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

async function markRead() {
	try {
		const response = await fetch('/api/v1/notifications/read', {
			method: 'POST',
			credentials: 'same-origin',
			headers: { accept: 'application/json' }
		});
		if (!response.ok) return;
		for (const badge of document.querySelectorAll('[data-badge="notifications"]')) badge.hidden = true;
	} catch {
		// The next visit marks them.
	}
}

async function refresh(root) {
	const list = root.querySelector('[role="list"]');
	if (!list) return;
	const url = new URL(location.href);
	url.searchParams.delete('before');
	try {
		const response = await fetch(url, {
			credentials: 'same-origin',
			headers: { 'x-tuita-fragment': '1', accept: 'text/html' }
		});
		if (!response.ok) return;
		const unseen = new Set([...root.querySelectorAll('[data-unread][data-group-id]')].map((row) => row.dataset.groupId));
		// A template's content is inert: no script in the fragment runs.
		const page = document.createElement('template');
		page.innerHTML = await response.text();
		for (const row of page.content.querySelectorAll('[data-group-id]')) {
			if (!unseen.has(row.dataset.groupId)) continue;
			row.dataset.unread = '';
			row.classList.add('unread');
		}
		list.replaceChildren(page.content);
		root.querySelector('[data-empty]')?.remove();
		void markRead();
	} catch {
		// The list stays as it was; the next event tries again.
	}
}

if (typeof document !== 'undefined' && !document.documentElement.dataset.notificationsBound) {
	document.documentElement.dataset.notificationsBound = '';
	mount('[data-island="notifications"]', (root) => {
		void markRead();
		let timer;
		const onStream = (event) => {
			const note = event.detail?.type === 'notification' ? event.detail.data : null;
			if (!note || !affects(root.dataset.filter, note.kind)) return;
			clearTimeout(timer);
			timer = setTimeout(() => void refresh(root), DEBOUNCE_MS);
		};
		root.addEventListener('tuita-stream', onStream);
		return () => {
			clearTimeout(timer);
			root.removeEventListener('tuita-stream', onStream);
		};
	});
}
