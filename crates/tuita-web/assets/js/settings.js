// Settings page helpers: region names, the copy button and the token box. Nothing here talks to the server.
//
// `[data-region="DE"]`: its text becomes the name of that region in the page language (`Intl.DisplayNames`),
//   falling back to the code.
// `[data-copy="<selector>"]` (a button): a click copies the value of the element the selector names and toasts
//   `token_copied` or `token_copy_failed`.
// `textarea[data-select]`: selects its text on focus.
// the token dialog (a `dialog` holding `[data-c="token-secret"]`): its box is blanked when the dialog closes, so a secret does not linger in the DOM.
//
// Client strings (`#tuita-l10n`): token_copied, token_copy_failed.

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

/** Region name of a two-letter country code, or the code when the runtime cannot name it. */
export function regionName(code, language) {
	try {
		return new Intl.DisplayNames(language, { type: 'region' }).of(code.toUpperCase()) ?? code;
	} catch {
		return code;
	}
}

const strings = () => {
	try {
		return JSON.parse(document.getElementById('tuita-l10n')?.textContent ?? '{}');
	} catch {
		return {};
	}
};

const say = (message) => document.dispatchEvent(new CustomEvent('tuita-toast', { detail: message }));

/** Copies `text` to the clipboard; false when the browser refuses. */
export async function copyText(text) {
	try {
		await navigator.clipboard.writeText(text);
		return true;
	} catch {
		try {
			const box = document.createElement('textarea');
			box.value = text;
			box.style.position = 'fixed';
			box.style.opacity = '0';
			document.body.append(box);
			box.select();
			const done = document.execCommand('copy');
			box.remove();
			return done;
		} catch {
			return false;
		}
	}
}

if (typeof document !== 'undefined' && !document.documentElement.dataset.settingsBound) {
	document.documentElement.dataset.settingsBound = '';

	mount('[data-region]', (el) => {
		el.textContent = regionName(el.dataset.region, document.documentElement.lang || undefined);
	});

	document.addEventListener('click', async (event) => {
		const button = event.target instanceof Element ? event.target.closest('[data-copy]') : null;
		if (!button) return;
		const source = document.querySelector(button.dataset.copy);
		if (!source) return;
		const table = strings();
		say((await copyText(source.value ?? source.textContent ?? '')) ? table.token_copied : table.token_copy_failed);
	});

	document.addEventListener('focusin', (event) => {
		if (event.target instanceof HTMLTextAreaElement && event.target.matches('[data-select]')) event.target.select();
	});

	document.addEventListener(
		'close',
		(event) => {
			if (event.target instanceof HTMLDialogElement && event.target.querySelector('[data-c="token-secret"]')) {
				const box = event.target.querySelector('textarea');
				if (box) box.value = '';
			}
		},
		true
	);
}
