// Vectors for the key store of assets/js/e2ee.js, plus the real
// IndexedDB store against the schema older clients wrote (`tuita-e2ee`: `identity`, `peers`).
const eq = (actual, expected, what) => {
	if (JSON.stringify(actual) !== JSON.stringify(expected)) {
		throw new Error(`${what}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
	}
};

const pin = (publicKey) => ({ publicKey, pinnedAt: '2026-11-11T00:00:00Z' });

export async function run(mod) {
	const { createIdentity, memoryStore, pinList, indexedDbStore } = mod;
	const ada = await createIdentity('correct horse battery staple');

	const store = memoryStore();
	eq(await store.identity('me'), null, 'no identity yet');
	await store.putIdentity('me', ada);
	eq((await store.identity('me'))?.publicKey, ada.publicKey, 'keeps an identity');
	eq(await store.identity('other'), null, 'per account');

	const pins = memoryStore();
	eq(await pins.pins('me', 'peer'), [], 'no pins yet');
	await pins.putPins('me', 'peer', [pin('k1')]);
	await pins.putPins('other', 'peer', [pin('k2')]);
	await pins.putPins('me', 'peer', [pin('k1'), pin('k3')]);
	eq((await pins.pins('me', 'peer')).map((p) => p.publicKey), ['k1', 'k3'], 'ordered pin list');
	eq((await pins.pins('other', 'peer')).map((p) => p.publicKey), ['k2'], 'per account and peer');

	const forgotten = memoryStore();
	await forgotten.putIdentity('me', ada);
	await forgotten.putIdentity('other', ada);
	await forgotten.putPins('me', 'peer', [pin('k1'), pin('k4')]);
	await forgotten.putPins('other', 'peer', [pin('k2')]);
	await forgotten.forget('me');
	eq(await forgotten.identity('me'), null, 'identity forgotten');
	eq(await forgotten.pins('me', 'peer'), [], 'pins forgotten');
	eq((await forgotten.identity('other'))?.publicKey, ada.publicKey, 'other identity kept');
	eq((await forgotten.pins('other', 'peer')).map((p) => p.publicKey), ['k2'], 'other pins kept');

	eq(pinList(pin('k1')), [pin('k1')], 'a lone pin is a list of one');
	eq(pinList([pin('k1'), pin('k2')]), [pin('k1'), pin('k2')], 'a list as it is');
	eq(pinList(undefined), [], 'nothing is empty');

	// The IndexedDB store keeps a non-extractable key across a store instance and forgets by prefix.
	const real = indexedDbStore();
	await real.forget('unit-me');
	await real.putIdentity('unit-me', ada);
	await real.putPins('unit-me', 'p1', [pin('k1')]);
	await real.putPins('unit-me', 'p2', [pin('k2')]);
	await real.putPins('unit-other', 'p1', [pin('k9')]);
	const again = indexedDbStore();
	const stored = await again.identity('unit-me');
	eq(stored.publicKey, ada.publicKey, 'identity survives');
	eq(stored.privateKey.extractable, false, 'the private key stays non-extractable');
	eq((await again.pins('unit-me', 'p1')).map((p) => p.publicKey), ['k1'], 'pins survive');
	await again.forget('unit-me');
	eq(await again.identity('unit-me'), null, 'idb identity forgotten');
	eq(await again.pins('unit-me', 'p1'), [], 'idb pins forgotten');
	eq(await again.pins('unit-me', 'p2'), [], 'every pin of the account forgotten');
	eq((await again.pins('unit-other', 'p1')).map((p) => p.publicKey), ['k9'], 'other account kept');
	await again.forget('unit-other');
	eq(await mod.browserStore().identity('unit-none'), null, 'the browser store reads');
}
