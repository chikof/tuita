// Known-answer vectors for assets/js/webauthn.js.
const assert = (ok, what) => {
	if (!ok) throw new Error(`assertion failed: ${what}`);
};
const eq = (actual, expected, what) => {
	if (JSON.stringify(actual) !== JSON.stringify(expected)) {
		throw new Error(`${what}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
	}
};

const dom = (name, message = '') => new DOMException(message, name);
const ALREADY = 'This device already has a passkey for this account.';
const UNSUPPORTED = 'This browser can’t use passkeys on this address.';

export async function run(mod) {
	const page = document.getElementById('tuita-l10n');
	const parent = page?.parentNode;
	page?.remove();
	try {
		await check(mod);
	} finally {
		if (page) parent.append(page);
	}
}

async function check(mod) {
	assert(mod.isCancel(dom('NotAllowedError')) && mod.isCancel(dom('AbortError')), 'cancels');
	assert(!mod.isCancel(dom('InvalidStateError')), 'invalid state is real');
	assert(!mod.isCancel(new Error('NotAllowedError')), 'plain Error is real');
	assert(!mod.isCancel(null), 'null is not a cancel');

	eq(mod.passkeyFailure(new Error('boom')), 'boom', 'message fallback');
	eq(mod.passkeyFailure('plain'), 'plain', 'string fallback');
	eq(mod.passkeyFailure(dom('InvalidStateError', 'raw')), 'raw', 'no l10n block falls back to the message');

	const block = document.createElement('script');
	block.type = 'application/json';
	block.id = 'tuita-l10n';
	block.textContent = JSON.stringify({ passkey_already_here: ALREADY, passkey_unsupported: UNSUPPORTED });
	document.head.append(block);
	try {
		eq(mod.passkeyFailure(dom('InvalidStateError')), ALREADY, 'already here');
		eq(mod.passkeyFailure(dom('SecurityError')), UNSUPPORTED, 'security error');
		eq(mod.passkeyFailure(dom('NotSupportedError')), UNSUPPORTED, 'not supported');
		eq(mod.passkeyFailure(dom('DataError', 'other')), 'other', 'other DOMException keeps its message');
	} finally {
		block.remove();
	}

	for (const ok of ['tuita.example', 'localhost:8081']) assert(mod.hostAllowsPasskeys(ok), `allows ${ok}`);
	for (const bad of ['127.0.0.1', '127.0.0.1:8080', '[::1]:8080', '']) assert(!mod.hostAllowsPasskeys(bad), `refuses ${bad}`);

	assert(typeof mod.passkeysSupported() === 'boolean', 'supported is a boolean');
	assert(typeof (await mod.conditionalSupported()) === 'boolean', 'conditional is a boolean');
}
