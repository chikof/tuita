// Follow buttons and the profile's relation menu. The server draws every button (profile header,
// people rows, hover card, right rail) as `<button data-follow data-acct data-handle data-remote
// data-state="none|pending|following">` with a `.label` span; markup inserted by islands is never
// hydrated, so this module drives them all.
//
// A click flips every button of the same account at once, calls `POST|DELETE /users/<acct>/follow`
// and settles on the answer's relation (rolling back with a toast on failure). `[data-profile-act]`
// (`mute`, `unmute`, `block`, `unblock`) are the profile menu's entries: they call the API, store
// the toast for after the reload and reload the page, which the server redraws from the new
// relation. Blocking asks first in `#confirm-profile-block`, which dispatches `profile-block`.
//
// Client strings used: follow, following, follow_requested, follow_cancel_request,
// follow_requested_toast, follow_cancelled_toast, muted_toast, unmuted_toast, blocked_toast,
// unblocked_toast, profile_followers (plural).

/** What a follow button shows: no follow, a request the remote account has not answered, or a follow. */
export const followView = (relation) => (relation.following ? 'following' : relation.followPending ? 'pending' : 'none');

/**
 * Where a click takes the button before the server answers. Following a remote account is a
 * request until its instance sends an Accept; a click on a pending request or a follow undoes it.
 */
export const followStep = (view, remote) => (view !== 'none' ? 'none' : remote ? 'pending' : 'following');

const FLASH = 'tuita:flash-toast';

const strings = () => {
	try {
		return JSON.parse(document.getElementById('tuita-l10n')?.textContent ?? '{}');
	} catch {
		return {};
	}
};

const lang = () => document.documentElement.lang || 'en';
const enc = encodeURIComponent;
const toast = (detail) => document.dispatchEvent(new CustomEvent('tuita-toast', { detail }));
const messageOf = (error) => (error instanceof Error ? error.message : String(error));

/** Fills `{name}` slots of a client string. */
export const fill = (template, values) =>
	String(template ?? '').replace(/\{(\w+)\}/g, (hit, name) => (name in values ? String(values[name]) : hit));

/** A plural client string for `count`: the sample whose plural category matches the count's. */
export function pluralText(entry, count, language = lang()) {
	if (typeof entry === 'string') return fill(entry, { count });
	const rules = new Intl.PluralRules(language);
	const wanted = rules.select(count);
	const sample = Object.keys(entry).find((key) => rules.select(Number(key)) === wanted) ?? Object.keys(entry)[0];
	return fill(entry[sample], { count });
}

const compact = (n) => new Intl.NumberFormat(lang(), { notation: 'compact', maximumFractionDigits: 1 }).format(n);

async function api(method, path) {
	const response = await fetch(`/api/v1${path}`, {
		method,
		credentials: 'same-origin',
		headers: { accept: 'application/json' }
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

const buttonsOf = (acct) => [...document.querySelectorAll(`[data-follow][data-acct="${CSS.escape(acct)}"]`)];

/** Draws `view` on one follow button: label, look, `aria-pressed` and the cancel hint of a request. */
export function paint(button, view) {
	const text = strings();
	button.dataset.state = view;
	button.setAttribute('aria-pressed', String(view !== 'none'));
	button.classList.toggle('solid', view === 'none');
	button.classList.toggle('outline', view !== 'none');
	const label = button.querySelector('.label');
	if (label) label.textContent = view === 'following' ? text.following : view === 'pending' ? text.follow_requested : text.follow;
	if (view === 'pending') button.setAttribute('title', text.follow_cancel_request ?? '');
	else button.removeAttribute('title');
}

/** The followers count of the profile header of `user`, after a follow or unfollow. */
function paintCounts(user) {
	for (const header of document.querySelectorAll(`[data-c="profile-header"][data-user-id="${CSS.escape(user.id)}"]`)) {
		const count = header.querySelector('[data-count="followers"]');
		if (count) count.textContent = compact(user.counts.followers);
		const label = header.querySelector('[data-count-label="followers"]');
		if (label) label.textContent = pluralText(strings().profile_followers, user.counts.followers);
	}
}

async function toggleFollow(button) {
	const acct = button.dataset.acct;
	if (!acct || button.dataset.busy) return;
	const was = button.dataset.state;
	const next = followStep(was, button.dataset.remote === 'true');
	const group = buttonsOf(acct);
	for (const each of group) {
		each.dataset.busy = 'true';
		paint(each, next);
	}
	const text = strings();
	try {
		const fresh = await api(next === 'none' ? 'DELETE' : 'POST', `/users/${enc(acct)}/follow`);
		const settled = followView(fresh.relation);
		for (const each of group) paint(each, settled);
		if (next === 'pending') toast(fill(text.follow_requested_toast, { handle: button.dataset.handle ?? '' }));
		else if (was === 'pending') toast(text.follow_cancelled_toast);
		paintCounts(fresh);
	} catch (error) {
		for (const each of group) paint(each, was);
		toast(messageOf(error));
	} finally {
		for (const each of group) delete each.dataset.busy;
	}
}

/** `[data-profile-act]` -> the request and the toast it leaves behind. */
const RELATIONS = {
	mute: ['POST', 'mute', 'muted_toast'],
	unmute: ['DELETE', 'mute', 'unmuted_toast'],
	block: ['POST', 'block', 'blocked_toast'],
	unblock: ['DELETE', 'block', 'unblocked_toast']
};

async function relate(act, { acct, handle }) {
	const [method, kind, message] = RELATIONS[act];
	try {
		await api(method, `/users/${enc(acct)}/${kind}`);
		try {
			sessionStorage.setItem(FLASH, fill(strings()[message], { handle: handle ?? '' }));
		} catch {
			// Without storage the page just reloads.
		}
		location.reload();
	} catch (error) {
		toast(messageOf(error));
	}
}

/** The account the open block question is about. */
let blocking = null;

if (typeof document !== 'undefined' && !document.documentElement.dataset.followBound) {
	document.documentElement.dataset.followBound = '';

	try {
		const message = sessionStorage.getItem(FLASH);
		if (message) {
			sessionStorage.removeItem(FLASH);
			toast(message);
		}
	} catch {
		// Storage is off.
	}

	document.addEventListener('click', (event) => {
		const target = event.target instanceof Element ? event.target : null;
		if (!target) return;
		const follow = target.closest('[data-follow]');
		if (follow) {
			event.preventDefault();
			void toggleFollow(follow);
			return;
		}
		const act = target.closest('[data-profile-act]');
		if (!act || !RELATIONS[act.dataset.profileAct]) return;
		const { profileAct, acct, handle } = act.dataset;
		if (profileAct === 'block') {
			blocking = { acct, handle };
			document.getElementById('confirm-profile-block')?.showModal();
		} else {
			void relate(profileAct, { acct, handle });
		}
	});

	addEventListener('profile-block', () => {
		const who = blocking;
		blocking = null;
		if (who) void relate('block', who);
	});
}
