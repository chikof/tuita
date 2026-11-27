// The composer: editor state, drafts, the character counter, mention, hashtag and emote suggestions, the
// poll, schedule and emoji pickers, thread posts, publishing, and the compose dialog.
//
// Markup contract (rendered by `compose::composer`; `upload.js` owns the media tray and the dropzone inside it):
//
//   form[data-island="composer"]   data-user=<viewer id> (drafts are stored per account; without it nothing is
//                                  stored)  data-slot="new" | "r<post id>" | "q<post id>"  (the draft's name)
//                                  data-context="inline" | "dialog"  data-reply-to / data-reply-handle (a fixed
//                                  reply target)  data-placeholder (overrides the default)  data-autofocus
//   its parts, found by `data-role` and `data-act`: text, cw, suggest, note, quote, extras, chip, error, counter,
//   send, schedule, add-extra, policy-label; buttons `[data-act]`: images video poll cw sensitive add-extra
//   remove-extra chip-clear add-option remove-option remove-poll schedule-set schedule-clear; menu items
//   `[data-policy]`; `button[data-emoji]` in the picker; templates `[data-tpl="extra" | "spinner"]`.
//
// The compose dialog is `dialog#compose` holding a composer with `data-context="dialog"`. A click on
// `[data-compose]` anywhere opens it: `data-compose="new"`, `"reply"` or `"quote"` (the last two with
// `data-post-id`; the post is the `post` of `GET /api/v1/posts/<id>`). The element `form.composer` has the
// methods `setTarget({replyTo, quote})` (posts as the API gives them), `focus()` and `reset()`.
//
// Out: a bubbling `tuita-posted` CustomEvent on the form when a post (or the first of a thread) was created or a
// scheduled post was saved; `detail` and `ev.data` (JSON) are `{ id, replyTo, quote, scheduled, count }`
// (`id` is null for a scheduled post, `replyTo`/`quote` the ids it answers or quotes).
//
// Logout forgetting: a click on any `[data-forget-keys="<id> <id>"]` deletes `tuita:draft:<id>:*` for each id.
//
// Strings come from `#tuita-l10n` (composer_*, policy_*, schedule_*, char_counter_*, post_sent, view).

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

// ---- limits and post bodies -------------------------------------------------------------------

export const MAX_TEXT = 500;
export const MAX_CW = 100;
/** `POST /posts/thread` takes 2 to 25 posts, so a thread has at most 24 after the first. */
export const MAX_EXTRA = 24;
export const POLL_HOURS = [1, 6, 24, 72, 168];
export const MAX_POLL_OPTIONS = 4;
export const MAX_OPTION_LENGTH = 25;

export const newPoll = () => ({ options: ['', ''], hours: 24, multiple: false });

/** The request form of a draft: blanks dropped, answers trimmed. */
export const pollInput = (draft) => ({
	options: draft.options.map((o) => o.trim()).filter(Boolean),
	multiple: draft.multiple,
	durationHours: draft.hours
});

/** Two to four different answers, as the server wants. */
export function pollReady(draft) {
	const { options } = pollInput(draft);
	return (
		options.length >= 2 &&
		options.length <= MAX_POLL_OPTIONS &&
		new Set(options.map((o) => o.toLowerCase())).size === options.length
	);
}

/** The `POST /posts` body of a draft. */
export const postBody = (draft) => ({
	text: draft.text.trim(),
	cw: draft.cw.trim(),
	sensitive: draft.mediaIds.length > 0 && draft.sensitive,
	replyTo: draft.replyTo ?? null,
	quoteOf: draft.quoteOf ?? null,
	replyPolicy: draft.policy,
	mediaIds: draft.mediaIds,
	poll: draft.poll ? pollInput(draft.poll) : null
});

/** A thread continues the chain with plain text posts that keep the reply policy. */
export const extraBody = (text, policy) => ({
	text: text.trim(),
	cw: '',
	sensitive: false,
	replyPolicy: policy,
	mediaIds: []
});

/** Characters as the server counts them: scalar values, each URL as 23. */
export function postLength(text) {
	let urls = 0;
	const rest = text.replace(/https?:\/\/[^\s<>"]+[^\s<>".,:;'!?)\]]/g, () => {
		urls++;
		return '';
	});
	return [...rest].length + urls * 23;
}

// ---- scheduling ------------------------------------------------------------------------------

/** Server limits for `publishAt`: at least a minute and at most 90 days ahead. */
export const MIN_AHEAD_MS = 60_000;
export const MAX_AHEAD_MS = 90 * 86_400_000;

const pad = (n) => String(n).padStart(2, '0');

