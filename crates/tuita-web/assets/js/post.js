// Posts: like, repost and bookmark, polls, content warnings, sensitive covers, the post menu, the
// dialogs of a post and the click-through of a card. The server draws every post as static markup
// with `data-*` hooks (src/post/*.rs); this island makes it work, because markup inserted by an
// island is not hydrated by Topcoat and the same card appears in feeds, threads and dialogs.
//
// Hooks: `.actions[data-post-id]` with `data-liked|reposted|bookmarked|likes|reposts|big`,
// `[data-action=like|bookmark|reply]`, `[data-act=repost]`; `.poll[data-poll]`; `[data-act]` buttons
// (cw, sensitive-show/hide, vote, vote-submit, goto, pin, delete, hide, mute, block, edit, history,
// report, list-picker, post-list); `article[data-post][data-href]`; `form[data-post-form]`.
// Copy, share and download are share.js; opening the compose dialog is the composer.

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

const toast = (detail) => document.dispatchEvent(new CustomEvent('tuita-toast', { detail }));

function strings() {
	try {
		return JSON.parse(document.getElementById('tuita-l10n')?.textContent ?? '{}');
	} catch {
		return {};
	}
}

const lang = () => document.documentElement.lang || 'en';

/** Fills `{name}` slots of a client string. */
export function fill(template, values) {
	return String(template ?? '').replace(/\{(\w+)\}/g, (hit, name) => (name in values ? String(values[name]) : hit));
}

/** A plural client string for `count`: the sample whose plural category matches the count's. */
export function pluralText(entry, count, language = lang()) {
	if (typeof entry === 'string') return fill(entry, { count });
	const rules = new Intl.PluralRules(language);
	const wanted = rules.select(count);
	const sample = Object.keys(entry).find((key) => rules.select(Number(key)) === wanted) ?? Object.keys(entry)[0];
	return fill(entry[sample], { count });
}

export const compact = (n, language = lang()) =>
	new Intl.NumberFormat(language, { notation: 'compact', maximumFractionDigits: 1 }).format(n);

/** Which flag and which counter each toggle changes. */
export const FLAGS = {
	like: ['liked', 'likes'],
	repost: ['reposted', 'reposts'],
	bookmark: ['bookmarked', null]
};

/** The state a bar shows: what the server sent for a post, or what a toggle makes of it. */
export const stateOf = (post) => ({
	liked: post.viewer.liked,
	likes: post.counts.likes,
	reposted: post.viewer.reposted,
	reposts: post.counts.reposts + post.counts.quotes,
	bookmarked: post.viewer.bookmarked
});

/** The state after flipping `kind`, counters moved by one. */
export function flip(state, kind) {
	const [flag, count] = FLAGS[kind];
	const on = !state[flag];
	return { ...state, [flag]: on, ...(count ? { [count]: state[count] + (on ? 1 : -1) } : {}) };
}

