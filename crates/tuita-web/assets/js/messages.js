// Direct messages: the conversation list, the open thread, its composer, the share-to-chat and
// new-message pickers.
// The server draws every list and thread (src/messages/);
// this island keeps them current. Keys, sealing and `conversationState` come from e2ee.js through
// `globalThis.tuitaE2ee` (that module loads first).
//
// Hooks:
// - `[data-island="chat-list"]` (`data-stream~="message"`, `data-open`): `input[data-search]` filters the
//   `a.row` links of `.rows`; a `message` event or `tuita-chat-refresh` folds in a fresh first page.
// - `[data-island="chat-thread"]` (`data-me|peer|peer-id|remote|name|handle|mode|peer-key|blocked-reason`):
//   `ol.list > li[data-id|mine|at|read]`, `a[data-older]`, `[data-chat-form]`, `[data-block]` variants of the
//   footer, `[data-note]` variants and `[data-safety]` in the page bar, `[data-act=chat-trust|chat-delete]`,
//   `#safety-number`, and the confirm dialog that fires `tuita-chat-delete`.
// - `[data-island="people-picker"]` (`data-mode=new|share`, `data-me`): the dialogs of new message and share.
// - `[data-act="share-chat"]` (a post's share menu item) opens `#share-chat`.
// Client strings: share_chat_sent, e2ee_block_*, composer_send_failed, e2ee_failed.

export const POLL_MS = 15_000;
export const GROUP_MS = 3 * 60_000;
const NEAR_PX = 160;
const DEBOUNCE_MS = 200;

/** Ids sort by time: shorter ids are older, equal lengths compare as text. */
export const compareIds = (a, b) => a.length - b.length || (a < b ? -1 : a > b ? 1 : 0);

/** Merges `incoming` into `list` (oldest first), dropping duplicates by id; later copies win. */
export function mergeMessages(list, incoming) {
	const byId = new Map();
	for (const message of list) byId.set(message.id, message);
	for (const message of incoming) byId.set(message.id, message);
	return [...byId.values()].sort((a, b) => compareIds(a.id, b.id));
}

/**
 * Runs of one sender with each message within 3 minutes of the one before. Input: `{ mine, createdAt }`
 * oldest first. Output: `{ message, first, last }`; `last` carries the time.
 */
export function groupBubbles(messages) {
	const joined = (a, b) => a.mine === b.mine && Date.parse(b.createdAt) - Date.parse(a.createdAt) <= GROUP_MS;
	return messages.map((message, i) => ({
		message,
		first: i === 0 || !joined(messages[i - 1], message),
		last: i === messages.length - 1 || !joined(message, messages[i + 1])
	}));
}

/** Id of the last own message the other side has read, or `null`. Input: `{ id, mine, readAt }`. */
export function seenId(messages) {
	for (let i = messages.length - 1; i >= 0; i--) {
		if (messages[i].mine && messages[i].readAt) return messages[i].id;
	}
	return null;
}

/**
 * Puts a freshly fetched first page (`fresh`, newest first) in front of the rows already loaded:
 * rows in the first page take their new place and content, the others keep their order below.
 * Rows are `{ id }`.
 */
export function mergeConversations(list, fresh) {
	const top = new Set(fresh.map((c) => c.id));
	return [...fresh, ...list.filter((c) => !top.has(c.id))];
}

/** Rows whose name or account contains `query`, ignoring case and a leading `@`. Rows are `{ name, acct }`. */
export function filterConversations(list, query) {
	const needle = query.trim().replace(/^@/, '').toLowerCase();
	if (!needle) return list;
	return list.filter((c) => c.name.toLowerCase().includes(needle) || c.acct.toLowerCase().includes(needle));
}

/** The other person's account name: `ada`, or `ada@remote.example`. */
export const acctOf = (who) => (who.domain ? `${who.handle}@${who.domain}` : who.handle);

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

const strings = () => {
	try {
		return JSON.parse(document.getElementById('tuita-l10n')?.textContent ?? '{}');
	} catch {
		return {};
	}
};

const toast = (detail) => document.dispatchEvent(new CustomEvent('tuita-toast', { detail }));
const e2ee = () => globalThis.tuitaE2ee;
const enc = encodeURIComponent;