/** `2026-11-09T14:30` in the viewer's time zone: the value format of `<input type="datetime-local">`. */
export function toLocalInput(date) {
	return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}`;
}

/** The ISO instant of a `datetime-local` value, or `null` when it is empty or invalid. */
export function fromLocalInput(value) {
	const at = new Date(value);
	return Number.isNaN(at.getTime()) ? null : at.toISOString();
}

/** Whether `iso` is inside the window the server accepts, measured from `now`. */
export function inWindow(iso, now = Date.now()) {
	const ahead = Date.parse(iso) - now;
	return ahead >= MIN_AHEAD_MS && ahead <= MAX_AHEAD_MS;
}

/** `min` and `max` attributes for the picker, a little inside the server window. */
export function localBounds(now = Date.now()) {
	return {
		min: toLocalInput(new Date(now + MIN_AHEAD_MS + 60_000)),
		max: toLocalInput(new Date(now + MAX_AHEAD_MS - 60_000))
	};
}

/** Quick choices from `now`: in an hour, 20:00 today (when at least 2 h away), 09:00 tomorrow, in a week at 09:00. */
export function presets(now = new Date()) {
	const at = (days, hour) => {
		const d = new Date(now);
		d.setDate(d.getDate() + days);
		d.setHours(hour, 0, 0, 0);
		return d;
	};
	const list = [{ id: 'hour', at: new Date(now.getTime() + 3_600_000) }];
	const evening = at(0, 20);
	if (evening.getTime() - now.getTime() >= 2 * 3_600_000) list.push({ id: 'evening', at: evening });
	list.push({ id: 'tomorrow', at: at(1, 9) }, { id: 'week', at: at(7, 9) });
	return list;
}

// ---- emotes ------------------------------------------------------------------------------------

/** Most rows the composer's `:` suggestion list shows. */
export const MAX_SUGGESTIONS = 8;

/** Typed letters needed before `:` starts suggesting. */
export const MIN_QUERY = 2;

/**
 * The shortcode characters typed after a `:` that ends `before` (the text up to the caret), or `null` when the
 * caret is not in one. The colon must open the text or follow a character that is neither a word character nor
 * `/`, so `12:30`, `https://x` and `a:b` never match.
 */
export function emoteQuery(before) {
	const hit = /(?:^|[^\p{L}\p{N}_/]):([A-Za-z0-9_]{0,32})$/u.exec(before);
	return hit ? hit[1] : null;
}

/** Emotes whose shortcode starts with `query`, then those that contain it; at most `limit`. */
export function rankEmotes(list, query, limit = MAX_SUGGESTIONS) {
	const needle = query.toLowerCase();
	if (needle.length < MIN_QUERY) return [];
	const prefix = [];
	const inside = [];
	for (const emote of list) {
		const at = emote.shortcode.toLowerCase().indexOf(needle);
		if (at === 0) prefix.push(emote);
		else if (at > 0) inside.push(emote);
	}
	return [...prefix, ...inside].slice(0, limit);
}

/** The text a picked emote puts in the box: a space follows unless one already does. */
export function emoteInsertion(emoji, next) {
	const isEmote = /^:[A-Za-z0-9_]{2,32}:$/.test(emoji);
	return isEmote && (next === undefined || !/\s/.test(next)) ? `${emoji} ` : emoji;
}

/** The picker's sections: uncategorised first, then each category in the order the list gives. */
export function groupEmotes(list) {
	const groups = new Map([[null, []]]);
	for (const emote of list) {
		const key = emote.category || null;
		const items = groups.get(key);
		if (items) items.push(emote);
		else groups.set(key, [emote]);
	}
	return [...groups].filter(([, items]) => items.length > 0).map(([category, items]) => ({ category, items }));
}

// ---- drafts ----------------------------------------------------------------------------

const DRAFT = 'tuita:draft:';

/** `tuita:draft:<id>:new`, `…:r<post>` or `…:q<post>`. */
export const draftKey = (userId, slot) => `${DRAFT}${userId}:${slot}`;

const keysOf = (storage) =>
	Array.from({ length: storage.length }, (_, i) => storage.key(i)).filter((key) => key !== null);

/** Removes every draft stored for `userId`. */
export function purgeDrafts(storage, userId) {
	for (const key of keysOf(storage)) if (key.startsWith(`${DRAFT}${userId}:`)) storage.removeItem(key);
}

/** Removes the drafts older versions wrote without an account id: `tuita:draft:<slot>`. */
export function purgeUnscopedDrafts(storage) {
	for (const key of keysOf(storage)) {
		if (key.startsWith(DRAFT) && !key.slice(DRAFT.length).includes(':')) storage.removeItem(key);
	}
}

/** Runs `action` and ignores a storage that is full, blocked or missing. */
export function quietly(action) {
	try {
		action();
	} catch {
		// Drafts are a convenience; the app works without storage.
	}
}

// ---- strings and small helpers -----------------------------------------------------------------------------

let cachedStrings;
const strings = () => (cachedStrings ??= JSON.parse(document.getElementById('tuita-l10n')?.textContent ?? '{}'));

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

/** `@ada`, or `@ada@remote.example` for remote accounts. */
export const handleOf = (who) => (who.domain ? `@${who.handle}@${who.domain}` : `@${who.handle}`);

const fullTime = (iso) =>
	new Date(iso).toLocaleString(document.documentElement.lang, {
		hour: 'numeric',
		minute: '2-digit',
		month: 'short',
		day: 'numeric',
		year: 'numeric'
	});

const say = (input) => document.dispatchEvent(new CustomEvent('tuita-toast', { detail: input }));

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
		if (res.status === 401) location.reload();
		let message = res.statusText || `HTTP ${res.status}`;
		try {
			const failure = await res.json();
			if (typeof failure.error === 'string' && failure.error) message = failure.error;
		} catch {
			// Non-JSON error bodies fall back to the status line.
		}
		throw Object.assign(new Error(message), { status: res.status });
	}
	if (res.status === 204 || res.headers.get('content-length') === '0') return undefined;
	return res.json();
}

