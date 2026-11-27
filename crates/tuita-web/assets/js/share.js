// Share menu actions: copy link/text, native share sheet, media download.
// Delegated on `button[data-act]`; the post island handles the other `data-act` values.

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

const toast = (detail) => document.dispatchEvent(new CustomEvent('tuita-toast', { detail }));
const clientString = (key) => {
	try {
		return JSON.parse(document.getElementById('tuita-l10n')?.textContent ?? '{}')[key] ?? key;
	} catch {
		return key;
	}
};

/** The native share sheet exists (most phones, some desktops). */
export const canShareNatively = () => typeof navigator !== 'undefined' && typeof navigator.share === 'function';

/** Copies `text` to the clipboard; false when the browser refuses (no permission, no secure context). */
export async function copyText(text) {
	try {
		await navigator.clipboard.writeText(text);
		return true;
	} catch {
		return false;
	}
}

/** Copies a link and toasts the outcome; a refused clipboard toasts the link itself. */
async function copyLink(url) {
	toast((await copyText(url)) ? clientString('link_copied') : url);
}

/**
 * Shares a link: the native sheet when `native` and available, else the clipboard, else the link in a
 * toast. A dismissed sheet is not an error. `post` is `{url, text}` with an absolute `url`.
 */
export async function sharePost(post, native) {
	if (native && canShareNatively()) {
		try {
			await navigator.share({ url: post.url, text: post.text });
			return;
		} catch (error) {
			if (error instanceof DOMException && error.name === 'AbortError') return;
		}
	}
	await copyLink(post.url);
}

/** `{id}.{ext}`, the extension taken from the media URL (`/media/{id}.{ext}`). */
export function mediaFileName(media) {
	const path = media.url.split(/[?#]/)[0];
	const ext = /\.([a-z0-9]{1,5})$/i.exec(path)?.[1]?.toLowerCase();
	return `${media.id}.${ext ?? (media.kind === 'video' ? 'mp4' : 'jpg')}`;
}

function save(href, name, tab) {
	const a = document.createElement('a');
	a.href = href;
	a.download = name;
	if (tab) {
		a.target = '_blank';
		a.rel = 'noopener';
	}
	document.body.append(a);
	a.click();
	a.remove();
}

/**
 * Saves a media file. Fetches it as a blob so the `download` name also applies to cross origin
 * storage; when the fetch is refused (CORS) the plain link opens in a new tab.
 */
export async function downloadMedia(media) {
	const name = mediaFileName(media);
	try {
		const response = await fetch(media.url);
		if (!response.ok) throw new Error(String(response.status));
		const href = URL.createObjectURL(await response.blob());
		save(href, name, false);
		setTimeout(() => URL.revokeObjectURL(href), 10_000);
	} catch {
		save(media.url, name, true);
	}
}

if (typeof document !== 'undefined' && !document.documentElement.dataset.shareBound) {
	document.documentElement.dataset.shareBound = '';
	document.addEventListener('click', (event) => {
		const button = event.target instanceof Element ? event.target.closest('button[data-act]') : null;
		if (!button) return;
		const { act, url, text, mediaId, kind } = button.dataset;
		switch (act) {
			case 'copy-link':
				void copyLink(location.origin + url);
				break;
			case 'copy-text':
				void copyText(text ?? '').then((ok) => toast(ok ? clientString('text_copied') : (text ?? '')));
				break;
			case 'share':
				void sharePost({ url: location.origin + url, text: text ?? '' }, true);
				break;
			case 'download':
				void downloadMedia({ id: mediaId, kind, url });
				break;
		}
	});
	mount('[data-needs-share]', (el) => {
		if (canShareNatively()) el.hidden = false;
	});
}
