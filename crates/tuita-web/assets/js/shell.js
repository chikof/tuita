// App shell behaviour. Each piece wires itself at load; markup contracts the Rust side renders:
//
// - `[data-island="bottom-bar"]`: gets class `hidden` while scrolling down past 80px.
// - `[data-back]`: a click goes back when this site opened the page, else to `/`.
// - `[popover][role="menu"]` (items `[role="menuitem"]`) and any popover holding a `[data-roving=""]`
//   container (items: its buttons): the first item takes focus on open, arrows/Home/End move it.
// - `[data-roving="tablist"]` (tabs, segmented): ArrowLeft/Right and Home/End focus and click the neighbouring
//   enabled `[role="tab"]` (wrapping).
// - `[data-c="dialog"] .handle`: pointer drag sets `--drag` (px) on the owning `<dialog>`; past 80px it closes.
// - `[data-island="announcements"]`: a `<section>` of `.banner[data-announcement-id]`, each with a
//   `button[data-dismiss]`. Dismissals persist in localStorage `tuita:ann:<id>`; the section goes when empty.
// - `[data-grain]`: the page wants the film grain on `<body>` (landing, sign-in, sign-up) while it is open.
// - `[data-island="clock"]`: holds `[data-part="h"|"m"|"s"]` spans, set to the visitor's local time every second.

export const DRAG_CLOSE_PX = 80;

/** Whether `referrer` is a page of the same origin, so `history.back()` stays on this site. */
export function canGoBack(referrer, origin) {
	try {
		return referrer !== '' && new URL(referrer).origin === origin;
	} catch {
		return false;
	}
}

/** The item to focus after `key` from item `at` of `length`; -1 when the key is not a menu key. */
export function nextIndex(key, at, length) {
	if (length <= 0) return -1;
	switch (key) {
		case 'ArrowDown':
			return (at + 1) % length;
		case 'ArrowUp':
			return at <= 0 ? length - 1 : at - 1;
		case 'Home':
			return 0;
		case 'End':
			return length - 1;
		default:
			return -1;
	}
}

/** A drag of `dy` px closes the dialog when it passes the threshold. */
export const dragCloses = (dy) => dy > DRAG_CLOSE_PX;

export const dismissKey = (id) => `tuita:ann:${id}`;

/** Storage that throws counts as "not dismissed". */
export function isDismissed(store, id) {
	try {
		return store.getItem(dismissKey(id)) !== null;
	} catch {
		return false;
	}
}

/** Private-mode storage may refuse; the banner then returns on reload. */
export function dismiss(store, id) {
	try {
		store.setItem(dismissKey(id), '1');
	} catch {
		// The banner is still removed from this page view.
	}
}

export const undismissed = (items, store) => items.filter((item) => !isDismissed(store, item.id));

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

const pad = (n) => String(n).padStart(2, '0');

function wireClock(clock) {
	const part = (name) => clock.querySelector(`[data-part="${name}"]`);
	const tick = () => {
		const now = new Date();
		for (const [name, value] of [['h', now.getHours()], ['m', now.getMinutes()], ['s', now.getSeconds()]]) {
			const el = part(name);
			if (el) el.textContent = pad(value);
		}
	};
	tick();
	const timer = setInterval(tick, 1000);
	return () => clearInterval(timer);
}

function wireGrain() {
	document.body.classList.add('grain');
	return () => document.body.classList.remove('grain');
}

function menuItems(panel) {
	if (panel.matches('[role="menu"]')) return [...panel.querySelectorAll('[role="menuitem"]:not(:disabled)')];
	const roving = panel.querySelector('[data-roving=""]');
	return roving ? [...roving.querySelectorAll('button:not(:disabled)')] : [];
}

const isMenuPanel = (el) => el.matches('[role="menu"]') || el.querySelector('[data-roving=""]') !== null;

function wireBottomBar(bar) {
	let last = 0;
	const onscroll = () => {
		const y = scrollY;
		if (Math.abs(y - last) < 8) return;
		bar.classList.toggle('hidden', y > last && y > 80);
		last = y;
	};
	addEventListener('scroll', onscroll, { passive: true });
	return () => removeEventListener('scroll', onscroll);
}

function wireAnnouncements(section) {
	const store = globalThis.localStorage;
	for (const banner of section.querySelectorAll('.banner[data-announcement-id]')) {
		if (isDismissed(store, banner.dataset.announcementId)) banner.remove();
	}
	const prune = () => {
		if (!section.querySelector('.banner')) section.remove();
	};
	section.addEventListener('click', (event) => {
		const button = event.target.closest('[data-dismiss]');
		const banner = button?.closest('.banner[data-announcement-id]');
		if (!banner) return;
		dismiss(store, banner.dataset.announcementId);
		banner.remove();
		prune();
	});
	prune();
}

if (typeof document !== 'undefined' && !document.documentElement.dataset.shellBound) {
	document.documentElement.dataset.shellBound = '';

	mount('[data-island="bottom-bar"]', wireBottomBar);
	mount('[data-island="announcements"]', wireAnnouncements);
	mount('[data-island="clock"]', wireClock);
	mount('[data-grain]', wireGrain);

	document.addEventListener('click', (event) => {
		if (!event.target.closest?.('[data-back]')) return;
		event.preventDefault();
		if (canGoBack(document.referrer, location.origin)) history.back();
		else location.assign('/');
	});

	document.addEventListener(
		'toggle',
		(event) => {
			const panel = event.target;
			if (event.newState !== 'open' || !panel.matches?.('[popover]') || !isMenuPanel(panel)) return;
			menuItems(panel)[0]?.focus();
		},
		true
	);

	document.addEventListener('keydown', (event) => {
		const list = event.target.closest?.('[data-roving="tablist"]');
		if (list) {
			const tabs = [...list.querySelectorAll('[role="tab"]:not(:disabled)')];
			const key = { ArrowLeft: 'ArrowUp', ArrowRight: 'ArrowDown', Home: 'Home', End: 'End' }[event.key];
			const next = key ? nextIndex(key, tabs.indexOf(document.activeElement), tabs.length) : -1;
			if (next < 0) return;
			event.preventDefault();
			tabs[next].focus();
			tabs[next].click();
			return;
		}
		const panel = event.target.closest?.('[popover]');
		if (!panel || !isMenuPanel(panel)) return;
		const items = menuItems(panel);
		const next = nextIndex(event.key, items.indexOf(document.activeElement), items.length);
		if (next < 0) return;
		event.preventDefault();
		items[next].focus();
	});

	document.addEventListener('pointerdown', (event) => {
		const handle = event.target.closest?.('[data-c="dialog"] .handle');
		const dialog = handle?.closest('dialog');
		if (!dialog) return;
		const origin = event.clientY;
		let drag = 0;
		handle.setPointerCapture?.(event.pointerId);
		const move = (e) => {
			drag = Math.max(0, e.clientY - origin);
			dialog.style.setProperty('--drag', `${drag}px`);
		};
		const end = () => {
			handle.removeEventListener('pointermove', move);
			handle.removeEventListener('pointerup', end);
			handle.removeEventListener('pointercancel', end);
			if (dragCloses(drag)) dialog.close();
			dialog.style.removeProperty('--drag');
		};
		handle.addEventListener('pointermove', move);
		handle.addEventListener('pointerup', end);
		handle.addEventListener('pointercancel', end);
	});
}