async function api(method, path, body) {
	let res;
	try {
		res = await fetch(`/api/v1${path}`, {
			method,
			credentials: 'same-origin',
			headers: body === undefined ? { accept: 'application/json' } : { accept: 'application/json', 'content-type': 'application/json' },
			body: body === undefined ? undefined : JSON.stringify(body)
		});
	} catch {
		throw new Error(strings().composer_send_failed ?? 'Network error');
	}
	if (!res.ok) {
		let message = res.statusText || `HTTP ${res.status}`;
		try {
			const failure = await res.json();
			if (typeof failure.error === 'string' && failure.error) message = failure.error;
		} catch {
			// Non-JSON error bodies fall back to the status line.
		}
		throw Object.assign(new Error(message), { status: res.status });
	}
	return res.json();
}

/** A page fragment (`x-tuita-fragment: 1`) as a document fragment; its scripts stay inert. */
async function fragment(url) {
	const res = await fetch(url, { credentials: 'same-origin', headers: { 'x-tuita-fragment': '1', accept: 'text/html' } });
	if (!res.ok) throw Object.assign(new Error(String(res.status)), { status: res.status });
	const page = document.createElement('template');
	page.innerHTML = await res.text();
	return page.content;
}

const rowOf = (a) => ({ id: a.dataset.id, name: a.dataset.name, acct: a.dataset.acct, el: a });

// ---------------------------------------------------------------------------------------------
// The conversation list

function mountList(root) {
	const rows = root.querySelector('.rows');
	const search = root.querySelector('input[data-search]');
	const none = root.querySelector('[data-search-none]');
	const open = root.dataset.open ?? '';
	let busy = false;

	const applyFilter = () => {
		const all = [...rows.querySelectorAll('a.row')].map(rowOf);
		const shown = new Set(filterConversations(all, search.value).map((row) => row.id));
		for (const row of all) row.el.hidden = !shown.has(row.id);
		none.hidden = !(search.value.trim() && all.length > 0 && shown.size === 0);
	};

	const refresh = async () => {
		if (busy) return;
		busy = true;
		try {
			const page = await fragment(`/messages${open ? `?open=${enc(open)}` : ''}`);
			const fresh = [...page.querySelectorAll('a.row')].map(rowOf);
			const existing = [...rows.querySelectorAll('a.row')].map(rowOf);
			const merged = mergeConversations(existing, fresh);
			const more = rows.querySelector('a[data-more]') ?? page.querySelector('a[data-more]');
			rows.replaceChildren(...merged.map((row) => row.el), ...(more ? [more] : []));
			applyFilter();
		} catch {
			// The next event retries.
		} finally {
			busy = false;
		}
	};

	search.addEventListener('input', applyFilter);
	const observer = new MutationObserver(() => applyFilter());
	observer.observe(rows, { childList: true });
	root.addEventListener('tuita-stream', () => void refresh());
	document.addEventListener('tuita-chat-refresh', refresh);
	return () => {
		observer.disconnect();
		document.removeEventListener('tuita-chat-refresh', refresh);
	};
}

// ---------------------------------------------------------------------------------------------
// The open thread

const clock = (iso) => new Date(iso).toLocaleTimeString(document.documentElement.lang || undefined, { hour: 'numeric', minute: '2-digit' });

