// Vectors for conversationState in assets/js/e2ee.js.
const eq = (actual, expected, what) => {
	if (JSON.stringify(actual) !== JSON.stringify(expected)) {
		throw new Error(`${what}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
	}
};
const match = (actual, subset, what) => {
	for (const [key, value] of Object.entries(subset)) {
		if (value && typeof value === 'object') match(actual[key], value, `${what}.${key}`);
		else eq(actual[key], value, `${what}.${key}`);
	}
};

const OLD = 'old-key';
const NEW = 'new-key';
const STATUSES = ['loading', 'error', 'none', 'locked', 'ready'];

export async function run(mod) {
	const input = (over) => ({ mode: 'sealed', blockedReason: null, remote: false, peerKey: NEW, keys: 'ready', pinned: NEW, ...over });
	const state = (over) => mod.conversationState(input(over));
	const plain = { mode: 'plain', peerKey: null, pinned: null };

	// plain
	eq(state({ ...plain, remote: true }), { note: 'remote', pin: null, send: 'plain', block: null }, 'remote person');
	for (const keys of STATUSES) eq(state({ ...plain, remote: true, keys }).send, 'plain', `plain needs no keys (${keys})`);
	eq(state(plain), { note: 'unencrypted', pin: null, send: 'plain', block: null }, 'local person without keys');
	eq(state({ ...plain, pinned: OLD }), { note: 'unencrypted', pin: null, send: null, block: { kind: 'downgrade' } }, 'no plaintext after a pin');
	eq(state({ ...plain, remote: true, pinned: OLD }), { note: 'remote', pin: null, send: null, block: { kind: 'downgrade' } }, 'no plaintext to a pinned remote');

	// unavailable
	match(state({ mode: 'unavailable', peerKey: null, pinned: null }), { send: null, block: { kind: 'peer-off' }, note: null }, 'peer without a key');
	match(state({ mode: 'unavailable', keys: 'none' }), { send: null, block: { kind: 'setup' } }, 'viewer without a key');
	match(state({ mode: 'unavailable', keys: 'ready' }), { send: null, block: { kind: 'self-off' } }, 'viewer thinks it has a key');

	// sealed
	for (const [keys, kind] of [['none', 'setup'], ['locked', 'unlock'], ['loading', 'loading'], ['error', 'error']]) {
		match(state({ keys, pinned: null }), { send: null, block: { kind }, pin: null }, `the ${keys} prompt`);
	}
	eq(state({ pinned: null }), { note: null, pin: 'new', send: 'sealed', block: null }, 'a new key is pinned');
	eq(state({}), { note: null, pin: 'same', send: 'sealed', block: null }, 'a matching pin goes on');
	eq(state({ pinned: OLD }), { note: null, pin: 'changed', send: null, block: { kind: 'changed' } }, 'a changed key blocks');
	match(state({ peerKey: null, pinned: null }), { send: null, block: { kind: 'peer-off' } }, 'sealed without a peer key');

	// server refusal
	for (const mode of ['sealed', 'plain', 'unavailable']) {
		match(
			state({ mode, blockedReason: 'You blocked them.', remote: true }),
			{ send: null, block: { kind: 'server', reason: 'You blocked them.' } },
			`the server refusal wins (${mode})`
		);
	}
	eq(state({ mode: 'plain', remote: true, blockedReason: 'No.' }).note, 'remote', 'the note of a plain conversation stays');
}
