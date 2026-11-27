// The scheduled posts page: change the time, post now and cancel.
//
// Root: `[data-island="scheduled"]` holding `[data-scheduled-body]` (the list; a fragment request of the page
// answers a fresh one, which replaces it), `dialog#reschedule` with `form#reschedule-form` (input `[name=when]`,
// `.dim[data-zone-note]`, `.error`) and `dialog#confirm-scheduled-cancel` whose confirm button
// `[data-act="sched-cancel-yes"]` carries the two labels in `data-pending` and `data-failed`.
// Cards are `article[data-id][data-status]` with buttons `[data-act="sched-resched|sched-publish|sched-cancel"]`.
// The page knows no time zone: this island writes it under the bar's title and renders `time[data-local]` in the
// viewer's zone.
//
// Client strings: scheduled_zone, scheduled_posted, scheduled_window_error, view.

const enc = encodeURIComponent;

const toast = (detail) => document.dispatchEvent(new CustomEvent('tuita-toast', { detail }));

function strings() {
	try {
		return JSON.parse(document.getElementById('tuita-l10n')?.textContent ?? '{}');
	} catch {
		return {};
	}
}

/** Fills `{name}` slots of a client string. */
export function fill(template, values) {
	return String(template ?? '').replace(/\{(\w+)\}/g, (hit, name) => (name in values ? String(values[name]) : hit));
}

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

/** `toLocaleString` of an ISO instant in the page language, or `null` for a bad one. */
export function localTime(iso, language = document.documentElement.lang || 'en') {
	const at = new Date(iso);
	return Number.isNaN(at.getTime()) ? null : at.toLocaleString(language);
}

async function api(method, path, body) {
	const response = await fetch(`/api/v1${path}`, {
		method,
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

const zone = () => Intl.DateTimeFormat().resolvedOptions().timeZone;

/** Writes the zone and the local times; run again after the list was swapped. */
function localize(root) {
	const sub = root.querySelector('[data-c="page-bar"] .sub');
	if (sub) sub.textContent = zone();
	const note = root.querySelector('[data-zone-note]');
	if (note) note.textContent = fill(strings().scheduled_zone, { zone: zone() });
	for (const el of root.querySelectorAll('time[data-local]')) {
		const text = localTime(el.getAttribute('datetime'));
		if (text) el.textContent = text;
	}
}

/** Asks the page for a fresh list and swaps it in. */
async function reload(root) {
	try {
		const response = await fetch(location.pathname, {
			credentials: 'same-origin',
			headers: { 'x-tuita-fragment': '1', accept: 'text/html' }
		});
		if (!response.ok) throw new Error(String(response.status));
		const holder = document.createElement('template');
		holder.innerHTML = await response.text();
		const fresh = holder.content.querySelector('[data-scheduled-body]');
		if (fresh) root.querySelector('[data-scheduled-body]')?.replaceWith(fresh);
		localize(root);
	} catch (error) {
		toast(messageOf(error));
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

if (typeof document !== 'undefined' && !document.documentElement.dataset.scheduledBound) {
	document.documentElement.dataset.scheduledBound = '';
	mount('[data-island="scheduled"]', (root) => {
		localize(root);
		let target = null;
		const dialog = () => root.querySelector('#reschedule');
		const form = () => root.querySelector('#reschedule-form');

		function openReschedule(card) {
			target = card.dataset.id;
			const input = form().elements.namedItem('when');
			const bounds = localBounds();
			input.min = bounds.min;
			input.max = bounds.max;
			input.value = toLocalInput(new Date(card.dataset.publishAt));
			const error = form().querySelector('.error');
			error.hidden = true;
			dialog().showModal();
		}

		async function saveReschedule() {
			const input = form().elements.namedItem('when');
			const error = form().querySelector('.error');
			const iso = fromLocalInput(input.value);
			if (!iso || !inWindow(iso)) {
				error.textContent = strings().scheduled_window_error ?? '';
				error.hidden = false;
				return;
			}
			error.hidden = true;
			try {
				await api('PATCH', `/scheduled/${enc(target)}`, { publishAt: iso });
				dialog().close();
				await reload(root);
			} catch (failure) {
				error.textContent = messageOf(failure);
				error.hidden = false;
			}
		}

		async function publish(card) {
			try {
				const posted = await api('POST', `/scheduled/${enc(card.dataset.id)}/publish`);
				const mine = document.querySelector('[data-island="stream"][data-handle]')?.dataset.handle;
				toast(
					mine && posted?.id
						? { message: strings().scheduled_posted, label: strings().view, href: `/@${mine}/${posted.id}` }
						: strings().scheduled_posted
				);
				await reload(root);
			} catch (error) {
				toast(messageOf(error));
			}
		}

		function askCancel(card) {
			target = card.dataset.id;
			const yes = root.querySelector('[data-act="sched-cancel-yes"]');
			const label = card.dataset.status === 'failed' ? yes.dataset.failed : yes.dataset.pending;
			yes.querySelector('.label').textContent = label;
			root.querySelector('#confirm-scheduled-cancel').showModal();
		}

		async function cancel() {
			try {
				await api('DELETE', `/scheduled/${enc(target)}`);
				await reload(root);
			} catch (error) {
				toast(messageOf(error));
			}
		}

		const onClick = (event) => {
			const act = event.target instanceof Element ? event.target.closest('[data-act^="sched-"]') : null;
			if (!act || !root.contains(act)) return;
			const card = act.closest('[data-c="scheduled-card"]');
			switch (act.dataset.act) {
				case 'sched-resched':
					if (card) openReschedule(card);
					break;
				case 'sched-publish':
					if (card) void publish(card);
					break;
				case 'sched-cancel':
					if (card) askCancel(card);
					break;
				case 'sched-cancel-yes':
					void cancel();
					break;
			}
		};
		const onSubmit = (event) => {
			if (event.target !== form()) return;
			event.preventDefault();
			void saveReschedule();
		};
		root.addEventListener('click', onClick);
		root.addEventListener('submit', onSubmit);
		return () => {
			root.removeEventListener('click', onClick);
			root.removeEventListener('submit', onSubmit);
		};
	});
}
