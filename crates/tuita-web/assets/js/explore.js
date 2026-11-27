// Explore: the recent searches of the search field, `[data-island="recent-searches"]` (a
// `<form role="search">` with an `input[role=combobox]`, a hidden `.recent` panel holding
// `ul[role=listbox]` and a clear button `[data-recent="clear"]`, `<template data-recent-icon>` with
// the row icon and `[data-op]` buttons of the search tips).
//
// Searches are stored per account in `localStorage` `tuita:recent-searches:<user id>` (8 at most,
// newest first); any click on `[data-forget-keys="<id> <id>"]` (every sign-out control) deletes the
// keys of those accounts before the request runs.

const MAX = 8;
const RECENT = 'tuita:recent-searches';

export const recentKey = (userId) => `${RECENT}:${userId}`;

/** `q` first, any earlier copy of it (ignoring case) dropped, at most 8 kept. */
export const withRecent = (list, q) => [q, ...list.filter((x) => x.toLowerCase() !== q.toLowerCase())].slice(0, MAX);

/** The stored searches under `key`; empty when there is none, it is damaged or storage is off. */
export function readRecent(storage, key) {
	try {
		const stored = JSON.parse(storage.getItem(key) ?? '[]');
		return Array.isArray(stored) ? stored.filter((x) => typeof x === 'string').slice(0, MAX) : [];
	} catch {
		return [];
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

const enc = encodeURIComponent;

function search(form) {
	const key = recentKey(form.dataset.userId ?? '');
	const field = form.querySelector('input[name="q"]');
	const panel = form.querySelector('.recent');
	const list = form.querySelector('[role="listbox"]');
	const icon = form.querySelector('template[data-recent-icon]');
	if (!field || !panel || !list) return undefined;

	let open = false;
	let active = -1;
	let recent = [];

	const read = () => {
		try {
			return readRecent(localStorage, key);
		} catch {
			return [];
		}
	};
	const write = (items) => {
		try {
			if (items.length) localStorage.setItem(key, JSON.stringify(items));
			else localStorage.removeItem(key);
		} catch {
			// Searching works without storage.
		}
	};
	const remember = (q) => write(withRecent(read(), q));
	const optionId = (index) => `${list.id}-option-${index}`;
	const shown = () => {
		const needle = field.value.trim().toLowerCase();
		return recent.filter((entry) => entry.toLowerCase().includes(needle));
	};

	function render() {
		const items = shown();
		const listed = open && items.length > 0;
		const current = listed && active < items.length ? active : -1;
		panel.hidden = !listed;
		field.setAttribute('aria-expanded', String(listed));
		if (current >= 0) field.setAttribute('aria-activedescendant', optionId(current));
		else field.removeAttribute('aria-activedescendant');
		list.replaceChildren(
			...items.map((query, i) => {
				const item = document.createElement('li');
				item.setAttribute('role', 'none');
				const link = document.createElement('a');
				link.id = optionId(i);
				link.className = i === current ? 'entry wash active' : 'entry wash';
				link.setAttribute('role', 'option');
				link.setAttribute('aria-selected', String(i === current));
				link.tabIndex = -1;
				link.href = `/explore?q=${enc(query)}`;
				link.dataset.query = query;
				if (icon) link.append(icon.content.cloneNode(true));
				const text = document.createElement('span');
				text.textContent = query;
				link.append(text);
				item.append(link);
				return item;
			})
		);
	}

	const reveal = () => {
		if (!open) {
			recent = read();
			active = -1;
		}
		open = true;
		render();
	};
	const close = () => {
		open = false;
		active = -1;
		render();
	};

	function move(step) {
		reveal();
		const count = shown().length;
		if (!count) return;
		const current = active >= 0 && active < count ? active : -1;
		active = current < 0 ? (step > 0 ? 0 : count - 1) : (current + step + count) % count;
		render();
		document.getElementById(optionId(active))?.scrollIntoView({ block: 'nearest' });
	}

	const on = (target, type, handler, options) => {
		target.addEventListener(type, handler, options);
		return () => target.removeEventListener(type, handler, options);
	};
	const stops = [
		on(form, 'focusin', reveal),
		on(form, 'focusout', (event) => {
			if (!(event.relatedTarget instanceof Node && form.contains(event.relatedTarget))) close();
		}),
		on(field, 'click', reveal),
		on(field, 'input', () => {
			active = -1;
			reveal();
		}),
		on(field, 'keydown', (event) => {
			const items = shown();
			const current = open && active >= 0 && active < items.length ? active : -1;
			if (event.key === 'Escape' && open) {
				event.preventDefault();
				close();
			} else if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
				event.preventDefault();
				move(event.key === 'ArrowDown' ? 1 : -1);
			} else if (event.key === 'Enter' && current >= 0) {
				event.preventDefault();
				const query = items[current];
				field.value = query;
				remember(query);
				close();
				location.assign(`/explore?q=${enc(query)}`);
			}
		}),
		on(form, 'submit', () => {
			const text = field.value.trim();
			if (text) remember(text);
			close();
		}),
		// Keep focus in the field while a mouse picks an entry or clears the list.
		on(panel, 'mousedown', (event) => event.preventDefault()),
		on(panel, 'click', (event) => {
			const target = event.target instanceof Element ? event.target : null;
			const entry = target?.closest('[data-query]');
			if (entry) {
				remember(entry.dataset.query);
				close();
				return;
			}
			if (target?.closest('[data-recent="clear"]')) {
				write([]);
				recent = [];
				render();
				field.focus();
			}
		}),
		on(form, 'click', (event) => {
			const op = event.target instanceof Element ? event.target.closest('[data-op]') : null;
			if (!op) return;
			op.closest('[popover]')?.hidePopover();
			field.value = `${field.value.trim()} ${op.dataset.op}`.trim();
			field.focus();
			field.setSelectionRange(field.value.length, field.value.length);
		})
	];
	return () => stops.forEach((stop) => stop());
}

if (typeof document !== 'undefined' && !document.documentElement.dataset.exploreBound) {
	document.documentElement.dataset.exploreBound = '';
	mount('[data-island="recent-searches"]', search);

	// Sign-out controls name the accounts whose device data goes; this runs before the request.
	document.addEventListener(
		'click',
		(event) => {
			const control = event.target instanceof Element ? event.target.closest('[data-forget-keys]') : null;
			if (!control) return;
			for (const id of (control.dataset.forgetKeys ?? '').split(/\s+/).filter(Boolean)) {
				try {
					localStorage.removeItem(recentKey(id));
				} catch {
					// Storage is off: nothing was kept.
				}
			}
		},
		true
	);
}
