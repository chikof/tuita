// Passkeys (WebAuthn).
//
// `[data-island="passkey-login"]` (login page): `data-add="true|false"`, `data-next="<safe path or empty>"`,
// `data-domain="<instance domain>"`. Holds the handle `input` (autocomplete "username webauthn"), a
// `[data-passkey]` block (initially `hidden`) with `button[data-action="passkey"]`, and
// `p[data-passkey-error][hidden]`. Shown only when the browser and host allow passkeys.
//
// `[data-island="passkey-register"]` (settings): `button[data-action="register"]` and an optional
// `input[data-passkey-name]`; the root is hidden when passkeys are unavailable, and any `[data-passkey-unsupported]`
// element on the page is shown. A registered passkey toasts `passkey_added` and
// dispatches a bubbling `tuita-passkey` CustomEvent on the root (`detail` and `data`, a JSON string,
// both hold the PasskeyDto); other failures fire a `tuita-toast` on `document` with the message.
//
// Client strings (`#tuita-l10n`): passkey_already_here, passkey_unsupported, passkey_added.

/** The browser can parse the server's JSON options and serialise the credential. */
export const passkeysSupported = () =>
	typeof window !== 'undefined' &&
	'PublicKeyCredential' in window &&
	typeof PublicKeyCredential.parseCreationOptionsFromJSON === 'function' &&
	typeof PublicKeyCredential.parseRequestOptionsFromJSON === 'function';

/** WebAuthn refuses IP addresses as relying party ids. `domain` may carry a port. */
export function hostAllowsPasskeys(domain) {
	const host = domain.replace(/:\d+$/, '').replace(/^\[|\]$/g, '');
	return host !== '' && !/^\d{1,3}(\.\d{1,3}){3}$/.test(host) && !host.includes(':');
}

/** Whether passkeys can be offered inside the handle field. */
export async function conditionalSupported() {
	if (!passkeysSupported()) return false;
	try {
		return (await PublicKeyCredential.isConditionalMediationAvailable?.()) === true;
	} catch {
		return false;
	}
}

/** The person closed the prompt, or a newer prompt replaced it. Not an error to show. */
export const isCancel = (error) =>
	error instanceof DOMException && (error.name === 'NotAllowedError' || error.name === 'AbortError');

const strings = () => {
	try {
		return JSON.parse(document.getElementById('tuita-l10n')?.textContent ?? '{}');
	} catch {
		return {};
	}
};

/** A sentence for a failed browser prompt that is not a cancel. */
export function passkeyFailure(error) {
	if (error instanceof DOMException) {
		const known = strings();
		if (error.name === 'InvalidStateError' && known.passkey_already_here) return known.passkey_already_here;
		if ((error.name === 'NotSupportedError' || error.name === 'SecurityError') && known.passkey_unsupported) {
			return known.passkey_unsupported;
		}
	}
	return error instanceof Error ? error.message : String(error);
}

async function call(url, body) {
	const init = { method: 'POST', credentials: 'same-origin' };
	if (body !== undefined) {
		init.headers = { 'content-type': 'application/json' };
		init.body = JSON.stringify(body);
	}
	const response = await fetch(url, init);
	if (!response.ok) {
		let message = response.statusText;
		try {
			message = (await response.json()).error || message;
		} catch {
			// Keep the status text.
		}
		throw new Error(message);
	}
	return response.json();
}

/** Creates a passkey on this device and registers it. Resolves to `null` when the person cancelled. */
export async function registerPasskey(name) {
	const options = await call('/api/v1/passkeys/register/options');
	let credential;
	try {
		credential = await navigator.credentials.create({
			publicKey: PublicKeyCredential.parseCreationOptionsFromJSON(options.publicKey)
		});
	} catch (error) {
		if (isCancel(error)) return null;
		throw error;
	}
	if (!(credential instanceof PublicKeyCredential)) return null;
	return call('/api/v1/passkeys/register', { challengeId: options.challengeId, credential: credential.toJSON(), name });
}