// ---- the instance's emotes, loaded once ----------------------------------------------------------------------

const emojiState = { list: [], loaded: false };
let inflight = null;

/** One `GET /emojis`; a failure keeps what is shown and may be retried on the next use. */
export function loadEmojis() {
	if (emojiState.loaded) return Promise.resolve();
	inflight ??= api('GET', '/emojis')
		.then((list) => {
			if (Array.isArray(list)) {
				emojiState.list = list;
				emojiState.loaded = true;
			}
		})
		.catch(() => {})
		.finally(() => {
			inflight = null;
		});
	return inflight;
}

// ---- one composer --------------------------------------------------------------------------------------------

const DRAFT_MS = 300;
const VIDEO_TYPES = ['video/mp4', 'video/webm', 'video/quicktime'];
const ATTACHABLE = ['image/png', 'image/jpeg', 'image/gif', 'image/webp', ...VIDEO_TYPES];
const forms = new Set();

function mountComposer(form) {
	const table = strings();
	const lang = document.documentElement.lang;
	const q = (selector) => form.querySelector(selector);
	const role = (name) => q(`[data-role="${name}"]`);
	const act = (name) => q(`[data-act="${name}"]`);

	const area = role('text');
	const cwInput = role('cw');
	const suggestList = role('suggest');
	const note = role('note');
	const quoteBox = role('quote');
	const extrasBox = role('extras');
	const chip = role('chip');
	const errorBox = role('error');
	const counterBox = role('counter');
	const sendButton = role('send');
	const scheduleBox = role('schedule');
	const addExtraBox = role('add-extra');
	const policyLabel = role('policy-label');
	const trayEl = q('[data-island="media-tray"]');
	const pollBox = q('[data-role="poll"]');
	const zone = form.closest('[data-island="dropzone"]');
	const dialog = form.dataset.context === 'dialog' ? form.closest('dialog') : null;
	const imagesInput = q('input[data-role="images"]');
	const videoInput = q('input[data-role="video"]');
	const emojiPanel = q('[data-c="emoji-picker"]')?.closest('[popover]');
	const schedulePanel = q('[data-role="schedule-panel"]')?.closest('[popover]');
	const customInput = q('[data-field="custom"]');

	const state = {
		cwOpen: false,
		sensitive: false,
		policy: 'everyone',
		busy: false,
		error: '',
		pollOpen: false,
		extras: [],
		scheduleAt: null,
		suggestions: [],
		active: 0,
		token: '',
		media: { count: 0, images: 0, video: false, busy: false, failed: false },
		replyTo: form.dataset.replyTo ? { id: form.dataset.replyTo, handle: form.dataset.replyHandle ?? '' } : null,
		quote: null,
		slot: form.dataset.slot || 'new',
		failure: ''
	};
	let draftTimer;
	let acTimer;
	let blurTimer;

	// -- drafts
	const draftName = () => (form.dataset.user ? draftKey(form.dataset.user, state.slot) : null);
	const loadDraft = () => {
		let value = '';
		quietly(() => {
			const key = draftName();
			value = key ? (localStorage.getItem(key) ?? '') : '';
		});
		return value;
	};
	const saveDraft = () => {
		clearTimeout(draftTimer);
		quietly(() => {
			const key = draftName();
			if (!key) return;
			if (area.value.trim()) localStorage.setItem(key, area.value);
			else localStorage.removeItem(key);
		});
	};
	const scheduleDraft = () => {
		clearTimeout(draftTimer);
		draftTimer = setTimeout(saveDraft, DRAFT_MS);
	};

	// -- derived state
	const length = () => postLength(area.value.trim());
	const pollDraft = () => {
		const options = [...pollBox.querySelectorAll('.option:not([hidden]) [data-field="option"]')].map((i) => i.value);
		const tab = pollBox.querySelector('[role="tab"][aria-selected="true"]');
		const hours = Number(/-seg-(\d+)$/.exec(tab?.id ?? '')?.[1] ?? 24);
		const multiple = pollBox.querySelector('[data-role="multiple"]')?.getAttribute('aria-checked') === 'true';
		return { options, hours, multiple };
	};
	const canSend = () => {
		const trimmed = area.value.trim();
		const hasMedia = state.media.count > 0;
		const empty = trimmed.length === 0 && !hasMedia;
		const pollOk = !state.pollOpen || (trimmed.length > 0 && pollReady(pollDraft()));
		const extrasOk = state.extras.every((t) => postLength(t.trim()) <= MAX_TEXT);
		return (
			!state.busy &&
			!empty &&
			length() <= MAX_TEXT &&
			cwInput.value.length <= MAX_CW &&
			pollOk &&
			extrasOk &&
			!state.media.busy &&
			!state.media.failed
		);
	};
	const sendLabel = () =>
		state.scheduleAt
			? table.composer_schedule
			: state.replyTo
				? table.composer_reply
				: state.extras.length
					? table.composer_post_all
					: table.composer_post;

	// -- painting
	const toggle = (node, shown) => {
		if (node) node.hidden = !shown;
	};
	const press = (name, on) => act(name)?.setAttribute('aria-pressed', on ? 'true' : 'false');
	const disable = (name, off) => {
		const button = act(name);
		if (button) button.disabled = off;
	};

	function paintCounter() {
		const count = length();
		toggle(counterBox, count > 0);
		const counter = counterBox.querySelector('.counter');
		if (!counter) return;
		const remaining = MAX_TEXT - count;
		const over = remaining < 0;
		counter.classList.toggle('over', over);
		counter.setAttribute('aria-label', text(over ? table.char_counter_over : table.char_counter_left, { count: Math.abs(remaining) }, lang));
		const rects = [...counter.querySelectorAll('rect')];
		const filled = Math.min(rects.length, Math.ceil((Math.max(count, 0) / MAX_TEXT) * rects.length));
		rects.forEach((rect, i) => rect.classList.toggle('on', i < filled));
		let number = counter.querySelector('.number');
		if (remaining <= 20) {
			if (!number) {
				number = document.createElement('span');
				number.className = 'number';
				number.setAttribute('aria-hidden', 'true');
				counter.append(number);
			}
			number.textContent = String(remaining);
		} else {
			number?.remove();
		}
	}

	function paintSend() {
		const label = sendButton.querySelector('.label');
		if (label) label.textContent = sendLabel();
		sendButton.disabled = state.busy || !canSend();
		if (state.busy) {
			sendButton.setAttribute('aria-busy', 'true');
			label?.classList.add('hidden');
			if (!sendButton.querySelector('.spinner')) {
				const spinner = document.createElement('span');
				spinner.className = 'spinner';
				spinner.append(form.querySelector('template[data-tpl="spinner"]').content.cloneNode(true));
				sendButton.append(spinner);
			}
		} else {
			sendButton.removeAttribute('aria-busy');
			label?.classList.remove('hidden');
			sendButton.querySelector('.spinner')?.remove();
		}
	}

	function sync() {
		const hasMedia = state.media.count > 0;
		paintCounter();
		paintSend();
		disable('images', state.pollOpen || state.media.video || state.media.images >= 4);
		disable('video', state.pollOpen || hasMedia);
		disable('poll', hasMedia);
		press('poll', state.pollOpen);
		press('cw', state.cwOpen);
		press('sensitive', state.sensitive);
		toggle(act('sensitive'), hasMedia);
		toggle(scheduleBox, !state.replyTo && state.extras.length === 0);
		toggle(addExtraBox, !state.replyTo && !state.scheduleAt && state.extras.length < MAX_EXTRA);
		toggle(pollBox, state.pollOpen);
		toggle(cwInput, state.cwOpen);
		toggle(chip, Boolean(state.scheduleAt));
		if (state.scheduleAt) role('chip-text').textContent = text(table.composer_schedule_chip, { time: fullTime(state.scheduleAt) }, lang);
		toggle(errorBox, Boolean(state.error));
		errorBox.textContent = state.error;
		if (trayEl) trayEl.dataset.poll = String(state.pollOpen);
		if (zone) {
			if (state.pollOpen) zone.dataset.disabled = 'true';
			else zone.removeAttribute('data-disabled');
		}
		const open = state.suggestions.length > 0;
		area.setAttribute('aria-expanded', String(open));
		if (open) area.setAttribute('aria-activedescendant', `${suggestList.id}-s${state.active}`);
		else area.removeAttribute('aria-activedescendant');
		policyLabel.textContent = table[`policy_${state.policy}`] ?? policyLabel.textContent;
	}

	function applyContext() {
		const { replyTo, quote } = state;
		note.textContent = replyTo ? text(table.composer_replying_to, { handle: replyTo.handle }, lang) : '';
		toggle(note, Boolean(replyTo));
		area.placeholder =
			form.dataset.placeholder ||
			(replyTo ? table.composer_reply_placeholder : table.composer_placeholder);
		if (dialog) {
			const title = dialog.querySelector('.surface header h2');
			if (title) title.textContent = replyTo ? table.composer_reply_title : quote ? table.composer_quote_title : table.composer_title;
		}
		quoteBox.replaceChildren();
		if (quote) {
			const card = document.createElement('div');
			card.className = 'quote-card';
			const who = document.createElement('p');
			who.className = 'who';
			const name = document.createElement('strong');
			name.textContent = quote.author.displayName;
			const handle = document.createElement('span');
			handle.textContent = handleOf(quote.author);
			who.append(name, ' ', handle);
			const body = document.createElement('p');
			body.className = 'text';
			body.textContent = quote.cw || quote.text;
			card.append(who, body);
			quoteBox.append(card);
		}
		toggle(quoteBox, Boolean(quote));
	}

	/**
	 * Size the box to its content. `scrollHeight` leaves out the border under a border-box, which clipped the
	 * last line's descenders, and Chrome ignores a wrapped placeholder, so an empty box is measured with the
	 * placeholder in it. A box that is not laid out (a closed dialog) is left alone.
	 */
	function fit() {
		if (!area.isConnected || area.clientWidth === 0) return;
		const empty = area.value === '';
		if (empty) area.value = area.placeholder;
		area.style.height = 'auto';
		const full = area.scrollHeight + area.offsetHeight - area.clientHeight;
		area.style.height = `${full}px`;
		area.style.overflowY = area.offsetHeight + 1 < full ? 'auto' : 'hidden';
		if (empty) area.value = '';
	}

	// -- suggestions
	function closeSuggestions() {
		state.suggestions = [];
		state.token = '';
		suggestList.replaceChildren();
		suggestList.hidden = true;
		sync();
	}

	function showSuggestions(list, token) {
		state.suggestions = list;
		state.token = token;
		state.active = 0;
		suggestList.replaceChildren();
		list.forEach((choice, i) => {
			const item = document.createElement('li');
			item.id = `${suggestList.id}-s${i}`;
			item.setAttribute('role', 'option');
			item.setAttribute('aria-selected', String(i === 0));
			item.classList.toggle('active', i === 0);
			if (choice.kind === 'user') {
				const main = document.createElement('span');
				main.className = 'main-line';
				main.textContent = choice.user.displayName;
				const sub = document.createElement('span');
				sub.className = 'sub';
				sub.textContent = handleOf(choice.user);
				item.append(main, sub);
			} else if (choice.kind === 'emote') {
				item.classList.add('emote');
				const img = document.createElement('img');
				img.src = choice.emoji.url;
				img.alt = '';
				img.height = 24;
				img.loading = 'lazy';
				img.decoding = 'async';
				const main = document.createElement('span');
				main.className = 'main-line';
				main.textContent = `:${choice.emoji.shortcode}:`;
				item.append(img, main);
			} else {
				const main = document.createElement('span');
				main.className = 'main-line';
				main.textContent = `#${choice.tag.tag}`;
				const sub = document.createElement('span');
				sub.className = 'sub';
				const posts = Number(choice.tag.posts);
				sub.textContent = text(table.composer_tag_posts, { count: posts, compact: new Intl.NumberFormat(lang, { notation: 'compact', maximumFractionDigits: 1 }).format(posts) }, lang);
				item.append(main, sub);
			}
			item.addEventListener('pointerdown', (event) => {
				event.preventDefault();
				pick(i);
			});
			suggestList.append(item);
		});
		suggestList.hidden = list.length === 0;
		sync();
	}

	function markActive() {
		[...suggestList.children].forEach((item, i) => {
			item.setAttribute('aria-selected', String(i === state.active));
			item.classList.toggle('active', i === state.active);
		});
		sync();
	}

	/** Suggests emotes for a `:query` at the caret, loading the list on first use. */
	async function suggestEmotes(query) {
		const found = `:${query}`;
		await loadEmojis();
		if (!area.value.slice(0, area.selectionStart).endsWith(found)) return;
		const list = rankEmotes(emojiState.list, query);
		if (!list.length) return closeSuggestions();
		showSuggestions(list.map((emoji) => ({ kind: 'emote', id: emoji.shortcode, emoji })), found);
	}

	function detect() {
		const before = area.value.slice(0, area.selectionStart);
		const hit = /(?:^|[^\p{L}\p{N}_@#&])([@#])([\p{L}\p{N}_]{1,30})$/u.exec(before);
		clearTimeout(acTimer);
		if (!hit) {
			const query = emoteQuery(before);
			if (query === null) return closeSuggestions();
			if (query.length < MIN_QUERY) {
				void loadEmojis();
				return closeSuggestions();
			}
			return void suggestEmotes(query);
		}
		const [, sigil, query] = hit;
		const found = `${sigil}${query}`;
		acTimer = setTimeout(async () => {
			try {
				let list;
				if (sigil === '@') {
					const { users } = await api('GET', `/search?${new URLSearchParams({ q: query, type: 'people' })}`);
					const me = form.dataset.user;
					list = users.filter((u) => u.id !== me).slice(0, 6).map((user) => ({ kind: 'user', id: user.id, user }));
				} else {
					const tags = await api('GET', `/hashtags?${new URLSearchParams({ prefix: query })}`);
					list = tags.map((tag) => ({ kind: 'tag', id: tag.tag, tag }));
				}
				if (!area.value.slice(0, area.selectionStart).endsWith(found)) return;
				showSuggestions(list, found);
			} catch {
				closeSuggestions();
			}
		}, 140);
	}

	function pick(index) {
		const choice = state.suggestions[index];
		if (!choice) return;
		const insert =
			choice.kind === 'user' ? `${handleOf(choice.user)} ` : choice.kind === 'emote' ? `:${choice.emoji.shortcode}: ` : `#${choice.tag.tag} `;
		const end = area.selectionStart;
		const start = end - state.token.length;
		area.value = area.value.slice(0, start) + insert + area.value.slice(end);
		closeSuggestions();
		changed();
		area.focus();
		area.setSelectionRange(start + insert.length, start + insert.length);
	}

	// -- text, cw, extras
	function changed() {
		scheduleDraft();
		fit();
		sync();
	}

	function insertText(emoji) {
		const start = area.selectionStart ?? area.value.length;
		const end = area.selectionEnd ?? start;
		const piece = emoteInsertion(emoji, area.value[end]);
		area.value = area.value.slice(0, start) + piece + area.value.slice(end);
		changed();
		area.focus();
		area.setSelectionRange(start + piece.length, start + piece.length);
	}

	function paintExtras() {
		extrasBox.replaceChildren();
		const template = form.querySelector('template[data-tpl="extra"]');
		state.extras.forEach((value, i) => {
			const row = template.content.firstElementChild.cloneNode(true);
			const box = row.querySelector('textarea');
			box.value = value;
			box.setAttribute('aria-label', text(table.composer_extra_label, { n: i + 2 }, lang));
			box.addEventListener('input', () => {
				state.extras[i] = box.value;
				sync();
			});
			row.querySelector('[data-act="remove-extra"]').setAttribute('aria-label', text(table.composer_extra_remove, { n: i + 2 }, lang));
			extrasBox.append(row);
		});
	}

	function addExtra() {
		state.extras.push('');
		paintExtras();
		sync();
		extrasBox.querySelector('.extra:last-child textarea')?.focus();
	}

	// -- poll
	function resetPoll() {
		state.pollOpen = false;
		for (const input of pollBox.querySelectorAll('[data-field="option"]')) input.value = '';
		pollBox.querySelectorAll('.option').forEach((option, i) => (option.hidden = i >= 2));
		toggle(pollBox.querySelector('[data-role="add-option"]'), true);
		const day = pollBox.querySelector('[role="tab"][id$="-seg-24"]');
		if (day && day.getAttribute('aria-selected') !== 'true') day.click();
		const multiple = pollBox.querySelector('[data-role="multiple"]');
		if (multiple?.getAttribute('aria-checked') === 'true') multiple.click();
	}

	function togglePoll() {
		if (state.pollOpen) return resetPollAndSync();
		if (state.media.count > 0) return say(table.composer_err_exclusive);
		state.pollOpen = true;
		sync();
	}

	function resetPollAndSync() {
		resetPoll();
		sync();
	}

	function addOption() {
		const next = pollBox.querySelector('.option[hidden]');
		if (next) next.hidden = false;
		toggle(pollBox.querySelector('[data-role="add-option"]'), Boolean(pollBox.querySelector('.option[hidden]')));
		sync();
	}

	function removeOption(index) {
		const options = [...pollBox.querySelectorAll('.option')];
		const visible = options.filter((o) => !o.hidden);
		const inputs = options.map((o) => o.querySelector('[data-field="option"]'));
		for (let i = index; i < visible.length - 1; i++) inputs[i].value = inputs[i + 1].value;
		const last = visible.length - 1;
		inputs[last].value = '';
		options[last].hidden = true;
		toggle(pollBox.querySelector('[data-role="add-option"]'), true);
		sync();
	}

	// -- attachments
	function attach(files) {
		const sorted = [...files].sort((a, b) => Number(isVideo(b)) - Number(isVideo(a)));
		trayEl?.attach?.(sorted);
		if (state.media.count === 0) state.sensitive = false;
		sync();
	}
	const isVideo = (file) => VIDEO_TYPES.includes(file.type);

	// -- scheduling
	function setSchedule(iso) {
		if (iso !== null && !inWindow(iso)) {
			state.failure = table.schedule_window;
			paintSchedulePanel();
			return;
		}
		state.failure = '';
		state.scheduleAt = iso;
		schedulePanel?.hidePopover();
		sync();
	}

	function paintSchedulePanel() {
		if (!schedulePanel) return;
		const list = role('presets');
		list.replaceChildren();
		const template = schedulePanel.querySelector('template[data-tpl="preset"]');
		for (const preset of presets()) {
			const button = template.content.firstElementChild.cloneNode(true);
			button.querySelector('[data-role="name"]').textContent = table[`schedule_${preset.id}`];
			button.querySelector('[data-role="when"]').textContent = fullTime(preset.at.toISOString());
			button.addEventListener('click', () => setSchedule(preset.at.toISOString()));
			list.append(button);
		}
		const bounds = localBounds();
		customInput.min = bounds.min;
		customInput.max = bounds.max;
		const hint = customInput.closest('[data-c="text-field"]')?.querySelector('.message');
		if (hint) hint.textContent = Intl.DateTimeFormat().resolvedOptions().timeZone;
		const failure = schedulePanel.querySelector('[data-role="failure"]');
		failure.textContent = state.failure;
		failure.hidden = !state.failure;
		schedulePanel.querySelector('[data-role="clear"]').hidden = !state.scheduleAt;
		schedulePanel.querySelector('[data-act="schedule-set"]').disabled = !customInput.value;
	}

	// -- publishing
	/** Empties the box and what hangs on it, keeping the stored draft. */
	function clearState() {
		clearTimeout(draftTimer);
		area.value = '';
		cwInput.value = '';
		state.cwOpen = false;
		state.sensitive = false;
		state.extras = [];
		state.scheduleAt = null;
		state.error = '';
		state.failure = '';
		paintExtras();
		resetPoll();
		trayEl?.tray?.clear();
		closeSuggestions();
		fit();
		sync();
	}

	/** After a post was sent: the box starts over and its draft is gone. */
	function reset() {
		clearTimeout(draftTimer);
		quietly(() => {
			const key = draftName();
			if (key) localStorage.removeItem(key);
		});
		clearState();
	}

	function announce(detail) {
		const event = new CustomEvent('tuita-posted', { bubbles: true, detail });
		event.data = JSON.stringify(detail);
		form.dispatchEvent(event);
	}

	async function submit(event) {
		event?.preventDefault();
		if (!canSend()) return;
		state.busy = true;
		state.error = '';
		sync();
		try {
			const mediaIds = trayEl?.tray ? await trayEl.tray.finalize() : [];
			const first = postBody({
				text: area.value,
				cw: state.cwOpen ? cwInput.value : '',
				sensitive: state.sensitive,
				policy: state.policy,
				mediaIds,
				poll: state.pollOpen ? pollDraft() : null,
				replyTo: state.replyTo?.id,
				quoteOf: state.quote?.id
			});
			const target = { replyTo: state.replyTo?.id ?? null, quote: state.quote?.id ?? null };
			if (state.scheduleAt) {
				const item = await api('POST', '/scheduled', { ...first, publishAt: state.scheduleAt });
				reset();
				say({ message: text(table.composer_scheduled, { time: fullTime(item.publishAt) }, lang), label: table.view, href: '/scheduled' });
				announce({ id: null, ...target, scheduled: true, count: 1 });
				dialog?.close();
				return;
			}
			const thread = state.extras.map((t) => t.trim()).filter(Boolean);
			let created;
			let count = 1;
			if (thread.length) {
				const { posts } = await api('POST', '/posts/thread', { posts: [first, ...thread.map((t) => extraBody(t, state.policy))] });
				created = posts[0];
				count = posts.length;
			} else {
				created = await api('POST', '/posts', first);
			}
			reset();
			say({
				message: count > 1 ? text(table.composer_thread_sent, { count }, lang) : table.post_sent,
				label: table.view,
				href: `/${handleOf(created.author)}/${created.id}`
			});
			announce({ id: created.id, ...target, scheduled: false, count });
			dialog?.close();
		} catch (failure) {
			state.error = failure instanceof Error ? failure.message : String(failure);
		} finally {
			state.busy = false;
			sync();
		}
	}

	// -- events
	function keydown(event) {
		if (state.suggestions.length) {
			const key = event.key;
			if (key === 'ArrowDown' || key === 'ArrowUp') {
				event.preventDefault();
				state.active = (state.active + (key === 'ArrowDown' ? 1 : -1) + state.suggestions.length) % state.suggestions.length;
				return markActive();
			}
			if (key === 'Enter' || key === 'Tab') {
				event.preventDefault();
				return pick(state.active);
			}
			if (key === 'Escape') {
				event.preventDefault();
				event.stopPropagation();
				return closeSuggestions();
			}
		}
		if (event.key === 'Enter' && (event.ctrlKey || event.metaKey) && canSend()) {
			event.preventDefault();
			void submit();
		}
	}

	function onclick(event) {
		const target = event.target instanceof Element ? event.target.closest('[data-act], [data-policy], [data-emoji]') : null;
		if (!target || !form.contains(target)) return;
		if (target.dataset.emoji) {
			target.closest('[popover]')?.hidePopover();
			return insertText(target.dataset.emoji);
		}
		if (target.dataset.policy) {
			state.policy = target.dataset.policy;
			return sync();
		}
		switch (target.dataset.act) {
			case 'images':
				return imagesInput.click();
			case 'video':
				return videoInput.click();
			case 'poll':
				return togglePoll();
			case 'cw':
				state.cwOpen = !state.cwOpen;
				sync();
				if (state.cwOpen) cwInput.focus();
				return;
			case 'sensitive':
				state.sensitive = !state.sensitive;
				return sync();
			case 'add-extra':
				return addExtra();
			case 'remove-extra': {
				const index = [...extrasBox.children].indexOf(target.closest('.extra'));
				state.extras.splice(index, 1);
				paintExtras();
				return sync();
			}
			case 'chip-clear':
				state.scheduleAt = null;
				return sync();
			case 'add-option':
				return addOption();
			case 'remove-option':
				return removeOption(Number(target.dataset.option));
			case 'remove-poll':
				return resetPollAndSync();
			case 'schedule-set':
				return setSchedule(fromLocalInput(customInput.value));
			case 'schedule-clear':
				return setSchedule(null);
		}
	}

	const onfiles = (event) => attach(event.detail.files);
	const onfile = (event) => {
		attach([...(event.currentTarget.files ?? [])]);
		event.currentTarget.value = '';
	};
	const onpaste = (event) => {
		const files = [...(event.clipboardData?.files ?? [])].filter((f) => ATTACHABLE.includes(f.type));
		if (!files.length) return;
		event.preventDefault();
		attach(files);
	};
	const onblur = () => {
		clearTimeout(blurTimer);
		blurTimer = setTimeout(closeSuggestions, 150);
	};
	const ontray = (event) => {
		state.media = event.detail;
		if (state.media.count === 0) state.sensitive = false;
		sync();
	};
	const inPoll = (event) => event.target instanceof Element && event.target.closest('[data-role="poll"]');
	const onpoll = (event) => {
		if (inPoll(event)) sync();
	};
	// The length picker and the switch change their own attributes in a click handler that may run after this one.
	const onpollclick = (event) => {
		if (inPoll(event)) setTimeout(sync, 0);
	};
	const ontoggleEmoji = (event) => {
		if (event.newState !== 'open') return;
		void loadEmojis().then(fillCustom);
	};
	const ontoggleSchedule = (event) => {
		if (event.newState === 'open') paintSchedulePanel();
	};
	const oncustomfocus = () => {
		if (!customInput.value) customInput.value = toLocalInput(new Date(Date.now() + 3_600_000));
		schedulePanel.querySelector('[data-act="schedule-set"]').disabled = false;
	};
	const oncustominput = () => {
		schedulePanel.querySelector('[data-act="schedule-set"]').disabled = !customInput.value;
	};
	const oncustomkey = (event) => {
		if (event.key !== 'Enter') return;
		event.preventDefault();
		setSchedule(fromLocalInput(customInput.value));
	};

	/** Fills the picker's second tab with the instance's emotes, by category, and shows the tabs. */
	function fillCustom() {
		const groups = groupEmotes(emojiState.list);
		const panel = q('[data-c="emoji-picker"] [data-role="custom"]');
		if (!panel) return;
		panel.replaceChildren();
		for (const group of groups) {
			const section = document.createElement('section');
			if (group.category) {
				const heading = document.createElement('h3');
				heading.textContent = group.category;
				section.append(heading);
			}
			const grid = document.createElement('div');
			grid.className = 'grid';
			for (const emote of group.items) {
				const button = document.createElement('button');
				button.type = 'button';
				button.setAttribute('aria-label', `:${emote.shortcode}:`);
				button.title = `:${emote.shortcode}:`;
				button.dataset.emoji = `:${emote.shortcode}:`;
				const img = document.createElement('img');
				img.src = emote.url;
				img.alt = '';
				img.height = 28;
				img.loading = 'lazy';
				img.decoding = 'async';
				button.append(img);
				grid.append(button);
			}
			section.append(grid);
			panel.append(section);
		}
		toggle(q('[data-c="emoji-picker"] [data-role="tabs"]'), groups.length > 0);
	}

	// -- targets (the dialog's box changes what it answers every time it opens)
	function setTarget({ replyTo = null, quote = null } = {}) {
		saveDraft();
		clearState();
		state.replyTo = replyTo ? { id: replyTo.id, handle: handleOf(replyTo.author) } : null;
		state.quote = quote;
		state.slot = replyTo ? `r${replyTo.id}` : quote ? `q${quote.id}` : 'new';
		applyContext();
		area.value = loadDraft();
		fit();
		sync();
	}

	const watch = new ResizeObserver(() => {
		if (area.clientWidth !== width) {
			width = area.clientWidth;
			fit();
		}
	});
	let width = area.clientWidth;
	watch.observe(area);
	void document.fonts?.ready.then(fit);

	form.composer = {
		setTarget,
		reset,
		focus: () => {
			fit();
			area.focus();
		}
	};
	forms.add(form);

	form.addEventListener('submit', submit);
	form.addEventListener('click', onclick);
	form.addEventListener('tuita-tray', ontray);
	form.addEventListener('input', onpoll);
	form.addEventListener('click', onpollclick);
	zone?.addEventListener('tuita-files', onfiles);
	imagesInput.addEventListener('change', onfile);
	videoInput.addEventListener('change', onfile);
	area.addEventListener('input', () => {
		changed();
		detect();
	});
	area.addEventListener('click', detect);
	area.addEventListener('keydown', keydown);
	area.addEventListener('paste', onpaste);
	area.addEventListener('blur', onblur);
	cwInput.addEventListener('input', sync);
	emojiPanel?.addEventListener('toggle', ontoggleEmoji);
	schedulePanel?.addEventListener('toggle', ontoggleSchedule);
	customInput?.addEventListener('focus', oncustomfocus);
	customInput?.addEventListener('input', oncustominput);
	customInput?.addEventListener('keydown', oncustomkey);
	window.addEventListener('pagehide', saveDraft);
	dialog?.addEventListener('close', () => {
		saveDraft();
		resetPoll();
		trayEl?.tray?.clear();
		state.extras = [];
		paintExtras();
		state.scheduleAt = null;
		state.error = '';
		sync();
	});

	area.value = loadDraft();
	applyContext();
	fit();
	sync();
	if (form.dataset.autofocus !== undefined && !dialog) area.focus();

	return () => {
		forms.delete(form);
		watch.disconnect();
		window.removeEventListener('pagehide', saveDraft);
		clearTimeout(draftTimer);
		clearTimeout(acTimer);
		clearTimeout(blurTimer);
		delete form.composer;
	};
}

mount('[data-island="composer"]', mountComposer);

// ---- opening the dialog, forgetting drafts ---------------------------------------------------------------------

async function openCompose(trigger) {
	const dialog = document.getElementById('compose');
	const form = dialog?.querySelector('[data-island="composer"]');
	if (!dialog || !form?.composer) return;
	const kind = trigger.dataset.compose || 'new';
	let post = null;
	if (kind !== 'new' && trigger.dataset.postId) {
		try {
			post = (await api('GET', `/posts/${encodeURIComponent(trigger.dataset.postId)}`)).post;
		} catch {
			return say(strings().composer_open_failed);
		}
	}
	form.composer.setTarget({ replyTo: kind === 'reply' ? post : null, quote: kind === 'quote' ? post : null });
	if (!dialog.open) dialog.showModal();
	form.composer.focus();
}

if (typeof document !== 'undefined' && !document.documentElement.dataset.composerBound) {
	document.documentElement.dataset.composerBound = '';
	quietly(() => purgeUnscopedDrafts(localStorage));
	document.addEventListener('click', (event) => {
		const trigger = event.target instanceof Element ? event.target.closest('[data-compose]') : null;
		if (!trigger) return;
		event.preventDefault();
		void openCompose(trigger);
	});
	document.addEventListener(
		'click',
		(event) => {
			const control = event.target instanceof Element ? event.target.closest('[data-forget-keys]') : null;
			if (!control) return;
			const ids = (control.dataset.forgetKeys ?? '').split(/\s+/).filter(Boolean);
			quietly(() => ids.forEach((id) => purgeDrafts(localStorage, id)));
			for (const form of forms) form.composer?.reset?.();
		},
		true
	);
}
