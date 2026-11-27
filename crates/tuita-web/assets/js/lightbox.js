// Media: the strip of a post's pictures (layout, pager, arrows) and the lightbox.
//
// Strip: `[data-island="media-grid"]` is the `.carousel` root with a `.track` of `.cell[data-aspect]`,
// `.pager`, `.nav.prev|.next` (buttons `data-act="media-prev|media-next"`). Pictures:
// `.cell .open img` hide their `.skeleton` once loaded. A picture button with `data-act="lightbox"`,
// `data-index` and `data-post-id` opens the viewer: `/_tuita/lightbox/<id>?index=<n>` answers a
// `<dialog data-island="lightbox">` that this module puts into `#lightbox-host`, opens, and removes
// when it closes. Inside it `.slide[data-index][data-alt]`, `.count`, `.caption`, `.frame.panel`,
// buttons `data-act="lightbox-prev|lightbox-next|lightbox-panel"`.

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

/** Pixels between two items of the strip. */
export const GAP = 2;
/** How much of the next item stays visible so the strip reads as scrollable. */
export const PEEK = 24;
/** The tallest a strip gets, as for a single photo. */
export const MAX_HEIGHT = 560;

const NO_SIZE_ASPECT = 4 / 3;
/** The first picture sets the strip height within 4:5 and 16:9. */
const FIRST_MIN = 4 / 5;
const WIDE = 16 / 9;
/** An item is never narrower than 1:2 nor wider than 16:9. */
const ITEM_MIN = 1 / 2;

const clamp = (value, min, max) => Math.min(max, Math.max(min, value));

/** Width over height; items without usable dimensions count as 4:3. */
export const aspectOf = ({ width, height }) =>
	width && height && width > 0 && height > 0 ? width / height : NO_SIZE_ASPECT;

/**
 * Item sizes for a strip in a column `columnWidth` px wide. The first item sets the height; every
 * item keeps its own shape within limits, so wide ones are cropped by `object-fit: cover`. A
 * portrait first item gets a height that fits two items side by side, plus a peek of the third.
 */
export function carouselLayout(items, columnWidth) {
	if (items.length === 0 || !(columnWidth > 0)) return { height: 0, widths: items.map(() => 0) };
	const many = items.length > 1;
	const firstShape = clamp(aspectOf(items[0]), ITEM_MIN, WIDE);
	let height = columnWidth / clamp(aspectOf(items[0]), FIRST_MIN, WIDE);
	if (many && firstShape < 1) {
		const room = columnWidth - GAP - (items.length > 2 ? PEEK : 0);
		height = Math.min(height, room / (2 * firstShape));
	}
	height = Math.floor(Math.min(height, MAX_HEIGHT));
	const widest = many ? columnWidth - PEEK : columnWidth;
	const widths = items.map((item) => Math.floor(Math.min(height * clamp(aspectOf(item), ITEM_MIN, WIDE), widest)));
	return { height, widths };
}

/** Left edge of every item inside the scroll content. */
export function starts(widths) {
	let left = 0;
	return widths.map((width) => {
		const start = left;
		left += width + GAP;
		return start;
	});
}

const maxScroll = (widths, viewport) => Math.max(0, widths.reduce((sum, width) => sum + width + GAP, -GAP) - viewport);

/** Where one step in `direction` (1 or -1) scrolls to: the next item start, never past the ends. */
export function scrollTarget(widths, scrollLeft, viewport, direction) {
	const max = maxScroll(widths, viewport);
	const lefts = starts(widths);
	if (direction === 1) {
		const next = lefts.find((left) => left > scrollLeft + 1);
		return next === undefined ? max : Math.min(next, max);
	}
	const prev = lefts.findLast((left) => left < scrollLeft - 1);
	return prev === undefined ? 0 : prev;
}