/**
 * Signs in with a passkey; resolves to the user, or `null` on a cancel. With `conditional` the browser
 * offers passkeys in the field marked `autocomplete="username webauthn"` and the promise stays pending
 * until one is chosen or `signal` aborts; a conditional prompt that cannot start also resolves `null`.
 */
export async function signInWithPasskey({ add, conditional, signal }) {
	let options;
	try {
		options = await call('/api/v1/auth/passkey/options');
	} catch (error) {
		if (conditional) return null;
		throw error;
	}
	let credential;
	try {
		credential = await navigator.credentials.get({
			publicKey: PublicKeyCredential.parseRequestOptionsFromJSON(options.publicKey),
			mediation: conditional ? 'conditional' : undefined,
			signal
		});
	} catch (error) {
		if (conditional || isCancel(error)) return null;
		throw error;
	}
	if (!(credential instanceof PublicKeyCredential)) return null;
	return (await call('/api/v1/auth/passkey', { challengeId: options.challengeId, credential: credential.toJSON(), add })).user;
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

function wireLogin(root) {
	if (!passkeysSupported() || !hostAllowsPasskeys(root.dataset.domain || location.host)) return;
	const block = root.querySelector('[data-passkey]');
	const button = root.querySelector('button[data-action="passkey"]');
	const error = root.querySelector('[data-passkey-error]');
	if (!block || !button) return;
	const add = root.dataset.add === 'true';
	const done = () => location.assign(root.dataset.next || '/');
	block.hidden = false;

	let conditional = null;
	const stopConditional = () => {
		conditional?.abort();
		conditional = null;
	};

	conditionalSupported().then(async (ok) => {
		if (!ok || !root.isConnected) return;
		conditional = new AbortController();
		try {
			if (await signInWithPasskey({ add, conditional: true, signal: conditional.signal })) done();
		} catch {
			// The button path reports failures; a background prompt stays quiet.
		}
	});

	button.addEventListener('click', async () => {
		stopConditional();
		if (error) error.hidden = true;
		button.disabled = true;
		button.setAttribute('aria-busy', 'true');
		try {
			if (await signInWithPasskey({ add })) done();
		} catch (failure) {
			if (error && !isCancel(failure)) {
				error.textContent = passkeyFailure(failure);
				error.hidden = false;
			}
		} finally {
			button.disabled = false;
			button.removeAttribute('aria-busy');
		}
	});

	root.querySelector('form')?.addEventListener('submit', stopConditional, true);
	return stopConditional;
}

function wireRegister(root) {
	if (!passkeysSupported() || !hostAllowsPasskeys(location.host)) {
		root.hidden = true;
		for (const note of document.querySelectorAll('[data-passkey-unsupported]')) note.hidden = false;
		return;
	}
	const button = root.querySelector('button[data-action="register"]');
	const name = root.querySelector('input[data-passkey-name]');
	button?.addEventListener('click', async () => {
		button.disabled = true;
		button.setAttribute('aria-busy', 'true');
		try {
			const dto = await registerPasskey(name?.value.trim() ?? '');
			if (!dto) return;
			const added = strings().passkey_added;
			if (added) document.dispatchEvent(new CustomEvent('tuita-toast', { detail: added.replace('{name}', dto.name) }));
			const event = new CustomEvent('tuita-passkey', { bubbles: true, detail: dto });
			event.data = JSON.stringify(dto);
			root.dispatchEvent(event);
		} catch (failure) {
			if (!isCancel(failure)) document.dispatchEvent(new CustomEvent('tuita-toast', { detail: passkeyFailure(failure) }));
		} finally {
			button.disabled = false;
			button.removeAttribute('aria-busy');
		}
	});
}

if (typeof document !== 'undefined' && !document.documentElement.dataset.passkeysBound) {
	document.documentElement.dataset.passkeysBound = '';
	mount('[data-island="passkey-login"]', wireLogin);
	mount('[data-island="passkey-register"]', wireRegister);
}
