// Pull to refresh on the home timeline. On a phone, pulling
// the page down from the very top shows a strip; letting go past the threshold asks the feed to start over.
//
// Active while a `[data-island="feed"]` is on the page. The refresh itself is `newposts.js`: this island
// dispatches a bubbling `tuita-feed-refresh` on `document` with `detail = { waits: [] }`; a listener pushes its
// promise onto `waits`, and the strip stays up until all of them settled and at least 600 ms passed.
//
// Client strings: pull_refresh, loading.

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

/** Finger travel is damped, so a long drag is needed to trigger a refresh. */
export const RESISTANCE = 0.5;
export const THRESHOLD = 64;
export const MAX_PULL = 96;
/** The strip stays at least this long after a refresh was asked for. */
export const MIN_BUSY_MS = 600;
/** Height of the strip while it spins. */
export const BUSY_HEIGHT = 48;

/** How far the indicator follows a finger that started at `startY` and is now at `y`. */
export const pullDistance = (startY, y) => Math.min(MAX_PULL, Math.max(0, (y - startY) * RESISTANCE));

export const shouldRefresh = (distance) => distance >= THRESHOLD;

function strings() {
	try {
		return JSON.parse(document.getElementById('tuita-l10n')?.textContent ?? '{}');
	} catch {
		return {};
	}
}

/** The `ui::loader` dots markup. */
function dots(label) {
	const loader = document.createElement('span');
	loader.className = 'loader dots';
	loader.dataset.c = 'loader';
	loader.setAttribute('role', 'status');
	loader.style.setProperty('--size', '24px');
	const text = document.createElement('span');
	text.className = 'sr-only';
	text.textContent = label;
	loader.append(text);
	for (let i = 0; i < 3; i++) {
		const cell = document.createElement('i');
		cell.style.setProperty('--i', String(i));
		cell.setAttribute('aria-hidden', 'true');
		loader.append(cell);
	}
	return loader;
}

async function refresh() {
	const detail = { waits: [] };
	document.dispatchEvent(new CustomEvent('tuita-feed-refresh', { detail }));
	await Promise.all([...detail.waits, new Promise((done) => setTimeout(done, MIN_BUSY_MS))]);
}

if (typeof document !== 'undefined' && !document.documentElement.dataset.pullBound) {
	document.documentElement.dataset.pullBound = '';
	mount('[data-island="feed"]', () => {
		let distance = 0;
		let busy = false;
		let startY = -1;
		let strip = null;

		const draw = () => {
			if (distance <= 0 && !busy) {
				strip?.remove();
				strip = null;
				return;
			}
			if (!strip) {
				strip = document.createElement('div');
				strip.dataset.pullIndicator = '';
				document.body.append(strip);
			}
			strip.style.height = `${busy ? BUSY_HEIGHT : distance}px`;
			const text = strings();
			const mode = busy || shouldRefresh(distance) ? 'busy' : 'note';
			if (strip.dataset.mode !== mode) {
				strip.dataset.mode = mode;
				if (mode === 'busy') {
					strip.replaceChildren(dots(text.loading ?? ''));
				} else {
					const note = document.createElement('span');
					note.className = 'note';
					note.textContent = text.pull_refresh ?? '';
					strip.replaceChildren(note);
				}
			}
		};

		const start = (event) => {
			startY = !busy && scrollY <= 0 && event.touches.length === 1 ? event.touches[0].clientY : -1;
		};
		const move = (event) => {
			if (startY < 0) return;
			if (scrollY > 0) {
				startY = -1;
				distance = 0;
				draw();
				return;
			}
			distance = pullDistance(startY, event.touches[0].clientY);
			draw();
		};
		const end = async () => {
			const go = startY >= 0 && shouldRefresh(distance);
			startY = -1;
			if (!go) {
				distance = 0;
				draw();
				return;
			}
			busy = true;
			draw();
			try {
				await refresh();
			} finally {
				busy = false;
				distance = 0;
				draw();
			}
		};

		const before = document.documentElement.style.overscrollBehaviorY;
		document.documentElement.style.overscrollBehaviorY = 'contain';
		addEventListener('touchstart', start, { passive: true });
		addEventListener('touchmove', move, { passive: true });
		addEventListener('touchend', end);
		addEventListener('touchcancel', end);
		return () => {
			removeEventListener('touchstart', start);
			removeEventListener('touchmove', move);
			removeEventListener('touchend', end);
			removeEventListener('touchcancel', end);
			document.documentElement.style.overscrollBehaviorY = before;
			strip?.remove();
		};
	});
}