/** The item the pager names: the first one still in view, the last one at the far end. */
export function currentIndex(widths, scrollLeft, viewport) {
	const max = maxScroll(widths, viewport);
	if (max > 0 && scrollLeft >= max - 1) return widths.length - 1;
	const lefts = starts(widths);
	const index = lefts.findIndex((left, i) => left + widths[i] > scrollLeft + 1);
	return index < 0 ? 0 : index;
}

/** Whether the strip overflows its column, so it needs arrows and a pager. */
export const scrolls = (widths, viewport) => maxScroll(widths, viewport) > 1;

/** Next index with wrap-around; `step` is 1 or -1. */
export const wrapIndex = (index, step, length) => (length === 0 ? 0 : (index + step + length) % length);

function clientString(key) {
	try {
		return JSON.parse(document.getElementById('tuita-l10n')?.textContent ?? '{}')[key] ?? key;
	} catch {
		return key;
	}
}

const calm = () => document.documentElement.dataset.motion === 'reduce';

// ---------------------------------------------------------------------------------------------
// The strip

function strip(root) {
	const track = root.querySelector('.track');
	const cells = [...root.querySelectorAll('.cell')];
	const pager = root.querySelector('.pager');
	const prev = root.querySelector('.nav.prev');
	const next = root.querySelector('.nav.next');
	const items = cells.map((cell) => ({ width: Number(cell.dataset.aspect) || 4 / 3, height: 1 }));
	let widths = cells.map((cell) => cell.offsetWidth);

	const sync = () => {
		const column = root.clientWidth;
		const left = track.scrollLeft;
		const overflow = cells.length > 1 && scrolls(widths, column);
		const current = currentIndex(widths, left, column);
		pager.hidden = !overflow;
		if (overflow) pager.textContent = clientString('media_pager').replace('{index}', current + 1).replace('{count}', cells.length);
		prev.hidden = !overflow || left <= 1;
		next.hidden = !overflow || current === cells.length - 1;
	};

	const layout = () => {
		const column = root.clientWidth;
		if (!(column > 0)) return;
		const result = carouselLayout(items, column);
		track.style.height = `${result.height}px`;
		cells.forEach((cell, i) => (cell.style.width = `${result.widths[i]}px`));
		widths = result.widths;
		sync();
	};

	const step = (direction) => {
		const left = scrollTarget(widths, track.scrollLeft, track.clientWidth, direction);
		track.scrollTo({ left, behavior: calm() ? 'auto' : 'smooth' });
		const max = track.scrollWidth - track.clientWidth;
		if (left <= 1 || left >= max - 1) track.focus({ preventScroll: true });
	};

	const click = (event) => {
		const button = event.target instanceof Element ? event.target.closest('[data-act="media-prev"], [data-act="media-next"]') : null;
		if (button && root.contains(button)) step(button.dataset.act === 'media-next' ? 1 : -1);
	};
	const key = (event) => {
		if (event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;
		if (event.key === 'ArrowRight') step(1);
		else if (event.key === 'ArrowLeft') step(-1);
		else return;
		event.preventDefault();
	};

	const observer = new ResizeObserver(layout);
	observer.observe(root);
	track.addEventListener('scroll', sync, { passive: true });
	track.addEventListener('keydown', key);
	root.addEventListener('click', click);
	layout();
	return () => {
		observer.disconnect();
		track.removeEventListener('scroll', sync);
		track.removeEventListener('keydown', key);
		root.removeEventListener('click', click);
	};
}

function picture(img) {
	const cover = img.closest('.cell')?.querySelector('.skeleton');
	if (!cover) return;
	const done = () => (cover.hidden = true);
	if (img.complete && img.naturalWidth > 0) done();
	else img.addEventListener('load', done, { once: true });
}

// ---------------------------------------------------------------------------------------------
// The viewer

/** Whether the viewer hid the side panel; it stays hidden for the next picture opened. */
let panelHidden = false;

function viewer(dialog) {
	const count = Number(dialog.dataset.count);
	const many = count > 1;
	const slides = [...dialog.querySelectorAll('.slide')];
	const counter = dialog.querySelector('.count');
	const caption = dialog.querySelector('.caption');
	const frame = dialog.querySelector('.frame');
	const toggle = dialog.querySelector('[data-act="lightbox-panel"]');
	let index = Number(dialog.dataset.index);

	const paintPanel = () => {
		frame.classList.toggle('panel', !panelHidden);
		const label = panelHidden ? dialog.dataset.showLabel : dialog.dataset.hideLabel;
		toggle?.setAttribute('aria-label', label);
	};

	const show = (to) => {
		const from = index;
		index = to;
		slides.forEach((slide, i) => {
			slide.hidden = i !== index;
			if (i === from && i !== index) slide.querySelector('video')?.pause();
		});
		const alt = slides[index]?.dataset.alt ?? '';
		caption.textContent = alt;
		caption.hidden = alt === '';
		if (many) counter.textContent = `${index + 1} / ${count}`;
		if (from !== index) slides[index]?.querySelector('video')?.play()?.catch(() => {});
	};

	const step = (by) => show(wrapIndex(index, by, count));

	const key = (event) => {
		if (!many || event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;
		// Text fields, the panel, menus and the video controls keep their arrow keys.
		if (event.target.closest('input, textarea, select, [contenteditable], [role=slider], [role=menu], .side, .video')) return;
		if (event.key === 'ArrowRight') step(1);
		else if (event.key === 'ArrowLeft') step(-1);
		else return;
		event.preventDefault();
	};

	let swipe = 0;
	const stage = dialog.querySelector('.stage');
	const down = (event) => (swipe = event.clientX);
	const up = (event) => {
		const delta = event.clientX - swipe;
		if (many && Math.abs(delta) > 60) step(delta < 0 ? 1 : -1);
	};

	const click = (event) => {
		const button = event.target instanceof Element ? event.target.closest('[data-act^="lightbox-"]') : null;
		if (!button || !dialog.contains(button)) return;
		if (button.dataset.act === 'lightbox-prev') step(-1);
		else if (button.dataset.act === 'lightbox-next') step(1);
		else if (button.dataset.act === 'lightbox-panel') {
			panelHidden = !panelHidden;
			paintPanel();
		}
	};

	dialog.addEventListener('keydown', key);
	dialog.addEventListener('click', click);
	stage.addEventListener('pointerdown', down);
	stage.addEventListener('pointerup', up);
	paintPanel();
	show(index);
	if (!dialog.open) dialog.showModal();
	// The viewer is gone with its dialog: pause what plays and drop it from the page.
	dialog.addEventListener(
		'close',
		() => {
			for (const video of dialog.querySelectorAll('video')) video.pause();
			dialog.remove();
		},
		{ once: true }
	);
	return () => {
		dialog.removeEventListener('keydown', key);
		dialog.removeEventListener('click', click);
	};
}

async function open(button) {
	const host = document.getElementById('lightbox-host');
	if (!host || !button.dataset.postId) return;
	try {
		const response = await fetch(`/_tuita/lightbox/${encodeURIComponent(button.dataset.postId)}?index=${encodeURIComponent(button.dataset.index ?? '0')}`, {
			credentials: 'same-origin',
			headers: { accept: 'text/html' }
		});
		if (!response.ok) return;
		const holder = document.createElement('template');
		holder.innerHTML = await response.text();
		const dialog = holder.content.querySelector('dialog');
		if (dialog) host.replaceChildren(dialog);
	} catch {
		// The picture stays where it is; nothing opened.
	}
}

if (typeof document !== 'undefined' && !document.documentElement.dataset.lightboxBound) {
	document.documentElement.dataset.lightboxBound = '';
	document.addEventListener('click', (event) => {
		const button = event.target instanceof Element ? event.target.closest('[data-act="lightbox"]') : null;
		if (button) void open(button);
	});
	mount('[data-island="media-grid"]', strip);
	mount('[data-c="media-grid"] .open img', picture);
	mount('dialog[data-island="lightbox"]', viewer);
}
