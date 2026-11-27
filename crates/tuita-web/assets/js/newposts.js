// The home feed: the "show new posts" pill, a post the member just wrote, and starting the timeline over.
//
// Root: `[data-island="feed"][data-feed="foryou|following"][data-stream="badges"]` (Home) holding a pill slot
// `[data-slot="pill"]` and the list region `[data-feed-body][data-since="<entry id of the first post>"]`.
//
// - Every 30 s (and on a `badges` stream event, delivered as `tuita-stream` on the root) it asks
//   `/_tuita/new-posts?feed=&since=` for the pill: markup with `.pill`, or an empty answer. A click on `.pill`
//   starts over.
// - Starting over (`tuita-feed-refresh` on `document`, from the pill and from `pull.js`, which collects the
//   promise from `detail.waits`) fetches the page itself as a fragment (`x-tuita-fragment: 1`), which answers one
//   `[data-feed-body]` element, and swaps it in.
// - `tuita-posted` (from the composer): a finished post that is not a reply is fetched from `/_tuita/post/<id>`
//   and put at the top of the list.
// Nothing inside the swapped markup is hydrated by Topcoat; `post.js` drives the cards.

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

export const POLL_MS = 30_000;
/** `since` of an empty timeline: every post is newer. */
export const NIL = '00000000-0000-0000-0000-000000000000';

const enc = encodeURIComponent;

/** Where the pill comes from. */
export const pillUrl = (feed, since = NIL) => `/_tuita/new-posts?feed=${enc(feed)}&since=${enc(since)}`;

/** The first page of a feed: the home page itself. */
export const feedUrl = (path, feed) => `${path}?feed=${enc(feed)}`;

/** HTML from `url` as a fragment, or `null` when the answer is empty or not OK. */
async function fetchFragment(url) {
	const response = await fetch(url, {
		credentials: 'same-origin',
		headers: { 'x-tuita-fragment': '1', accept: 'text/html' }
	});
	if (!response.ok || response.status === 204) return null;
	// A template's content is inert: no script in the fragment runs.
	const holder = document.createElement('template');
	holder.innerHTML = await response.text();
	return holder.content;
}

if (typeof document !== 'undefined' && !document.documentElement.dataset.newpostsBound) {
	document.documentElement.dataset.newpostsBound = '';
	mount('[data-island="feed"]', (root) => {
		const feed = root.dataset.feed ?? 'foryou';
		const slot = () => root.querySelector('[data-slot="pill"]');
		const body = () => root.querySelector('[data-feed-body]');
		let refreshing = null;

		async function check() {
			if (document.hidden) return;
			try {
				const found = await fetchFragment(pillUrl(feed, body()?.dataset.since || NIL));
				const target = slot();
				if (target) found ? target.replaceChildren(found) : target.replaceChildren();
			} catch {
				// The next poll retries.
			}
		}

		function refresh() {
			refreshing ??= (async () => {
				try {
					const found = await fetchFragment(feedUrl(location.pathname, feed));
					const fresh = found?.querySelector('[data-feed-body]');
					if (fresh) {
						body()?.replaceWith(fresh);
						slot()?.replaceChildren();
					}
				} catch {
					// The old list stays.
				} finally {
					refreshing = null;
				}
			})();
			return refreshing;
		}

		async function posted(detail) {
			if (!detail?.id || detail.replyTo) return;
			const list = body()?.querySelector('[role="list"]');
			if (!list) return;
			try {
				const card = (await fetchFragment(`/_tuita/post/${enc(detail.id)}`))?.querySelector('article');
				if (!card) return;
				for (const old of list.querySelectorAll(`article[data-post-id="${CSS.escape(String(detail.id))}"]`)) {
					(old.closest('[role="listitem"]') ?? old).remove();
				}
				const row = document.createElement('div');
				row.setAttribute('role', 'listitem');
				row.append(card);
				list.prepend(row);
				body()?.querySelector('[data-empty]')?.remove();
			} catch {
				// The post shows after the next load.
			}
		}

		const timer = setInterval(() => void check(), POLL_MS);
		const onStream = () => void check();
		const onRefresh = (event) => {
			const promise = refresh();
			event.detail?.waits?.push(promise);
		};
		const onPosted = (event) => void posted(event.detail);
		const onClick = (event) => {
			if (!(event.target instanceof Element) || !event.target.closest('[data-c="new-posts-pill"] .pill')) return;
			void refresh();
			scrollTo({ top: 0, behavior: 'smooth' });
		};
		root.addEventListener('tuita-stream', onStream);
		root.addEventListener('click', onClick);
		document.addEventListener('tuita-feed-refresh', onRefresh);
		document.addEventListener('tuita-posted', onPosted);
		return () => {
			clearInterval(timer);
			root.removeEventListener('tuita-stream', onStream);
			root.removeEventListener('click', onClick);
			document.removeEventListener('tuita-feed-refresh', onRefresh);
			document.removeEventListener('tuita-posted', onPosted);
		};
	});
}
