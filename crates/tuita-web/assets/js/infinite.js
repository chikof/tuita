// Infinite lists. The server ends a page with `<a data-more href="?before=…">`; near the viewport
// this fetches that href as a fragment and swaps the link for the next items (and next link).
// Without JS, or after a failed fetch, the link keeps working as a plain "more" link.

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

export async function loadMore(link) {
	if (link.getAttribute('aria-busy') === 'true') return false;
	link.setAttribute('aria-busy', 'true');
	try {
		const response = await fetch(link.href, {
			credentials: 'same-origin',
			headers: { 'x-tuita-fragment': '1', accept: 'text/html' }
		});
		if (!response.ok) throw new Error(String(response.status));
		// A template's content is inert: no script in the fragment runs.
		const page = document.createElement('template');
		page.innerHTML = await response.text();
		// A sentinel inside a `data-more-host` item (feed lists) is replaced whole, so the items do not nest in it.
		(link.closest('[data-more-host]') ?? link).replaceWith(page.content);
		return true;
	} catch {
		link.removeAttribute('aria-busy');
		link.dataset.failed = '';
		return false;
	}
}

if (typeof document !== 'undefined' && 'IntersectionObserver' in window && !document.documentElement.dataset.infiniteBound) {
	document.documentElement.dataset.infiniteBound = '';
	// Margin: start loading a screen ahead.
	const watcher = new IntersectionObserver(
		(entries) => {
			for (const entry of entries) {
				if (!entry.isIntersecting) continue;
				const link = entry.target;
				if (!('failed' in link.dataset)) void loadMore(link);
			}
		},
		{ rootMargin: '600px 0px' }
	);
	// A click before the observer fires loads in place; after a failure it follows the link.
	document.addEventListener('click', (event) => {
		const link = event.target instanceof Element ? event.target.closest('a[data-more]') : null;
		if (!link || 'failed' in link.dataset || event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
		event.preventDefault();
		void loadMore(link);
	});
	mount('a[data-more]', (link) => {
		watcher.observe(link);
		return () => watcher.unobserve(link);
	});
}
