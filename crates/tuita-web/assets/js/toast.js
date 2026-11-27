// Toasts. `document` events `tuita-toast` (detail: a message string, or `{message, label, href}`) and
// `[data-toast]` elements (`<output hidden data-toast data-kind data-label data-href>message</output>`,
// the server's flash) become `.toast` items inside `#toaster`.

export const TOAST_MS = 5000;

const CLOSE_ICON =
	'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="16" height="16" fill="currentColor" shape-rendering="crispEdges" aria-hidden="true" focusable="false"><path d="M7 19H5V17H7V19ZM19 19H17V17H19V19ZM9 15V17H7V15H9ZM17 17H15V15H17V17ZM11 15H9V13H11V15ZM15 15H13V13H15V15ZM13 13H11V11H13V13ZM11 11H9V9H11V11ZM15 11H13V9H15V11ZM9 9H7V7H9V9ZM17 9H15V7H17V9ZM7 7H5V5H7V7ZM19 7H17V5H19V7Z"/></svg>';

function dismissLabel() {
	try {
		const strings = JSON.parse(document.getElementById('tuita-l10n')?.textContent ?? '{}');
		return strings.dismiss ?? 'dismiss';
	} catch {
		return 'dismiss';
	}
}

function make(tag, className, attrs = {}) {
	const el = document.createElement(tag);
	if (className) el.className = className;
	for (const [name, value] of Object.entries(attrs)) el.setAttribute(name, value);
	return el;
}

/** Shows a toast that dismisses itself after 5 s. The action button follows `href` and closes the toast. */
export function show(input) {
	const { message, label, href, kind } = typeof input === 'string' ? { message: input } : (input ?? {});
	const host = document.getElementById('toaster');
	if (!host || !message) return null;

	const toast = make('div', 'toast', { role: 'status' });
	if (kind) toast.dataset.kind = kind;
	toast.style.setProperty('--ms', `${TOAST_MS}ms`);
	const dismiss = () => toast.remove();

	const body = make('p');
	body.textContent = message;
	toast.append(body);

	if (label && href) {
		const action = make('button', 'btn outline sm wash', { type: 'button', 'data-c': 'button' });
		const text = make('span', 'label');
		text.textContent = label;
		action.append(text);
		action.addEventListener('click', () => {
			dismiss();
			location.assign(href);
		});
		toast.append(action);
	}

	const close = make('button', 'icon-button ghost sm wash', { type: 'button', 'data-c': 'icon-button', 'aria-label': dismissLabel() });
	close.innerHTML = CLOSE_ICON;
	close.addEventListener('click', dismiss);
	toast.append(close, make('i', 'drain', { 'aria-hidden': 'true' }));

	host.append(toast);
	setTimeout(dismiss, TOAST_MS);
	return toast;
}

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

if (typeof document !== 'undefined' && !document.documentElement.dataset.toastBound) {
	document.documentElement.dataset.toastBound = '';
	// A toast raised from a Topcoat handler (`raw!`) arrives as a runtime surrogate (a String object): unwrap it.
	document.addEventListener('tuita-toast', (event) => {
		const detail = event.detail;
		show(typeof detail?.dehydrate === 'function' ? detail.dehydrate() : detail);
	});
	mount('[data-toast]', (el) => {
		show({ message: el.textContent.trim(), label: el.dataset.label, href: el.dataset.href, kind: el.dataset.kind });
		el.remove();
	});
}