function mountThread(root) {
	const { keys, conversationState, sendMessage, safetyNumber } = e2ee();
	const me = root.dataset.me;
	const peer = root.dataset.peer;
	const peerId = root.dataset.peerId;
	const remote = root.dataset.remote === 'true';
	const base = () => `/conversations/${enc(peer)}`;
	const list = root.querySelector('ol.list');
	const thread = root.querySelector('.thread');
	const form = root.querySelector('[data-chat-form]');
	const input = form.querySelector('input');
	const submit = form.querySelector('button[type="submit"]');
	const footer = root.querySelector('[data-c="chat-footer"]');
	const dock = root.querySelector('.dock');
	const notes = [...document.querySelectorAll('[data-c="chat-note"] [data-note]')];
	const safetyButton = document.querySelector('[data-c="chat-note"] [data-safety]');
	const safetyList = document.querySelector('#safety-number ol.number');
	let sending = false;
	let rendering = 0;
	let served = '';
	let trusted = '';

	const items = () => [...list.querySelectorAll(':scope > li[data-id]')];
	const nearBottom = () => window.innerHeight + window.scrollY >= document.documentElement.scrollHeight - NEAR_PX;
	const toBottom = () => window.scrollTo({ top: document.documentElement.scrollHeight });

	/** Groups the items into runs, marks the seen one and writes each time in the viewer's clock. */
	const regroup = () => {
		const lis = items();
		const entries = lis.map((li) => ({ id: li.dataset.id, mine: li.dataset.mine === 'true', createdAt: li.dataset.at, readAt: li.dataset.read }));
		const seen = seenId(entries);
		groupBubbles(entries).forEach((bubble, i) => {
			const li = lis[i];
			li.classList.toggle('first', bubble.first);
			li.querySelector('.meta').hidden = !bubble.last;
			li.querySelector('.seen').hidden = li.dataset.id !== seen;
			const time = li.querySelector('time[data-clock]');
			const text = clock(li.dataset.at);
			if (time.textContent !== text) time.textContent = text;
		});
		if (lis.length > 0) thread.querySelector('.intro')?.remove();
	};

	/** Reads the state the server serves (the `[data-view]` marker of a fragment) into the root. */
	const applyView = (marker) => {
		if (!marker) return;
		for (const key of ['mode', 'peerKey', 'blockedReason']) {
			if (marker.dataset[key] === undefined) delete root.dataset[key];
			else root.dataset[key] = marker.dataset[key];
		}
	};

	const view = () => ({ mode: root.dataset.mode, blockedReason: root.dataset.blockedReason ?? null, peerKey: root.dataset.peerKey ?? null });

	/** The pinned key of the other person (`null` while the keys are not loaded) and how a message would go out. */
	const decide = async () => {
		let pin = null;
		try {
			pin = await keys.pinFor(peerId);
		} catch {
			pin = null;
		}
		return { pin, next: conversationState({ ...view(), remote, keys: keys.status, pinned: pin }) };
	};

	const show = (el, on) => {
		el.hidden = !on;
	};

	const render = async () => {
		const run = ++rendering;
		const { pin, next } = await decide();
		const v = view();
		const mine = keys.identity?.publicKey;
		const numbers = await Promise.all([
			mine && v.peerKey ? safetyNumber(mine, v.peerKey) : '',
			mine && v.peerKey && pin && pin !== v.peerKey ? safetyNumber(mine, pin) : ''
		]);
		if (run !== rendering) return;
		[served, trusted] = numbers;

		const sealed = v.mode === 'sealed' && keys.status === 'ready' && v.peerKey !== null;
		for (const note of notes) show(note, note.dataset.note === 'sealed' ? sealed : !sealed && next.note === note.dataset.note);
		if (safetyButton) show(safetyButton, Boolean(served));
		if (safetyList) safetyList.replaceChildren(...served.split(' ').filter(Boolean).map((group) => Object.assign(document.createElement('li'), { textContent: group })));

		const kind = next.block?.kind ?? null;
		for (const block of footer.querySelectorAll('[data-block]')) show(block, block.dataset.block === kind);
		if (kind === 'server') footer.querySelector('[data-block="server"]').textContent = next.block.reason;
		show(footer.querySelector('.rest'), kind !== 'setup' && kind !== 'unlock');
		const numbersBox = footer.querySelector('[data-numbers]');
		show(numbersBox, Boolean(trusted && served));
		footer.querySelector('[data-old]').textContent = trusted;
		footer.querySelector('[data-new]').textContent = served;
		dock.classList.toggle('tall', kind === 'setup' || kind === 'unlock' || kind === 'changed');

		const sendable = next.block === null;
		input.disabled = !sendable;
		submit.disabled = !sendable || sending || !input.value.trim();
	};

	/** Adds the items of a fragment that are not here yet, in order; updates the read mark of the others. Returns the new ones. */
	const merge = (page) => {
		const known = new Map(items().map((li) => [li.dataset.id, li]));
		const added = [];
		for (const li of page.querySelectorAll('li[data-id]')) {
			const have = known.get(li.dataset.id);
			if (have) have.dataset.read = li.dataset.read;
			else added.push(li);
		}
		const merged = mergeMessages(
			[...known.values()].map((li) => ({ id: li.dataset.id, li })),
			added.map((li) => ({ id: li.dataset.id, li }))
		);
		let previous = null;
		for (const entry of merged) {
			if (added.includes(entry.li)) list.insertBefore(entry.li, previous ? previous.nextSibling : list.firstChild);
			previous = entry.li;
		}
		regroup();
		return added;
	};

	const markRead = () => {
		api('POST', `${base()}/read`)
			.then(() => document.dispatchEvent(new CustomEvent('tuita-chat-refresh')))
			.catch(() => {});
	};

	/** `onlyNew` asks for the messages after the newest one shown; without it the newest page comes back, which also refreshes read receipts. */
	const fetchNewer = async (onlyNew = true) => {
		const last = items().at(-1)?.dataset.id;
		const url = `/messages/${enc(peer)}${onlyNew && last ? `?after=${enc(last)}` : ''}`;
		try {
			const page = await fragment(url);
			applyView(page.querySelector('[data-view]'));
			const stick = nearBottom();
			const added = merge(page);
			if (added.some((li) => li.dataset.mine === 'false' && li.dataset.read === '')) markRead();
			if (stick && added.length > 0) toBottom();
			void render();
		} catch {
			// The next event or poll retries.
		}
	};

	const loadOlder = async (link) => {
		if (link.getAttribute('aria-busy') === 'true') return;
		link.setAttribute('aria-busy', 'true');
		try {
			const page = await fragment(link.getAttribute('href'));
			const height = document.documentElement.scrollHeight;
			merge(page);
			const next = page.querySelector('a[data-older]');
			if (next) link.setAttribute('href', next.getAttribute('href'));
			else link.closest('.older')?.remove();
			link.removeAttribute('aria-busy');
			window.scrollBy(0, document.documentElement.scrollHeight - height);
		} catch (error) {
			link.removeAttribute('aria-busy');
			toast(error.message);
		}
	};

	const send = async () => {
		const body = input.value.trim();
		if (!body || sending || (await decide()).next.send === null) return;
		sending = true;
		void render();
		try {
			const result = await sendMessage(body, view(), {
				me: { id: me, identity: keys.identity, keys: keys.status },
				peer: { id: peerId, remote },
				pinned: () => keys.pinFor(peerId),
				pin: (key) => keys.trust(peerId, key),
				post: (request) => api('POST', base(), request),
				fetchView: () => api('GET', `${base()}?limit=1`)
			});
			root.dataset.mode = result.view.mode;
			if (result.view.peerKey === null) delete root.dataset.peerKey;
			else root.dataset.peerKey = result.view.peerKey;
			if (result.view.blockedReason === null) delete root.dataset.blockedReason;
			else root.dataset.blockedReason = result.view.blockedReason;
			if (result.sent) {
				input.value = '';
				await fetchNewer(true);
				document.dispatchEvent(new CustomEvent('tuita-chat-refresh'));
				toBottom();
			}
		} catch (error) {
			toast(error.message);
		} finally {
			sending = false;
			void render();
		}
	};

	let target = null;
	const confirmDelete = async () => {
		const li = target;
		target = null;
		if (!li) return;
		try {
			await api('DELETE', `${base()}/messages/${enc(li.dataset.id)}`);
			li.remove();
			regroup();
			document.dispatchEvent(new CustomEvent('tuita-chat-refresh'));
		} catch (error) {
			toast(error.message);
		}
	};

	form.addEventListener('submit', (event) => {
		event.preventDefault();
		void send();
	});
	input.addEventListener('input', () => void render());
	root.addEventListener('click', (event) => {
		const el = event.target instanceof Element ? event.target : null;
		const older = el?.closest('a[data-older]');
		if (older) {
			event.preventDefault();
			void loadOlder(older);
			return;
		}
		const act = el?.closest('[data-act]');
		if (act?.dataset.act === 'chat-trust') {
			const key = root.dataset.peerKey;
			if (key) void keys.trust(peerId, key).then(render);
		} else if (act?.dataset.act === 'chat-delete') {
			target = act.closest('li[data-id]');
			document.getElementById('confirm-chat-delete')?.showModal();
		}
	});
	root.addEventListener('tuita-stream', (event) => {
		const from = event.detail?.data?.from;
		if (from && acctOf(from).toLowerCase() === peer.toLowerCase()) void fetchNewer(true);
	});
	window.addEventListener('tuita-chat-delete', confirmDelete);
	document.addEventListener('tuita-e2ee', render);
	const timer = setInterval(() => void fetchNewer(false), POLL_MS);

	regroup();
	toBottom();
	markRead();
	void keys.ensure(me).then(render);
	void render();
	return () => {
		clearInterval(timer);
		window.removeEventListener('tuita-chat-delete', confirmDelete);
		document.removeEventListener('tuita-e2ee', render);
	};
}

