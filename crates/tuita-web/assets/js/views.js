// Reports posts as viewed: half of an `article[data-post]` on screen for 1 s, batched to `POST /api/v1/views`.
// Only signed-in members report (the stream island's root is rendered for them).

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

/** `POST /views` takes at most 50 ids. */
export const BATCH = 50;
export const FLUSH_MS = 5000;
/** A post counts once half of it was on screen for this long. */
export const DWELL_MS = 1000;

/** Collects viewed post ids, once each, and sends them in batches of at most `size` every `every` ms. */
export function createBatcher(send, later, size = BATCH, every = FLUSH_MS) {
	const sent = new Set();
	const queue = [];
	let cancel;

	function flush() {
		cancel?.();
		cancel = undefined;
		while (queue.length) void Promise.resolve(send(queue.splice(0, size))).catch(() => {});
	}

	return {
		mark(id) {
			if (sent.has(id)) return;
			sent.add(id);
			queue.push(id);
			if (queue.length >= size) flush();
			else cancel ??= later(flush, every);
		},
		flush
	};
}

if (typeof document !== 'undefined' && 'IntersectionObserver' in window && !document.documentElement.dataset.viewsBound) {
	document.documentElement.dataset.viewsBound = '';
	const batcher = createBatcher(
		(ids) =>
			fetch('/api/v1/views', {
				method: 'POST',
				credentials: 'same-origin',
				headers: { 'content-type': 'application/json' },
				body: JSON.stringify({ ids })
			}),
		(run, ms) => {
			const timer = setTimeout(run, ms);
			return () => clearTimeout(timer);
		}
	);
	const timers = new Map();
	// The gallery fakes a session; its sample posts do not exist.
	const reporting = () => document.querySelector('[data-island="stream"]') !== null && !location.pathname.startsWith('/design');
	const watcher = new IntersectionObserver(
		(entries) => {
			for (const entry of entries) {
				const el = entry.target;
				const timer = timers.get(el);
				if (entry.isIntersecting && timer === undefined) {
					timers.set(
						el,
						setTimeout(() => {
							timers.delete(el);
							watcher.unobserve(el);
							if (el.dataset.post && reporting()) batcher.mark(el.dataset.post);
						}, DWELL_MS)
					);
				} else if (!entry.isIntersecting && timer !== undefined) {
					clearTimeout(timer);
					timers.delete(el);
				}
			}
		},
		{ threshold: 0.5 }
	);
	mount('article[data-post]', (el) => {
		watcher.observe(el);
		return () => {
			clearTimeout(timers.get(el));
			timers.delete(el);
			watcher.unobserve(el);
		};
	});
	document.addEventListener('visibilitychange', () => {
		if (document.visibilityState === 'hidden') batcher.flush();
	});
}
