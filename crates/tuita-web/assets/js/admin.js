// The admin console's small browser parts. The browser-only bits: copying a
// value, the infinite lists, the tab strip and the emote form.
//
// Contracts (all `mount`ed or bound on `document`, so they work on markup added later):
//
// `tuita-copy` (event on `document`): copies `detail.text` to the clipboard and toasts the outcome.
//   detail  { text, ok, fail, v }  `ok` and `fail` name `#tuita-l10n` strings; `{<v>}` in the string is
//           replaced by the text (`v: 'code'` fills `{code}`).
//
// `tuita-reload` (event on `window`): reloads the page, and when `detail` is a non-empty string toasts
//   it once the new page is up (the toast is kept in `sessionStorage` `tuita:toast-next`).
//
// `button[data-auto-more]`: clicked by the island when it comes within 600px of the viewport, so a list
//   that grows through a button ("Show more") loads like an infinite scroll. The button stays a
//   working button without script.
//
// `[data-c="admin-tabs"]`: the current tab (`[aria-current="page"]`) is scrolled into view in the strip.
//
// `[data-island="emote-form"]`: wraps the emote upload `<form data-island="multipart-form">` (which
//   posts the form, see `upload.js`). It checks the shortcode and the picture as the fields change,
//   shows a preview, enables the submit button only for a valid pair, takes a file dropped on the zone
//   (`tuita-files` from the `ui::dropzone` around the form) and, once the upload went through and the page is about to
//   reload, queues the "Added :code:." toast for the next page.
//   attributes  data-invalid, data-file-type, data-file-size   error texts
//               data-hint, data-choose, data-change            the file note and the button's two labels
//               data-added                                     toast text with `{shortcode}`
//   children    input[name="shortcode"], input[type="file"], button[data-role="pick"],
//               [data-role="preview"], [data-role="file-note"], button[type="submit"]

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

export const MAX_EMOTE_BYTES = 256 * 1024;
export const EMOTE_TYPES = ['image/png', 'image/jpeg', 'image/gif', 'image/webp'];
const SHORTCODE = /^[A-Za-z0-9_]{2,32}$/;
const TOAST_KEY = 'tuita:toast-next';

/** Whether `code` (trimmed) is a shortcode the server accepts. */
export const validShortcode = (code) => SHORTCODE.test(code.trim());

/** Why `file` cannot be an emote: `'type'`, `'size'`, or `null` when it can. */
export function fileProblem(file) {
	if (!EMOTE_TYPES.includes(file.type)) return 'type';
	if (file.size > MAX_EMOTE_BYTES) return 'size';
	return null;
}

/** `template` with `{name}` replaced by `value`. */
export const fill = (template, name, value) => template.replaceAll(`{${name}}`, value);

function strings() {
	try {
		return JSON.parse(document.getElementById('tuita-l10n')?.textContent ?? '{}');
	} catch {
		return {};
	}
}

function toast(message) {
	if (message) document.dispatchEvent(new CustomEvent('tuita-toast', { bubbles: true, detail: message }));
}

async function copy(text) {
	try {
		await navigator.clipboard.writeText(text);
		return true;
	} catch {
		return false;
	}
}

if (typeof document !== 'undefined' && !document.documentElement.dataset.adminBound) {
	document.documentElement.dataset.adminBound = '';

	document.addEventListener('tuita-copy', async (event) => {
		const { text = '', ok = '', fail = '', v = 'text' } = event.detail ?? {};
		const table = strings();
		toast(fill((await copy(text)) ? (table[ok] ?? ok) : (table[fail] ?? fail), v, text));
	});

	window.addEventListener('tuita-reload', (event) => {
		try {
			if (typeof event.detail === 'string' && event.detail) sessionStorage.setItem(TOAST_KEY, event.detail);
		} catch {
			// Private mode: the page still reloads, without the toast.
		}
		location.reload();
	});

	try {
		const pending = sessionStorage.getItem(TOAST_KEY);
		if (pending) {
			sessionStorage.removeItem(TOAST_KEY);
			toast(pending);
		}
	} catch {
		// No storage, no queued toast.
	}

	mount('button[data-auto-more]', (button) => {
		if (typeof IntersectionObserver === 'undefined') return;
		const watcher = new IntersectionObserver(
			(entries) => {
				if (entries.some((entry) => entry.isIntersecting) && !button.disabled) button.click();
			},
			{ rootMargin: '600px 0px' }
		);
		watcher.observe(button);
		return () => watcher.disconnect();
	});

	mount('[data-c="admin-tabs"]', (nav) => {
		nav.querySelector('[aria-current="page"]')?.scrollIntoView({ inline: 'center', block: 'nearest' });
	});

	mount('[data-island="emote-form"]', mountEmoteForm);
}