const LINK = /https?:\/\/[^\s<>"]+[^\s<>".,:;'!?)\]]/g;

/** The length of a post: every link counts 23 characters. */
export function postLength(text) {
	let urls = 0;
	const rest = text.replace(LINK, () => {
		urls++;
		return '';
	});
	return [...rest].length + urls * 23;
}

const MAX_POST = 500;

async function api(method, path, body, keepalive = false) {
	const response = await fetch(`/api/v1${path}`, {
		method,
		keepalive,
		credentials: 'same-origin',
		headers: body === undefined ? { accept: 'application/json' } : { accept: 'application/json', 'content-type': 'application/json' },
		body: body === undefined ? undefined : JSON.stringify(body)
	});
	let data = null;
	try {
		data = await response.json();
	} catch {
		data = null;
	}
	if (!response.ok) throw new Error(data?.error ?? response.statusText ?? String(response.status));
	return data;
}

const messageOf = (error) => (error instanceof Error ? error.message : String(error));
const enc = encodeURIComponent;

async function fragment(url) {
	const response = await fetch(url, { credentials: 'same-origin', headers: { accept: 'text/html' } });
	if (!response.ok || response.status === 204) throw new Error(String(response.status));
	const holder = document.createElement('template');
	holder.innerHTML = await response.text();
	return holder.content;
}

// ---------------------------------------------------------------------------------------------
// Like, repost, bookmark

const bars = (id) => [...document.querySelectorAll(`[data-c="post-actions"][data-post-id="${CSS.escape(id)}"]`)];

/** Draws `state` on every action bar of post `id`. `pop` plays the heart's fill. */
export function paint(id, state, pop = false) {
	const text = strings();
	for (const bar of bars(id)) {
		const big = bar.dataset.big === 'true';
		bar.dataset.liked = String(state.liked);
		bar.dataset.likes = String(state.likes);
		bar.dataset.reposted = String(state.reposted);
		bar.dataset.reposts = String(state.reposts);
		bar.dataset.bookmarked = String(state.bookmarked);
		const count = (el, n) => {
			if (el) el.textContent = big || n === 0 ? '' : compact(n);
		};
		const like = bar.querySelector('[data-action="like"]');
		if (like) {
			like.classList.toggle('on', state.liked);
			like.setAttribute('aria-pressed', String(state.liked));
			like.setAttribute('aria-label', fill(text.post_like_label, { count: new Intl.NumberFormat(lang()).format(state.likes) }));
			count(like.querySelector('.count'), state.likes);
			const heart = like.querySelector('[data-c="heart"]');
			heart?.classList.toggle('filled', state.liked);
			heart?.classList.toggle('pop', state.liked && pop);
		}
		const repost = bar.querySelector('span[data-action="repost"]');
		if (repost) {
			repost.classList.toggle('on', state.reposted);
			count(repost.querySelector('.count'), state.reposts);
			const sr = repost.querySelector('.sr-only');
			if (sr) sr.textContent = fill(text.post_reposts_label, { count: new Intl.NumberFormat(lang()).format(state.reposts) });
		}
		const item = bar.querySelector('[data-act="repost"]');
		if (item) {
			const label = item.querySelector('span');
			if (label) label.textContent = state.reposted ? item.dataset.on : item.dataset.off;
		}
		const bookmark = bar.querySelector('[data-action="bookmark"]');
		if (bookmark) {
			bookmark.classList.toggle('on', state.bookmarked);
			bookmark.setAttribute('aria-pressed', String(state.bookmarked));
		}
	}
}

const readState = (bar) => ({
	liked: bar.dataset.liked === 'true',
	likes: Number(bar.dataset.likes),
	reposted: bar.dataset.reposted === 'true',
	reposts: Number(bar.dataset.reposts),
	bookmarked: bar.dataset.bookmarked === 'true'
});

const inflight = new Set();

/** Flips like, repost or bookmark at once and rolls back when the server refuses. */
async function toggle(bar, kind) {
	const id = bar.dataset.postId;
	const key = `${id}:${kind}`;
	if (inflight.has(key)) return;
	inflight.add(key);
	const before = readState(bar);
	const next = flip(before, kind);
	const [flag, count] = FLAGS[kind];
	// Only this toggle's own fields are settled from the answer or rolled back: another toggle of the
	// same post may be in flight, and its optimistic state must survive.
	const settle = (source) => {
		const now = readState(bar);
		paint(id, { ...now, [flag]: source[flag], ...(count ? { [count]: source[count] } : {}) });
	};
	for (const el of bars(id)) el.dataset.pending = String(Number(el.dataset.pending ?? 0) + 1);
	paint(id, next, kind === 'like');
	try {
		// keepalive: leaving the page right after a click must not cancel the request.
		const post = await api(next[flag] ? 'POST' : 'DELETE', `/posts/${enc(id)}/${kind}`, undefined, true);
		settle(stateOf(post));
	} catch (error) {
		settle(before);
		toast(messageOf(error));
	} finally {
		inflight.delete(key);
		for (const el of bars(id)) {
			const left = Number(el.dataset.pending ?? 1) - 1;
			if (left > 0) el.dataset.pending = String(left);
			else delete el.dataset.pending;
		}
	}
}

// ---------------------------------------------------------------------------------------------
// Dialogs

/** Fetches a dialog fragment into `#post-dialogs` and opens it modally; it removes itself on close. */
async function openDialog(url) {
	try {
		const content = await fragment(url);
		const dialog = content.querySelector('dialog');
		const host = document.getElementById('post-dialogs');
		if (!dialog || !host) return null;
		host.replaceChildren(dialog);
		dialog.addEventListener('close', () => dialog.remove(), { once: true });
		dialog.showModal();
		return dialog;
	} catch (error) {
		toast(messageOf(error));
		return null;
	}
}

function showFailure(form, message) {
	const failure = form.querySelector('.failure');
	if (!failure) return;
	failure.textContent = message;
	failure.hidden = !message;
}

function syncEdit(form) {
	const text = form.querySelector('[name="text"]');
	const cw = form.querySelector('[name="cw"]');
	const save = document.querySelector('[data-act="edit-save"]');
	const trimmed = text.value.trim();
	const length = postLength(trimmed);
	const empty = trimmed === '' && !('keeps' in form.dataset);
	const changed = trimmed !== form.dataset.text || cw.value.trim() !== form.dataset.cw;
	if (save) save.disabled = empty || !changed || length > MAX_POST;
	const counter = form.querySelector('[data-c="char-counter"]');
	if (counter) {
		const left = MAX_POST - length;
		const over = left < 0;
		const rects = counter.querySelectorAll('rect');
		const filled = Math.min(rects.length, Math.ceil((Math.max(0, length) / MAX_POST) * rects.length));
		rects.forEach((rect, i) => rect.classList.toggle('on', i < filled));
		counter.classList.toggle('over', over);
		const text2 = strings();
		const entry = over ? text2.char_counter_over : text2.char_counter_left;
		counter.setAttribute('aria-label', pluralText(entry, Math.abs(left)));
		let number = counter.querySelector('.number');
		if (left <= 20) {
			if (!number) {
				number = document.createElement('span');
				number.className = 'number';
				number.setAttribute('aria-hidden', 'true');
				counter.append(number);
			}
			number.textContent = String(left);
		} else number?.remove();
	}
}

async function submitForm(form) {
	const kind = form.dataset.postForm;
	const submit = document.querySelector(`[form="${form.id}"][type="submit"]`);
	if (submit?.disabled) return;
	showFailure(form, '');
	if (submit) submit.setAttribute('aria-busy', 'true');
	try {
		if (kind === 'edit') {
			await api('PATCH', `/posts/${enc(form.dataset.postId)}`, {
				text: form.querySelector('[name="text"]').value.trim(),
				cw: form.querySelector('[name="cw"]').value.trim()
			});
			toast(strings().edit_saved);
			form.closest('dialog')?.close();
			await refreshCards(form.dataset.postId);
		} else if (kind === 'report') {
			const reason = form.querySelector('[name="reason"]:checked')?.value;
			if (!reason) return;
			const note = form.querySelector('[name="note"]').value.trim();
			const body = form.dataset.postId ? { postId: form.dataset.postId, reason, note } : { handle: form.dataset.handle, reason, note };
			const ack = await api('POST', '/reports', body);
			toast(ack?.duplicate ? strings().report_duplicate : strings().report_sent);
			form.closest('dialog')?.close();
		}
	} catch (error) {
		showFailure(form, messageOf(error));
	} finally {
		submit?.removeAttribute('aria-busy');
	}
}

// ---------------------------------------------------------------------------------------------
// Cards

/** Replaces every card of post `id` with a fresh one, keeping how each was drawn. */
async function refreshCards(id) {
	for (const old of document.querySelectorAll(`article[data-post="${CSS.escape(id)}"]`)) {
		const query = new URLSearchParams();
		if ('main' in old.dataset) query.set('main', '1');
		if (old.dataset.thread) query.set('thread', old.dataset.thread);
		if ('hideReply' in old.dataset) query.set('hide_reply', '1');
		try {
			const content = await fragment(`/_tuita/post/${enc(id)}?${query}`);
			const fresh = content.querySelector('article');
			if (!fresh) continue;
			const context = old.querySelector(':scope > .context');
			if (context && !fresh.querySelector(':scope > .context')) fresh.prepend(context);
			old.replaceWith(fresh);
		} catch {
			// The old card stays; a failed refresh is not worth a toast.
		}
	}
}

const forget = (selector) => {
	for (const el of document.querySelectorAll(selector)) el.remove();
};

/** The action the open confirmation runs when confirmed. */
let pending = null;

function ask(dialogId, action) {
	pending = action;
	document.getElementById(dialogId)?.showModal();
}

async function removePost(id) {
	try {
		await api('DELETE', `/posts/${enc(id)}`);
		const main = document.querySelector(`article[data-post="${CSS.escape(id)}"][data-main]`);
		toast(strings().post_deleted);
		forget(`article[data-post="${CSS.escape(id)}"]`);
		if (main) location.assign('/');
	} catch (error) {
		toast(messageOf(error));
	}
}

async function pin(button) {
	const id = button.dataset.postId;
	const pinned = button.dataset.pinned === 'true';
	try {
		await api(pinned ? 'DELETE' : 'POST', `/posts/${enc(id)}/pin`);
		toast(pinned ? strings().post_unpinned : strings().post_pinned_toast);
		await refreshCards(id);
	} catch (error) {
		toast(messageOf(error));
	}
}

async function hide(button) {
	const id = button.dataset.postId;
	const hidden = button.dataset.hidden === 'true';
	try {
		const post = await api(hidden ? 'DELETE' : 'POST', `/posts/${enc(id)}/hide`);
		toast(post.hiddenReply ? strings().post_reply_hidden_toast : strings().post_reply_unhidden_toast);
		await refreshCards(id);
	} catch (error) {
		toast(messageOf(error));
	}
}

async function relate(kind, button) {
	const { acct, userId, handle } = button.dataset;
	try {
		await api('POST', `/users/${enc(acct)}/${kind}`);
		forget(`article[data-author-id="${CSS.escape(userId)}"]`);
		toast(fill(kind === 'mute' ? strings().muted_toast : strings().blocked_toast, { handle }));
	} catch (error) {
		toast(messageOf(error));
	}
}

// ---------------------------------------------------------------------------------------------
// Polls

async function vote(poll, choices) {
	if (poll.dataset.busy || choices.length === 0) return;
	poll.dataset.busy = 'true';
	const id = poll.dataset.poll;
	for (const button of poll.querySelectorAll('button')) button.disabled = true;
	try {
		await api('POST', `/posts/${enc(id)}/vote`, { choices });
		for (const open of document.querySelectorAll(`.poll[data-poll="${CSS.escape(id)}"]`)) {
			const fresh = (await fragment(`/_tuita/poll/${enc(id)}`)).querySelector('.poll');
			if (fresh) open.replaceWith(fresh);
		}
	} catch (error) {
		delete poll.dataset.busy;
		for (const button of poll.querySelectorAll('button')) button.disabled = false;
		syncVote(poll);
		toast(messageOf(error));
	}
}

const picked = (poll) => [...poll.querySelectorAll('[data-act="vote"].on')].map((button) => Number(button.dataset.index));

function syncVote(poll) {
	const submit = poll.querySelector('[data-act="vote-submit"]');
	if (submit) submit.disabled = picked(poll).length === 0;
}

function choose(button) {
	const poll = button.closest('.poll');
	if (!poll) return;
	if (!('multiple' in poll.dataset)) return void vote(poll, [Number(button.dataset.index)]);
	const on = !button.classList.contains('on');
	button.classList.toggle('on', on);
	button.setAttribute('aria-pressed', String(on));
	button.querySelector('.tick')?.toggleAttribute('hidden', !on);
	syncVote(poll);
}

// ---------------------------------------------------------------------------------------------
// Covers and warnings

function toggleWarning(button) {
	const open = button.getAttribute('aria-expanded') !== 'true';
	const root = button.closest('[data-c="content-warning"]');
	button.setAttribute('aria-expanded', String(open));
	button.textContent = open ? button.dataset.hide : button.dataset.show;
	const body = document.getElementById(button.getAttribute('aria-controls'));
	if (body) body.hidden = !open;
	const veil = root?.querySelector(':scope > .veil');
	if (veil) veil.hidden = open;
}

function toggleCover(cover, shown) {
	const content = cover.querySelector(':scope > .content');
	cover.classList.toggle('shown', shown);
	if (content) {
		content.inert = !shown;
		content.setAttribute('aria-hidden', String(!shown));
	}
	cover.querySelector(':scope > .veil')?.toggleAttribute('hidden', shown);
	cover.querySelector(':scope > .hide')?.toggleAttribute('hidden', !shown);
}

// ---------------------------------------------------------------------------------------------
// Events

function openCard(event, card) {
	if (event.defaultPrevented || event.button !== 0) return;
	if (event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
	const hit = event.target.closest('a, button, input, textarea, dialog, summary, [role=menu], [data-c="video-player"]');
	if (hit && card.contains(hit)) return;
	if (getSelection()?.toString()) return;
	location.assign(card.dataset.href);
}

const ACTS = {
	cw: (button) => toggleWarning(button),
	'sensitive-show': (button) => toggleCover(button.closest('[data-c="sensitive-cover"]'), true),
	'sensitive-hide': (button) => toggleCover(button.closest('[data-c="sensitive-cover"]'), false),
	vote: (button) => choose(button),
	'vote-submit': (button) => {
		const poll = button.closest('.poll');
		if (poll) void vote(poll, picked(poll));
	},
	repost: (button) => {
		const bar = button.closest('[data-c="post-actions"]');
		if (bar) void toggle(bar, 'repost');
	},
	goto: (button) => location.assign(button.dataset.url),
	pin: (button) => void pin(button),
	hide: (button) => void hide(button),
	delete: (button) => ask('confirm-delete-post', { kind: 'delete-post', run: () => removePost(button.dataset.postId) }),
	mute: (button) => void relate('mute', button),
	block: (button) => {
		const title = document.getElementById('confirm-block-title');
		if (title) title.textContent = fill(strings().block_title, { handle: button.dataset.handle });
		ask('confirm-block', { kind: 'block', run: () => relate('block', button) });
	},
	edit: (button) => void openDialog(`/_tuita/dialog/edit/${enc(button.dataset.postId)}`),
	history: (button) => void openDialog(`/_tuita/dialog/history/${enc(button.dataset.postId)}`),
	report: (button) => void openDialog(`/_tuita/dialog/report/${enc(button.dataset.postId)}`),
	'report-user': (button) => void openDialog(`/_tuita/dialog/report-user/${enc(button.dataset.acct)}`),
	'list-picker': (button) => void openDialog(`/_tuita/dialog/lists/${enc(button.dataset.acct)}`),
	'post-list': (button) =>
		void openDialog(`/_tuita/dialog/post-list/${enc(button.dataset.postId)}/${enc(button.dataset.kind)}`)
};

if (typeof document !== 'undefined' && !document.documentElement.dataset.postBound) {
	document.documentElement.dataset.postBound = '';

	document.addEventListener('tuita-posted', (event) => void posted(event.detail ?? {}));

	for (const [name, kind] of [['tuita-delete', 'delete-post'], ['tuita-block', 'block']]) {
		addEventListener(name, () => {
			const action = pending;
			pending = null;
			if (action && action.kind === kind) void action.run();
		});
	}

	document.addEventListener('click', (event) => {
		const target = event.target instanceof Element ? event.target : null;
		if (!target) return;

		const action = target.closest('[data-action]');
		if (action?.dataset.action === 'like' || action?.dataset.action === 'bookmark') {
			const bar = action.closest('[data-c="post-actions"]');
			if (bar) void toggle(bar, action.dataset.action);
			return;
		}
		if (action?.dataset.action === 'reply' && action.getAttribute('aria-disabled') === 'true') {
			toast(action.title);
			return;
		}

		const act = target.closest('[data-act]');
		if (act && !act.disabled && ACTS[act.dataset.act]) {
			ACTS[act.dataset.act](act);
			return;
		}

		const card = target.closest('article[data-href]');
		if (card) openCard(event, card);
	});

	document.addEventListener('input', (event) => {
		const form = event.target instanceof Element ? event.target.closest('form[data-post-form="edit"]') : null;
		if (form) syncEdit(form);
	});

	document.addEventListener('change', (event) => {
		const form = event.target instanceof Element ? event.target.closest('form[data-post-form="report"]') : null;
		if (form) document.querySelector('[data-act="report-send"]').disabled = !form.querySelector('[name="reason"]:checked');
		const member = event.target instanceof Element ? event.target.closest('[data-act="list-member"]') : null;
		if (member) void toggleMember(member);
	});

	document.addEventListener('submit', (event) => {
		const form = event.target instanceof Element ? event.target.closest('form[data-post-form]') : null;
		if (!form) return;
		event.preventDefault();
		void submitForm(form);
	});

	mount('[data-scroll-to]', (el) => el.scrollIntoView({ block: 'start' }));
}

async function toggleMember(box) {
	const on = box.checked;
	const path = `/lists/${enc(box.dataset.listId)}/members/${enc(box.dataset.acct)}`;
	try {
		await api(on ? 'POST' : 'DELETE', path);
	} catch (error) {
		box.checked = !on;
		toast(messageOf(error));
	}
}

/** A reply or quote was posted by the composer: show the reply at the top of its thread and refresh the parent's counts. */
async function posted({ id, replyTo, quote }) {
	if (replyTo) {
		const list = document.querySelector(`[data-replies="${CSS.escape(replyTo)}"]`);
		if (list && id) {
			try {
				const card = (await fragment(`/_tuita/post/${enc(id)}?hide_reply=1`)).querySelector('article');
				if (card) {
					const row = document.createElement('div');
					row.setAttribute('role', 'listitem');
					row.className = 'row';
					row.append(card);
					list.prepend(row);
					list.hidden = false;
				}
			} catch {
				// The reply shows after the next load.
			}
		}
		await refreshCards(replyTo);
	}
	if (quote) await refreshCards(quote);
}
