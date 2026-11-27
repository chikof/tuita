// Profile hover card. Triggers are `[data-hovercard="ada"]` links; the card is a server fragment from
// `/_tuita/hovercard/<acct>`, shown as a manual popover below the trigger.

export const DELAYS = { open: 450, focus: 600, close: 250 };

const realTimers = {
	set: (run, ms) => setTimeout(run, ms),
	clear: (handle) => clearTimeout(handle)
};

/**
 * The open/close timing of the one hover card. Pure: it only calls `onchange` with the target that
 * should be shown (or `null`), so the timers can be faked in tests.
 *
 * A pending open belongs to one target; the close timer is shared, so moving from the trigger into
 * the card cancels it, and moving to another trigger lets the old card close on its own.
 */
export function createHoverCard(onchange, delays = DELAYS, timers = realTimers) {
	let shown = null;
	let pending = null;
	let closing = null;
	let overTrigger = null;
	let overCard = false;

	const stopPending = () => {
		if (pending) timers.clear(pending.handle);
		pending = null;
	};
	const stopClosing = () => {
		if (closing !== null) timers.clear(closing);
		closing = null;
	};
	const hide = () => {
		stopClosing();
		if (shown === null) return;
		shown = null;
		onchange(null);
	};
	const show = (target) => {
		stopPending();
		stopClosing();
		if (shown === target) return;
		shown = target;
		onchange(target);
	};
	const schedule = (target, ms) => {
		if (shown === target) {
			stopClosing();
			return;
		}
		if (pending?.target === target) return;
		stopPending();
		pending = { target, handle: timers.set(() => show(target), ms) };
	};
	const scheduleClose = () => {
		if (shown === null || closing !== null) return;
		closing = timers.set(hide, delays.close);
	};

	return {
		get shown() {
			return shown;
		},
		enter(target) {
			overTrigger = target;
			schedule(target, delays.open);
		},
		leave(target) {
			if (overTrigger === target) overTrigger = null;
			if (pending?.target === target) stopPending();
			if (shown === target && !overCard) scheduleClose();
		},
		enterCard() {
			overCard = true;
			stopClosing();
		},
		leaveCard() {
			overCard = false;
			if (overTrigger !== shown) scheduleClose();
		},
		focus(target) {
			schedule(target, delays.focus);
		},
		/** Losing focus closes the card at once, unless the pointer is still on it. */
		blur(target) {
			if (pending?.target === target) stopPending();
			if (shown === target && overTrigger !== target && !overCard) hide();
		},
		/** Escape, a click on the trigger, scrolling: close now and forget the pointer. */
		close() {
			overTrigger = null;
			overCard = false;
			stopPending();
			hide();
		}
	};
}

/** Places `panel` below `trigger` (fixed), flipping above when it would overflow, kept on screen. */
export function placeBelow(trigger, panel) {
	const t = trigger.getBoundingClientRect();
	const p = panel.getBoundingClientRect();
	const gap = 6;
	const fitsBelow = t.bottom + gap + p.height <= window.innerHeight;
	const top = fitsBelow ? t.bottom + gap : Math.max(gap, t.top - gap - p.height);
	const left = Math.min(Math.max(gap, t.left), Math.max(gap, window.innerWidth - p.width - gap));
	panel.style.top = `${top}px`;
	panel.style.left = `${left}px`;
}

const TTL = 60_000;
const cache = new Map();
const inflight = new Map();

/** The card fragment of `acct` (`null` when the server shows none); a minute of cache, one request per account. */
export function loadHoverCard(acct) {
	const hit = cache.get(acct);
	if (hit && Date.now() - hit.at < TTL) return Promise.resolve(hit.html);
	let request = inflight.get(acct);
	if (!request) {
		request = fetch(`/_tuita/hovercard/${encodeURIComponent(acct)}`, { credentials: 'same-origin' })
			.then(async (response) => {
				if (!response.ok || response.status === 204) return null;
				const html = await response.text();
				cache.set(acct, { at: Date.now(), html });
				return html;
			})
			.catch(() => null)
			.finally(() => inflight.delete(acct));
		inflight.set(acct, request);
	}
	return request;
}

export function forgetHoverCards() {
	cache.clear();
	inflight.clear();
}

if (typeof document !== 'undefined' && !document.documentElement.dataset.hovercardBound) {
	document.documentElement.dataset.hovercardBound = '';
	let card = null;
	let anchor = null;
	let resize = null;

	const remove = () => {
		resize?.disconnect();
		resize = null;
		card?.remove();
		card = null;
		anchor?.removeAttribute('aria-describedby');
		anchor = null;
	};

	const hover = createHoverCard((trigger) => {
		remove();
		if (trigger === null) return;
		void open(trigger);
	});

	async function open(trigger) {
		const html = await loadHoverCard(trigger.dataset.hovercard);
		// The pointer may have moved on while the request ran.
		if (html === null || hover.shown !== trigger || !trigger.isConnected) return;
		const page = document.createElement('template');
		page.innerHTML = html;
		const el = page.content.firstElementChild;
		if (!el) return;
		remove();
		el.id = 'hovercard';
		el.style.margin = '0';
		el.style.right = 'auto';
		el.style.bottom = 'auto';
		document.body.append(el);
		card = el;
		anchor = trigger;
		trigger.setAttribute('aria-describedby', 'hovercard');
		el.showPopover();
		placeBelow(trigger, el);
		resize = new ResizeObserver(() => placeBelow(trigger, el));
		resize.observe(el);
		el.addEventListener('pointerenter', () => hover.enterCard());
		el.addEventListener('pointerleave', () => hover.leaveCard());
		el.addEventListener('click', (event) => {
			if (event.target instanceof Element && event.target.closest('a')) hover.close();
		});
	}

	const triggerOf = (event) =>
		event.target instanceof Element && event.target.matches('[data-hovercard]') ? event.target : null;
	// Skipped inside dialogs and menus, where a card in the top layer would sit outside the modal.
	const blocked = (el) => el.closest('dialog, [role=menu]') !== null;
	// Only a mouse hovers: touch and pen never open the card. A `hover: hover` media query is not used because it headless browsers deny although a mouse is moving.
	const canHover = (event) => event.pointerType === 'mouse';

	// pointerenter/leave and focus/blur do not bubble; the capture phase sees them on the document.
	document.addEventListener(
		'pointerenter',
		(event) => {
			const el = triggerOf(event);
			if (el && event.buttons === 0 && canHover(event) && !blocked(el)) hover.enter(el);
		},
		true
	);
	document.addEventListener(
		'pointerleave',
		(event) => {
			const el = triggerOf(event);
			if (el) hover.leave(el);
		},
		true
	);
	document.addEventListener('pointerdown', (event) => {
		const el = event.target instanceof Element ? event.target.closest('[data-hovercard]') : null;
		if (!el) return;
		hover.leave(el);
		if (hover.shown === el) hover.close();
	}, true);
	document.addEventListener(
		'focus',
		(event) => {
			const el = triggerOf(event);
			if (el && el.matches(':focus-visible') && !blocked(el)) hover.focus(el);
		},
		true
	);
	document.addEventListener(
		'blur',
		(event) => {
			const el = triggerOf(event);
			if (el) hover.blur(el);
		},
		true
	);
	document.addEventListener('keydown', (event) => {
		if (event.key === 'Escape' && hover.shown !== null) hover.close();
	});
	document.addEventListener(
		'scroll',
		(event) => {
			if (hover.shown !== null && !(card && event.target instanceof Node && card.contains(event.target))) hover.close();
		},
		true
	);
	addEventListener('resize', () => hover.close());
}