function mountEmoteForm(root) {
	const form = root.querySelector('form');
	const code = root.querySelector('input[name="shortcode"]');
	const input = root.querySelector('input[type="file"]');
	const pick = root.querySelector('[data-role="pick"]');
	const preview = root.querySelector('[data-role="preview"]');
	const note = root.querySelector('[data-role="file-note"]');
	const submit = root.querySelector('button[type="submit"]');
	if (!form || !code || !input || !pick || !preview || !note || !submit) return;
	const text = root.dataset;
	const field = code.closest('.field');
	let problem = null;
	let url = null;

	const showCodeError = (message) => {
		field?.querySelector('.message.error')?.remove();
		if (!message) {
			code.removeAttribute('aria-invalid');
			field?.classList.remove('invalid');
			return;
		}
		code.setAttribute('aria-invalid', 'true');
		field?.classList.add('invalid');
		const line = document.createElement('p');
		line.className = 'message error';
		line.textContent = message;
		field?.append(line);
	};

	const refresh = () => {
		const typed = code.value.trim();
		showCodeError(typed !== '' && !validShortcode(typed) ? text.invalid : '');
		const file = input.files?.[0];
		submit.disabled = !(validShortcode(typed) && file && !problem);
		const img = preview.querySelector('img');
		if (img) img.alt = typed ? `:${typed}:` : '';
	};

	const choose = (file) => {
		if (!file) return;
		problem = fileProblem(file);
		if (url) URL.revokeObjectURL(url);
		url = URL.createObjectURL(file);
		preview.replaceChildren(Object.assign(document.createElement('img'), { src: url, decoding: 'async' }));
		preview.removeAttribute('aria-hidden');
		note.textContent = problem === 'type' ? text.fileType : problem === 'size' ? text.fileSize : file.name;
		note.classList.toggle('bad', problem !== null);
		if (problem) note.setAttribute('role', 'alert');
		else note.removeAttribute('role');
		pick.querySelector('.label').textContent = text.change;
		refresh();
	};

	const onfile = () => choose(input.files?.[0]);
	const onpick = () => input.click();
	const ondrop = (event) => {
		const [file] = event.detail?.files ?? [];
		if (!file) return;
		const transfer = new DataTransfer();
		transfer.items.add(file);
		input.files = transfer.files;
		choose(file);
	};
	const onleave = () => {
		if (form.getAttribute('aria-busy') !== 'true') return;
		const shortcode = code.value.trim();
		try {
			sessionStorage.setItem(TOAST_KEY, fill(text.added ?? '', 'shortcode', shortcode));
		} catch {
			// Private mode: no toast after the reload.
		}
	};

	code.addEventListener('input', refresh);
	input.addEventListener('change', onfile);
	pick.addEventListener('click', onpick);
	root.addEventListener('tuita-files', ondrop);
	window.addEventListener('pagehide', onleave);
	refresh();
	return () => {
		code.removeEventListener('input', refresh);
		input.removeEventListener('change', onfile);
		pick.removeEventListener('click', onpick);
		root.removeEventListener('tuita-files', ondrop);
		window.removeEventListener('pagehide', onleave);
		if (url) URL.revokeObjectURL(url);
	};
}
