// Vectors for the Keys class of assets/js/e2ee.js.
const eq = (actual, expected, what) => {
	if (JSON.stringify(actual) !== JSON.stringify(expected)) {
		throw new Error(`${what}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
	}
};
const ok = (value, what) => {
	if (!value) throw new Error(what);
};

const ADA = 'aaaaaaaa-0000-7000-8000-000000000001';
const BOB = 'bbbbbbbb-0000-7000-8000-000000000002';
const PASSPHRASE = 'correct horse battery staple';

/** A server that holds one key per account, like `GET`/`PUT /me/message-key`. */
function server() {
	let key = null;
	const puts = [];
	const api = {
		current: async () => ({ key }),
		put: async (body) => {
			puts.push(body);
			if (key && !body.replace) throw Object.assign(new Error('You already have a message key.'), { status: 409 });
			key = { publicKey: body.publicKey, backup: body.backup, createdAt: '2026-11-11T00:00:00Z' };
			return key;
		}
	};
	return { api, puts };
}

const rejects = async (promise) => {
	try {
		await promise;
	} catch (error) {
		return error;
	}
	return null;
};

export async function run(mod) {
	const { Keys, E2eeError, memoryStore, passphraseProblem, seal } = mod;

	eq(passphraseProblem('short one', 'short one'), 'short', 'nine characters are short');
	eq(passphraseProblem('🔒🔒🔒🔒🔒🔒🔒🔒🔒', '🔒🔒🔒🔒🔒🔒🔒🔒🔒'), 'short', 'characters, not code units');
	eq(passphraseProblem('0123456789', '0123456780'), 'mismatch', 'mismatch');
	eq(passphraseProblem('0123456789', '0123456789'), null, 'acceptable');

	// none, then ready after setup; uploads only a backup and the public key
	let remote = server();
	let keys = new Keys(memoryStore(), remote.api);
	await keys.load(ADA);
	eq(keys.status, 'none', 'no key on the server');
	await keys.setup(PASSPHRASE);
	eq(keys.status, 'ready', 'ready after setup');
	eq(remote.puts.length, 1, 'one upload');
	eq(remote.puts[0].replace, false, 'not a replacement');
	ok(/^v1\.\d{6,8}(\.[A-Za-z0-9_-]+){3}$/.test(remote.puts[0].backup), 'the backup is a v1 text');
	eq(keys.identity.publicKey, remote.puts[0].publicKey, 'the uploaded key is the identity');

	// locked on a device without the key
	remote = server();
	const first = new Keys(memoryStore(), remote.api);
	await first.load(ADA);
	await first.setup(PASSPHRASE);
	const second = new Keys(memoryStore(), remote.api);
	await second.load(ADA);
	eq(second.status, 'locked', 'locked on a second device');
	const error = await rejects(second.unlock('wrong passphrase here'));
	ok(error instanceof E2eeError && error.reason === 'passphrase', 'a wrong passphrase is named');
	eq(second.status, 'locked', 'still locked');
	await second.unlock(PASSPHRASE);
	eq(second.status, 'ready', 'unlocked');
	eq(second.identity.publicKey, first.identity.publicKey, 'the same key');

	// ready after a reload of the same device
	remote = server();
	let store = memoryStore();
	let started = new Keys(store, remote.api);
	await started.load(ADA);
	await started.setup(PASSPHRASE);
	const reloaded = new Keys(store, remote.api);
	await reloaded.ensure(ADA);
	eq(reloaded.status, 'ready', 'ready after reload');

	// locked when the key stored here is not the server key any more
	remote = server();
	store = memoryStore();
	started = new Keys(store, remote.api);
	await started.load(ADA);
	await started.setup(PASSPHRASE);
	const elsewhere = new Keys(memoryStore(), remote.api);
	await elsewhere.load(ADA);
	await elsewhere.reset(PASSPHRASE);
	const stale = new Keys(store, remote.api);
	await stale.load(ADA);
	eq(stale.status, 'locked', 'a stale key is locked');

	// forget
	remote = server();
	keys = new Keys(memoryStore(), remote.api);
	await keys.load(ADA);
	await keys.setup(PASSPHRASE);
	await keys.trust(BOB, 'bob-key');
	await keys.forget();
	eq(keys.status, 'locked', 'forgotten goes back to locked');
	eq(keys.identity, null, 'identity forgotten');
	eq(await keys.pinFor(BOB), null, 'pins forgotten');

	// reset keeps the pins
	remote = server();
	keys = new Keys(memoryStore(), remote.api);
	await keys.load(ADA);
	await keys.setup(PASSPHRASE);
	const before = keys.identity.publicKey;
	await keys.trust(BOB, 'bob-key');
	await keys.reset('another long passphrase');
	eq(remote.puts.at(-1).replace, true, 'reset replaces');
	eq(keys.status, 'ready', 'ready after reset');
	ok(keys.identity.publicKey !== before, 'a new key');
	eq(await keys.pinFor(BOB), 'bob-key', 'pins kept');

	// another device got there first
	remote = server();
	keys = new Keys(memoryStore(), remote.api);
	await keys.load(ADA);
	const other = new Keys(memoryStore(), remote.api);
	await other.load(ADA);
	await other.setup(PASSPHRASE);
	const conflict = await rejects(keys.setup(PASSPHRASE));
	eq(conflict?.status, 409, 'a conflict');
	eq(keys.status, 'locked', 'locked, not none');

	// the server cannot be asked
	keys = new Keys(memoryStore(), {
		current: async () => {
			throw new Error('offline');
		},
		put: async () => {
			throw new Error('offline');
		}
	});
	await keys.load(ADA);
	eq(keys.status, 'error', 'an error');

	// pins
	keys = new Keys(memoryStore(), server().api);
	await keys.load(ADA);
	await keys.setup(PASSPHRASE);
	eq(await keys.observe(BOB, 'k1'), 'new', 'first sight');
	eq(await keys.pinFor(BOB), 'k1', 'pinned');
	eq(await keys.observe(BOB, 'k1'), 'same', 'same key');
	eq(await keys.observe(BOB, 'k2'), 'changed', 'other key');
	eq(await keys.pinFor(BOB), 'k1', 'not pinned until trusted');
	await keys.trust(BOB, 'k2');
	eq(await keys.observe(BOB, 'k2'), 'same', 'trusted');

	// the change callback runs after each change
	let changes = 0;
	keys = new Keys(memoryStore(), server().api, () => changes++);
	await keys.load(ADA);
	const afterLoad = changes;
	ok(afterLoad >= 2, 'loading and none are announced');
	await keys.trust(BOB, 'k1');
	await keys.trust(BOB, 'k1');
	eq(changes, afterLoad + 1, 'only a real pin change is announced');

	// Keys.line
	const start = async (id) => {
		const k = new Keys(memoryStore(), server().api);
		await k.load(id);
		await k.setup(PASSPHRASE);
		return k;
	};
	const dto = (ciphertext, over = {}) => ({
		id: '10',
		text: ciphertext === null ? 'plain words' : '',
		ciphertext,
		createdAt: '2026-11-11T00:00:00Z',
		mine: false,
		readAt: null,
		emojis: [],
		...over
	});
	let ada = await start(ADA);
	let bob = await start(BOB);
	let sealed = await seal({ text: 'secret', mine: ada.identity, theirs: bob.identity.publicKey, senderId: ADA, recipientId: BOB });
	eq(await bob.line(dto(null), ADA, []), { text: 'plain words' }, 'plaintext as it is');
	const locked = new Keys(memoryStore(), server().api);
	await locked.load(BOB);
	eq(await locked.line(dto(sealed), ADA, [ada.identity.publicKey]), { placeholder: 'sealed' }, 'sealed while locked');
	const opened = await bob.line(dto(sealed), ADA, [ada.identity.publicKey]);
	eq(opened, { text: 'secret' }, 'opens');
	ok((await bob.line(dto(sealed), ADA, [ada.identity.publicKey])) === opened, 'opens once');
	eq(await bob.line(dto(sealed), ADA, [bob.identity.publicKey]), { placeholder: 'peer-key' }, 'another pinned key');
	eq(await bob.line(dto(sealed), ADA, []), { placeholder: 'peer-key' }, 'no pinned key');
	sealed = await seal({ text: 'sent by ada', mine: ada.identity, theirs: bob.identity.publicKey, senderId: ADA, recipientId: BOB });
	eq(await ada.line(dto(sealed, { mine: true }), BOB, [bob.identity.publicKey]), { text: 'sent by ada' }, 'own message');

	// trusted history
	ada = await start(ADA);
	await ada.trust(BOB, 'k1');
	await ada.trust(BOB, 'k1');
	await ada.trust(BOB, 'k2');
	eq(await ada.trustedFor(BOB), ['k1', 'k2'], 'appends and keeps');
	eq(await ada.pinFor(BOB), 'k2', 'pins the newest');
	eq(await ada.observe(BOB, 'k1'), 'changed', 'an earlier key is not the pin');
	await ada.forget();
	eq(await ada.trustedFor(BOB), [], 'forgets the whole list');

	// both directions stay readable after the contact resets
	ada = await start(ADA);
	const oldBob = await start(BOB);
	const newBob = await start(BOB);
	const adaKey = ada.identity;
	const message = (ciphertext, id, mine) => ({ id, text: '', ciphertext, createdAt: '2026-11-11T00:00:00Z', mine, readAt: null, emojis: [] });
	const toBob = await seal({ text: 'to old bob', mine: adaKey, theirs: oldBob.identity.publicKey, senderId: ADA, recipientId: BOB });
	const fromBob = await seal({ text: 'from old bob', mine: oldBob.identity, theirs: adaKey.publicKey, senderId: BOB, recipientId: ADA });
	const fromNewBob = await seal({ text: 'from new bob', mine: newBob.identity, theirs: adaKey.publicKey, senderId: BOB, recipientId: ADA });
	const sent = message(toBob, '1', true);
	const received = message(fromBob, '2', false);
	const fresh = message(fromNewBob, '3', false);
	await ada.trust(BOB, oldBob.identity.publicKey);
	eq(await ada.line(received, BOB, await ada.trustedFor(BOB)), { text: 'from old bob' }, 'old key read');
	await ada.trust(BOB, newBob.identity.publicKey);
	const trusted = await ada.trustedFor(BOB);
	eq(await ada.line(sent, BOB, trusted), { text: 'to old bob' }, 'sent to the old key');
	eq(await ada.line(received, BOB, trusted), { text: 'from old bob' }, 'received from the old key');
	eq(await ada.line(fresh, BOB, trusted), { text: 'from new bob' }, 'received from the new key');

	// a key that was never trusted
	ada = await start(ADA);
	bob = await start(BOB);
	const eve = await start(BOB);
	const fromEve = message(await seal({ text: 'from eve', mine: eve.identity, theirs: ada.identity.publicKey, senderId: BOB, recipientId: ADA }), '4', false);
	await ada.trust(BOB, bob.identity.publicKey);
	eq(await ada.line(fromEve, BOB, await ada.trustedFor(BOB)), { placeholder: 'peer-key' }, 'never trusted');

	// Keys.preview
	const adaKeys = await start(ADA);
	const bobKeys = await start(BOB);
	const ciphertext = await seal({ text: 'see you at six', mine: adaKeys.identity, theirs: bobKeys.identity.publicKey, senderId: ADA, recipientId: BOB });
	const row = (peerKey) => ({
		user: { id: ADA },
		unread: 1,
		peerKey,
		last: { id: '9', text: '', ciphertext, createdAt: '2026-11-11T00:00:00Z', mine: false, readAt: null, emojis: [] }
	});
	const served = adaKeys.identity.publicKey;
	eq(await bobKeys.preview(row(served)), null, 'no preview before a pin');
	await bobKeys.trust(ADA, served);
	eq(await bobKeys.preview(row(served)), 'see you at six', 'preview under the pin');
	eq(await bobKeys.preview(row(bobKeys.identity.publicKey)), null, 'no preview when the served key moved');
	eq(await bobKeys.preview(row(null)), null, 'no preview without a served key');

	// forgetAccounts
	const gone = memoryStore();
	await gone.putIdentity('a', adaKeys.identity);
	await gone.putIdentity('b', adaKeys.identity);
	await mod.forgetAccounts(['a'], gone);
	eq(await gone.identity('a'), null, 'forgotten account');
	eq((await gone.identity('b'))?.publicKey, adaKeys.identity.publicKey, 'other account kept');
	await mod.forgetAccounts(['b'], { forget: async () => { throw new Error('store failed'); } });
}
