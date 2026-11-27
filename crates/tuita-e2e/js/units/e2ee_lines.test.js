// Vectors for assets/js/e2ee.js: line opening, placeholders and key error text.
const eq = (actual, expected, what) => {
	if (JSON.stringify(actual) !== JSON.stringify(expected)) {
		throw new Error(`${what}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
	}
};

const ADA = 'aaaaaaaa-0000-7000-8000-000000000001';
const BOB = 'bbbbbbbb-0000-7000-8000-000000000002';
const PASSPHRASE = 'correct horse battery staple';

export async function run(mod) {
	const { createIdentity, seal, openLine, placeholderFor, E2eeError, keyErrorText, blockText, placeholderText } = mod;
	const [ada, bob, eve] = await Promise.all([createIdentity(PASSPHRASE), createIdentity(PASSPHRASE), createIdentity(PASSPHRASE)]);
	const sealed = await seal({ text: 'meet at noon', mine: ada, theirs: bob.publicKey, senderId: ADA, recipientId: BOB });
	const asBob = (over) => ({ sealed, mine: bob, theirs: ada.publicKey, sentByMe: false, senderId: ADA, recipientId: BOB, ...over });

	eq(await openLine(asBob({})), { text: 'meet at noon' }, 'opens under the pinned key');
	const cases = [
		['an unpinned peer', { theirs: null }, 'peer-key'],
		['a changed peer key', { theirs: eve.publicKey }, 'peer-key'],
		['a key the viewer no longer has', { mine: eve }, 'own-key'],
		['a flipped direction', { sentByMe: true }, 'own-key'],
		['another conversation', { recipientId: ADA }, 'unreadable'],
		['a mangled ciphertext', { sealed: sealed.slice(0, -4) + 'AAAA' }, 'unreadable'],
		['text that is no ciphertext', { sealed: 'hello there' }, 'unreadable']
	];
	for (const [name, over, placeholder] of cases) {
		const line = await openLine(asBob(over));
		eq(line, { placeholder }, `a placeholder for ${name}`);
		if (JSON.stringify(line).includes(sealed.slice(3, 40))) throw new Error(`the ciphertext leaks for ${name}`);
	}
	eq(
		['format', 'passphrase', 'key-mismatch', 'own-key', 'decrypt', 'unsupported'].map(placeholderFor),
		['unreadable', 'unreadable', 'peer-key', 'own-key', 'unreadable', 'unsupported'],
		'every reason has a placeholder'
	);

	// The words around it.
	const table = {
		e2ee_line_peer_key: 'untrusted',
		e2ee_block_peer_off: 'Off: {handle}',
		e2ee_block_downgrade: 'Used before: {handle}',
		e2ee_block_changed: 'Trust {handle}',
		e2ee_block_self_off: 'Turn on',
		e2ee_block_unlock: 'Unlock first',
		e2ee_block_error: 'Broken',
		e2ee_unlock_wrong: 'Wrong passphrase',
		e2ee_unsupported: 'No crypto',
		e2ee_failed: 'Failed'
	};
	eq(placeholderText('peer-key', table), 'untrusted', 'placeholder text');
	eq(blockText({ kind: 'server', reason: 'You blocked them.' }, '@ada', table), 'You blocked them.', 'server reason');
	eq(blockText({ kind: 'peer-off' }, '@ada', table), 'Off: @ada', 'peer off names the person');
	eq(blockText({ kind: 'downgrade' }, '@ada', table), 'Used before: @ada', 'downgrade names the person');
	eq(blockText({ kind: 'changed' }, '@ada', table), 'Trust @ada', 'changed names the person');
	eq(blockText({ kind: 'setup' }, '@ada', table), 'Turn on', 'setup says turn on');
	eq(blockText({ kind: 'self-off' }, '@ada', table), 'Turn on', 'self-off says turn on');
	eq(blockText({ kind: 'unlock' }, '@ada', table), 'Unlock first', 'unlock');
	eq(blockText({ kind: 'loading' }, '@ada', table), 'Broken', 'loading is the error line');
	eq(blockText({ kind: 'error' }, '@ada', table), 'Broken', 'error');
	eq(keyErrorText(new E2eeError('passphrase'), table), 'Wrong passphrase', 'wrong passphrase');
	eq(keyErrorText(new E2eeError('unsupported'), table), 'No crypto', 'unsupported');
	eq(keyErrorText(Object.assign(new Error('Server said no'), { status: 409 }), table), 'Server said no', 'an API error speaks');
	eq(keyErrorText(new Error('boom'), table), 'Failed', 'anything else is generic');
}