// ---------------------------------------------------------------------------------------------
// The people pickers

function mountPicker(root) {
	const { keys, sendMessage, blockText } = e2ee();
	const dialog = root.closest('dialog');
	const input = root.querySelector('input[type="search"]');
	const results = root.querySelector('.results');
	const error = root.querySelector('[data-error]');
	const mode = root.dataset.mode;
	let timer = 0;
	let run = 0;
	let sending = false;

	const fail = (message) => {
		error.textContent = message ?? '';
		error.hidden = !message;
	};
	const reset = () => {
		clearTimeout(timer);
		run++;
		input.value = '';
		results.replaceChildren();
		fail('');
	};

	const search = async () => {
		const q = input.value.trim();
		const mine = ++run;
		if (!q) {
			results.replaceChildren();
			fail('');
			return;
		}
		try {
			const page = await fragment(`/_tuita/chat-people?q=${enc(q)}`);
			if (mine !== run) return;
			results.replaceChildren(page);
			fail('');
		} catch {
			if (mine === run) fail(strings().e2ee_failed);
		}
	};

	const share = async (pick) => {
		if (sending) return;
		sending = true;
		fail('');
		try {
			const path = `/conversations/${enc(pick.dataset.acct)}`;
			await keys.ensure(root.dataset.me);
			const first = await api('GET', `${path}?limit=1`);
			const result = await sendMessage(dialog.dataset.postUrl, first, {
				me: { id: root.dataset.me, identity: keys.identity, keys: keys.status },
				peer: { id: pick.dataset.id, remote: false },
				pinned: () => keys.pinFor(pick.dataset.id),
				pin: (key) => keys.trust(pick.dataset.id, key),
				post: (body) => api('POST', path, body),
				fetchView: () => api('GET', `${path}?limit=1`)
			});
			if (!result.sent) {
				fail(blockText(result.state.block, pick.dataset.handle));
				return;
			}
			toast(strings().share_chat_sent.replace('{handle}', pick.dataset.handle));
			dialog.close();
		} catch (caught) {
			fail(caught.message);
		} finally {
			sending = false;
		}
	};

	input.addEventListener('input', () => {
		clearTimeout(timer);
		timer = setTimeout(search, DEBOUNCE_MS);
	});
	results.addEventListener('click', (event) => {
		const pick = event.target instanceof Element ? event.target.closest('[data-pick]') : null;
		if (!pick) return;
		if (mode === 'new') location.assign(`/messages/${enc(pick.dataset.acct)}`);
		else void share(pick);
	});
	dialog.addEventListener('close', reset);
	return () => clearTimeout(timer);
}

if (typeof document !== 'undefined' && !document.documentElement.dataset.messagesBound) {
	document.documentElement.dataset.messagesBound = '';
	mount('[data-island="chat-list"]', mountList);
	mount('[data-island="chat-thread"]', mountThread);
	mount('[data-island="people-picker"]', mountPicker);

	// A post's share menu: "Send via chat" opens the picker with the post's link as the message.
	document.addEventListener('click', (event) => {
		const act = event.target instanceof Element ? event.target.closest('[data-act="share-chat"]') : null;
		const dialog = document.getElementById('share-chat');
		if (!act || !dialog) return;
		const path = act.closest('[popover]')?.querySelector('[data-act="copy-link"]')?.dataset.url;
		if (!path) return;
		dialog.dataset.postUrl = `${location.origin}${path}`;
		dialog.showModal();
		dialog.querySelector('input[type="search"]')?.focus();
	});
}
