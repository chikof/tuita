// End-to-end encryption for direct messages, on WebCrypto only.
// The algorithms, parameters, IndexedDB `tuita-e2ee` schema and wire format are fixed: what older clients sealed still opens.
//
// - Identity: one ECDH P-256 key pair per account.
// - Backup: the private key as JWK, sealed with AES-256-GCM under PBKDF2-SHA-256 of a passphrase.
// - Message: ECDH(my private, their public), HKDF-SHA-256 with a random salt per message, then
//   AES-256-GCM. The padded plaintext hides the length to within 128 bytes. The additional data
//   binds sender id, recipient id and both public keys.
// There is no forward secrecy: a stolen private key opens every message sealed to it.
//
// The module also drives the page (all of it plain markup the server rendered):
// - `[data-e2ee-lines]` (a thread: `data-me`, `data-peer-id`, `data-peer-key`, `data-mode`) opens
//   every `[data-ciphertext]` bubble inside it, pins the peer's key the first time it is seen, and
//   re-opens after DOM changes or key changes. A bubble holds `[data-ph]` (placeholder text),
//   `[data-text]` (the opened text) and `[data-lock]`.
// - `[data-e2ee-previews]` (the conversation list: `data-me`) opens `[data-preview]` rows.
// - `[data-island="e2ee-settings"]` (`data-me`) shows `[data-when="none locked ready loading error"]`
//   elements by key status; `[data-act="e2ee-forget"|"e2ee-retry"]` buttons.
// - `[data-prompt="setup|unlock|reset"] form` is the passphrase form (setup, unlock, reset).
// - A click on `[data-forget-keys="<id> <id>"]` forgets those accounts' keys before the click's own
//   handler (the sign-out) runs.
// - `globalThis.tuitaE2ee` hands the keys and the pure helpers to messages.js.
// Client strings: e2ee_line_*, e2ee_preview, e2ee_unlock_wrong, e2ee_unsupported, e2ee_failed,
// e2ee_passphrase_short, e2ee_passphrase_mismatch, e2ee_*_done.

const VERSION = 'v1';
const CURVE = { name: 'ECDH', namedCurve: 'P-256' };
const PBKDF2_ITERATIONS = 600_000;
const PUBLIC_KEY_BYTES = 65;
const SALT_BYTES = 16;
const IV_BYTES = 12;
const PAD_BLOCK = 128;
const LENGTH_PREFIX = 4;
const LABEL = 'tuita-dm-v1';

const encoder = new TextEncoder();
const decoder = new TextDecoder('utf-8', { fatal: true });

/** Why an operation failed, for the caller to tell the person:
 * 'format' | 'passphrase' | 'key-mismatch' | 'own-key' | 'decrypt' | 'unsupported'. */
export class E2eeError extends Error {
	constructor(reason, message = reason) {
		super(message);
		this.name = 'E2eeError';
		this.reason = reason;
	}
}

export function toBase64Url(bytes) {
	let binary = '';
	for (const byte of bytes) binary += String.fromCharCode(byte);
	return btoa(binary).replaceAll('+', '-').replaceAll('/', '_').replace(/=+$/, '');
}

export function fromBase64Url(text) {
	if (!/^[A-Za-z0-9_-]*$/.test(text) || text.length % 4 === 1) throw new E2eeError('format');
	const padded = text.replaceAll('-', '+').replaceAll('_', '/') + '='.repeat((4 - (text.length % 4)) % 4);
	const binary = atob(padded);
	const out = new Uint8Array(binary.length);
	for (let i = 0; i < binary.length; i++) out[i] = binary.charCodeAt(i);
	return out;
}

const concat = (...parts) => {
	const out = new Uint8Array(parts.reduce((sum, part) => sum + part.length, 0));
	let at = 0;
	for (const part of parts) {
		out.set(part, at);
		at += part.length;
	}
	return out;
};

export function sameBytes(a, b) {
	if (a.length !== b.length) return false;
	let diff = 0;
	for (let i = 0; i < a.length; i++) diff |= a[i] ^ b[i];
	return diff === 0;
}

const random = (length) => crypto.getRandomValues(new Uint8Array(length));

const subtle = () => {
	if (typeof crypto === 'undefined' || !crypto.subtle) throw new E2eeError('unsupported');
	return crypto.subtle;
};

const publicFromJwk = (jwk) => concat(new Uint8Array([4]), fromBase64Url(jwk.x), fromBase64Url(jwk.y));

const importPrivate = (jwk) =>
	subtle().importKey('jwk', { ...jwk, key_ops: ['deriveBits'], ext: false }, CURVE, false, ['deriveBits']);

const importPublic = (raw) => {
	if (raw.length !== PUBLIC_KEY_BYTES || raw[0] !== 4) throw new E2eeError('format');
	return subtle().importKey('raw', raw, CURVE, false, []);
};

const passphraseKey = async (passphrase, salt, iterations) => {
	const base = await subtle().importKey('raw', encoder.encode(passphrase.normalize('NFKC')), 'PBKDF2', false, ['deriveKey']);
	return subtle().deriveKey(
		{ name: 'PBKDF2', hash: 'SHA-256', salt, iterations },
		base,
		{ name: 'AES-GCM', length: 256 },
		false,
		['encrypt', 'decrypt']
	);
};

/** Makes a key pair and seals the private half under `passphrase`:
 * `{ publicKey, privateKey, backup }`. */
export async function createIdentity(passphrase) {
	const pair = await subtle().generateKey(CURVE, true, ['deriveBits']);
	const jwk = await subtle().exportKey('jwk', pair.privateKey);
	const publicKey = toBase64Url(publicFromJwk(jwk));
	const salt = random(SALT_BYTES);
	const iv = random(IV_BYTES);
	const key = await passphraseKey(passphrase, salt, PBKDF2_ITERATIONS);
	const sealed = await subtle().encrypt(
		{ name: 'AES-GCM', iv, additionalData: fromBase64Url(publicKey) },
		key,
		encoder.encode(JSON.stringify({ kty: jwk.kty, crv: jwk.crv, x: jwk.x, y: jwk.y, d: jwk.d }))
	);
	return {
		publicKey,
		privateKey: await importPrivate(jwk),
		backup: [VERSION, PBKDF2_ITERATIONS, toBase64Url(salt), toBase64Url(iv), toBase64Url(new Uint8Array(sealed))].join('.')
	};
}

/**
 * Opens a backup with `passphrase`. `publicKey` is what the server says this account's key is;
 * a backup whose key differs is refused, so the server cannot pair a key of its own with it.
 */
export async function restoreIdentity(publicKey, backup, passphrase) {
	const parts = backup.split('.');
	if (parts.length !== 5 || parts[0] !== VERSION) throw new E2eeError('format');
	const iterations = Number(parts[1]);
	if (!Number.isInteger(iterations) || iterations < 100_000 || iterations > 10_000_000) {
		throw new E2eeError('format');
	}
	const claimed = fromBase64Url(publicKey);
	let jwk;
	try {
		const key = await passphraseKey(passphrase, fromBase64Url(parts[2]), iterations);
		const opened = await subtle().decrypt(
			{ name: 'AES-GCM', iv: fromBase64Url(parts[3]), additionalData: claimed },
			key,
			fromBase64Url(parts[4])
		);
		jwk = JSON.parse(decoder.decode(opened));
	} catch (error) {
		if (error instanceof E2eeError) throw error;
		throw new E2eeError('passphrase');
	}
	if (!sameBytes(publicFromJwk(jwk), claimed)) throw new E2eeError('key-mismatch');
	return { publicKey, privateKey: await importPrivate(jwk) };
}

const pad = (text) => {
	const body = encoder.encode(text);
	const total = Math.ceil((LENGTH_PREFIX + body.length) / PAD_BLOCK) * PAD_BLOCK;
	const out = new Uint8Array(total);
	new DataView(out.buffer).setUint32(0, body.length);
	out.set(body, LENGTH_PREFIX);
	return out;
};

const unpad = (padded) => {
	if (padded.length < LENGTH_PREFIX) throw new E2eeError('decrypt');
	const length = new DataView(padded.buffer, padded.byteOffset, padded.byteLength).getUint32(0);
	if (LENGTH_PREFIX + length > padded.length) throw new E2eeError('decrypt');
	return decoder.decode(padded.subarray(LENGTH_PREFIX, LENGTH_PREFIX + length));
};

const messageKey = async (mine, theirs, salt, usage) => {
	const shared = await subtle().deriveBits({ name: 'ECDH', public: await importPublic(theirs) }, mine, 256);
	const base = await subtle().importKey('raw', shared, 'HKDF', false, ['deriveKey']);
	return subtle().deriveKey(
		{ name: 'HKDF', hash: 'SHA-256', salt, info: encoder.encode(LABEL) },
		base,
		{ name: 'AES-GCM', length: 256 },
		false,
		[usage]
	);
};

const context = (senderId, recipientId, senderKey, recipientKey) =>
	concat(encoder.encode(`${LABEL}|${senderId}|${recipientId}|`), senderKey, recipientKey);

/**
 * Seals `text` from `mine` to `theirs`: `v1.<sender key>.<recipient key>.<salt>.<iv>.<ciphertext>`.
 * Input: `{ text, mine: Identity, theirs: base64url public key, senderId, recipientId }`.
 */
export async function seal(input) {
	const senderKey = fromBase64Url(input.mine.publicKey);
	const recipientKey = fromBase64Url(input.theirs);
	const salt = random(SALT_BYTES);
	const iv = random(IV_BYTES);
	const key = await messageKey(input.mine.privateKey, recipientKey, salt, 'encrypt');
	const sealed = await subtle().encrypt(
		{ name: 'AES-GCM', iv, additionalData: context(input.senderId, input.recipientId, senderKey, recipientKey) },
		key,
		pad(input.text)
	);
	return [
		VERSION,
		toBase64Url(senderKey),
		toBase64Url(recipientKey),
		toBase64Url(salt),
		toBase64Url(iv),
		toBase64Url(new Uint8Array(sealed))
	].join('.');
}

/** The keys a sealed message names, without opening it. */
export function keysOf(sealed) {
	const parts = sealed.split('.');
	if (parts.length !== 6 || parts[0] !== VERSION) return null;
	return { sender: parts[1], recipient: parts[2] };
}

/**
 * Opens a sealed message. The sender key it names must be the pinned key of the other side (or the
 * viewer's own), and the recipient key the viewer's own (or the pinned one): anything else means
 * the server or a key change put it there, and nothing is decrypted.
 * Input: `{ sealed, mine, sentByMe, theirs, senderId, recipientId }`.
 */
export async function open(input) {
	const parts = input.sealed.split('.');
	if (parts.length !== 6 || parts[0] !== VERSION) throw new E2eeError('format');
	const senderKey = fromBase64Url(parts[1]);
	const recipientKey = fromBase64Url(parts[2]);
	const salt = fromBase64Url(parts[3]);
	const iv = fromBase64Url(parts[4]);
	const sealed = fromBase64Url(parts[5]);
	if (salt.length !== SALT_BYTES || iv.length !== IV_BYTES) throw new E2eeError('format');

	const mine = fromBase64Url(input.mine.publicKey);
	const theirs = fromBase64Url(input.theirs);
	const [ownSide, otherSide] = input.sentByMe ? [senderKey, recipientKey] : [recipientKey, senderKey];
	if (!sameBytes(ownSide, mine)) throw new E2eeError('own-key');
	if (!sameBytes(otherSide, theirs)) throw new E2eeError('key-mismatch');

	try {
		const key = await messageKey(input.mine.privateKey, theirs, salt, 'decrypt');
		const padded = await subtle().decrypt(
			{ name: 'AES-GCM', iv, additionalData: context(input.senderId, input.recipientId, senderKey, recipientKey) },
			key,
			sealed
		);
		return unpad(new Uint8Array(padded));
	} catch (error) {
		if (error instanceof E2eeError) throw error;
		throw new E2eeError('decrypt');
	}
}

/** Sixty digits, in fifteen groups, both sides compute the same from the two public keys. */
export async function safetyNumber(a, b) {
	const [first, second] = [fromBase64Url(a), fromBase64Url(b)].sort((x, y) => {
		for (let i = 0; i < Math.min(x.length, y.length); i++) if (x[i] !== y[i]) return x[i] - y[i];
		return x.length - y.length;
	});
	const digest = new Uint8Array(await subtle().digest('SHA-256', concat(first, second)));
	const groups = [];
	for (let i = 0; i < 30; i += 2) {
		groups.push((((digest[i] << 8) | digest[i + 1]) % 10_000).toString().padStart(4, '0'));
	}
	return groups.join(' ');
}

/** What a stored pin says about the key the server serves now: 'new' | 'same' | 'changed'. */
export const checkPin = (pinned, served) => (pinned === null ? 'new' : pinned === served ? 'same' : 'changed');

// ---------------------------------------------------------------------------------------------
// Key store

/** Reads a stored `peers` value: absent is empty, a lone pin from before the list is a list of one. */
export const pinList = (stored) => (stored == null ? [] : Array.isArray(stored) ? stored : [stored]);

/** A pin for `publicKey`, made now. */
export const pinned = (publicKey) => ({ publicKey, pinnedAt: new Date().toISOString() });

const pinKey = (me, peer) => `${me}:${peer}`;

export function memoryStore() {
	const identities = new Map();
	const pins = new Map();
	return {
		identity: async (userId) => identities.get(userId) ?? null,
		putIdentity: async (userId, identity) => void identities.set(userId, identity),
		pins: async (me, peer) => pins.get(pinKey(me, peer)) ?? [],
		putPins: async (me, peer, list) => void pins.set(pinKey(me, peer), list),
		forget: async (userId) => {
			identities.delete(userId);
			for (const key of pins.keys()) if (key.startsWith(`${userId}:`)) pins.delete(key);
		}
	};
}

const DB = 'tuita-e2ee';

const done = (request) =>
	new Promise((resolve, reject) => {
		request.onsuccess = () => resolve(request.result);
		request.onerror = () => reject(request.error);
	});

const finished = (tx) =>
	new Promise((resolve, reject) => {
		tx.oncomplete = () => resolve();
		tx.onerror = tx.onabort = () => reject(tx.error);
	});

/** IndexedDB `tuita-e2ee`: stores `identity` (by user id) and `peers` (by `<me>:<peer>`). */
export function indexedDbStore() {
	let opened;
	const db = () =>
		(opened ??= new Promise((resolve, reject) => {
			const request = indexedDB.open(DB, 1);
			request.onupgradeneeded = () => {
				request.result.createObjectStore('identity');
				request.result.createObjectStore('peers');
			};
			request.onsuccess = () => resolve(request.result);
			request.onerror = () => reject(request.error);
		}));
	const store = async (name, mode) => (await db()).transaction(name, mode).objectStore(name);

	return {
		identity: async (userId) => (await done((await store('identity', 'readonly')).get(userId))) ?? null,
		putIdentity: async (userId, identity) => {
			await done((await store('identity', 'readwrite')).put({ publicKey: identity.publicKey, privateKey: identity.privateKey }, userId));
		},
		pins: async (me, peer) => pinList(await done((await store('peers', 'readonly')).get(pinKey(me, peer)))),
		putPins: async (me, peer, list) => {
			await done((await store('peers', 'readwrite')).put(list, pinKey(me, peer)));
		},
		forget: async (userId) => {
			const tx = (await db()).transaction(['identity', 'peers'], 'readwrite');
			tx.objectStore('identity').delete(userId);
			tx.objectStore('peers').delete(IDBKeyRange.bound(`${userId}:`, `${userId}:\uffff`));
			await finished(tx);
		}
	};
}

/** The browser's store; without IndexedDB the keys last until the page closes. */
export const browserStore = () => (typeof indexedDB === 'undefined' ? memoryStore() : indexedDbStore());

// ---------------------------------------------------------------------------------------------
// Lines: what a message shows

const PLACEHOLDERS = {
	'own-key': 'own-key',
	'key-mismatch': 'peer-key',
	unsupported: 'unsupported',
	format: 'unreadable',
	decrypt: 'unreadable',
	passphrase: 'unreadable'
};

/** The placeholder ('sealed' | 'own-key' | 'peer-key' | 'unreadable' | 'unsupported') for a failure reason. */
export const placeholderFor = (reason) => PLACEHOLDERS[reason];

export const SEALED = { placeholder: 'sealed' };

/** Opens one sealed message; every failure becomes a placeholder. */
export async function openLine(input) {
	if (input.theirs === null) return { placeholder: placeholderFor('key-mismatch') };
	try {
		return { text: await open({ ...input, theirs: input.theirs }) };
	} catch (error) {
		return { placeholder: placeholderFor(error instanceof E2eeError ? error.reason : 'decrypt') };
	}
}

// ---------------------------------------------------------------------------------------------
// Conversation state

const blocked = (note, block, pin = null) => ({ note, pin, send: null, block });

/**
 * How a message to this person goes out, or why it cannot. Input: `{ mode, blockedReason, remote,
 * peerKey, keys, pinned }`. Output: `{ note, pin, send, block }`; `note` is 'remote' | 'unencrypted'
 * | null, `send` 'sealed' | 'plain' | null, `block` `{ kind, reason? }` | null with `kind` one of
 * server, peer-off, self-off, downgrade, loading, error, setup, unlock, changed.
 */
export function conversationState(input) {
	const { mode, peerKey, keys: status, pinned: pin } = input;
	const note = mode !== 'plain' ? null : input.remote ? 'remote' : 'unencrypted';

	if (input.blockedReason) return blocked(note, { kind: 'server', reason: input.blockedReason });

	if (mode === 'plain') {
		// A pin exists only for a peer this browser sealed with, so no plaintext after it, whatever
		// the server says about where the peer lives.
		if (pin !== null) return blocked(note, { kind: 'downgrade' });
		return { note, pin: null, send: 'plain', block: null };
	}

	if (mode === 'unavailable') {
		if (peerKey === null) return blocked(note, { kind: 'peer-off' });
		return blocked(note, status === 'none' ? { kind: 'setup' } : { kind: 'self-off' });
	}

	if (peerKey === null) return blocked(note, { kind: 'peer-off' });
	switch (status) {
		case 'loading':
		case 'error':
		case 'none':
		case 'locked':
			return blocked(note, { kind: status === 'locked' ? 'unlock' : status === 'none' ? 'setup' : status });
		default: {
			const check = checkPin(pin, peerKey);
			return check === 'changed' ? blocked(note, { kind: 'changed' }, check) : { note, pin: check, send: 'sealed', block: null };
		}
	}
}

// ---------------------------------------------------------------------------------------------
// Sending

const isConflict = (error) => typeof error === 'object' && error !== null && 'status' in error && error.status === 409;

/**
 * Sends `text` the way the conversation's state says: sealed to the pinned key, or plain, and
 * never plain to someone who has a key. A 409 (keys changed, or the other side turned encryption
 * on) re-reads the view and decides again, once; a refusal after that is the caller's to show.
 *
 * `first` is `{ mode, blockedReason, peerKey }`; `deps` is `{ me: { id, identity, keys },
 * peer: { id, remote }, pinned(), pin(key), post(body), fetchView() }`. Resolves to
 * `{ sent: true, message, view }` or `{ sent: false, state, view }`.
 */
export async function sendMessage(text, first, deps) {
	let view = first;
	for (let attempt = 0; ; attempt++) {
		const state = conversationState({
			...view,
			remote: deps.peer.remote,
			keys: deps.me.keys,
			pinned: await deps.pinned()
		});
		if (state.send === null) return { sent: false, state, view };

		try {
			if (state.send === 'plain') {
				return { sent: true, message: await deps.post({ text }), view };
			}
			const theirs = view.peerKey;
			if (theirs === null || deps.me.identity === null) throw new Error('sealed send without keys');
			if (state.pin === 'new') await deps.pin(theirs);
			const ciphertext = await seal({
				text,
				mine: deps.me.identity,
				theirs,
				senderId: deps.me.id,
				recipientId: deps.peer.id
			});
			return { sent: true, message: await deps.post({ ciphertext }), view };
		} catch (error) {
			if (attempt > 0 || !isConflict(error)) throw error;
			view = await deps.fetchView();
		}
	}
}

// ---------------------------------------------------------------------------------------------
// Keys of the signed-in account

export const MIN_PASSPHRASE = 10;

/** `null` when the passphrase pair is acceptable, else 'short' or 'mismatch'. */
export function passphraseProblem(passphrase, again) {
	if ([...passphrase].length < MIN_PASSPHRASE) return 'short';
	return passphrase === again ? null : 'mismatch';
}

/**
 * The signed-in account's message keys on this browser. `status`: `none` (the server has no key),
 * `locked` (it has one this browser cannot use yet), `ready` (the unwrapped private key sits in
 * the store), `loading`, `error`. Opened messages are cached in memory only, per message and
 * pinned key. `onchange` runs after every change of status or pins.
 */
export class Keys {
	status = 'loading';

	#store;
	#api;
	#onchange;
	#userId = null;
	#identity = null;
	#loading = null;
	#lines = {};

	constructor(store, api, onchange = () => {}) {
		this.#store = store;
		this.#api = api;
		this.#onchange = onchange;
	}

	get identity() {
		return this.#identity;
	}

	get userId() {
		return this.#userId;
	}

	#set(status) {
		this.status = status;
		this.#onchange();
	}

	/** Reads the state for `userId` once; later calls for the same account share the result. */
	ensure(userId) {
		if (this.#loading?.userId === userId) return this.#loading.done;
		return this.load(userId);
	}

	/** Reads the server's key and this browser's identity again. */
	load(userId) {
		this.#userId = userId;
		this.#identity = null;
		this.#lines = {};
		this.#set('loading');
		const done = this.#read(userId);
		this.#loading = { userId, done };
		return done;
	}

	async #read(userId) {
		try {
			const { key } = await this.#api.current();
			const local = await this.#store.identity(userId);
			if (this.#userId !== userId) return;
			if (key === null) {
				this.#set('none');
			} else if (local !== null && local.publicKey === key.publicKey) {
				this.#identity = local;
				this.#set('ready');
			} else {
				this.#set('locked');
			}
		} catch {
			if (this.#userId === userId) this.#set('error');
			this.#loading = null;
		}
	}

	#account() {
		if (this.#userId === null) throw new Error('message keys are not loaded');
		return this.#userId;
	}

	async #adopt(userId, identity) {
		await this.#store.putIdentity(userId, identity);
		this.#identity = identity;
		this.#lines = {};
		this.#set('ready');
	}

	/** Turns encryption on: a new key, sealed under `passphrase`, then uploaded. */
	async setup(passphrase) {
		await this.#publish(passphrase, false);
	}

	/** A new key replaces the old one: earlier messages stay sealed to a key nobody has. */
	async reset(passphrase) {
		await this.#publish(passphrase, true);
	}

	async #publish(passphrase, replace) {
		const userId = this.#account();
		const made = await createIdentity(passphrase);
		try {
			await this.#api.put({ publicKey: made.publicKey, backup: made.backup, replace });
		} catch (error) {
			// Another device got there first: this one is now locked, not "none".
			await this.load(userId);
			throw error;
		}
		await this.#adopt(userId, { publicKey: made.publicKey, privateKey: made.privateKey });
	}

	/** Opens the server's backup with `passphrase`; a wrong one throws `E2eeError('passphrase')`. */
	async unlock(passphrase) {
		const userId = this.#account();
		const { key } = await this.#api.current();
		if (key === null) {
			await this.load(userId);
			return;
		}
		await this.#adopt(userId, await restoreIdentity(key.publicKey, key.backup, passphrase));
	}

	/** Removes this account's key and pins from the device; the server keeps the backup. */
	async forget() {
		const userId = this.#account();
		await this.#store.forget(userId);
		this.#identity = null;
		this.#lines = {};
		await this.load(userId);
	}

	/** Every key trusted for `peerId`, oldest first; the last is the current pin. */
	async trustedFor(peerId) {
		return (await this.#store.pins(this.#account(), peerId)).map((pin) => pin.publicKey);
	}

	/** The key pinned for `peerId`, or `null`. */
	async pinFor(peerId) {
		return (await this.trustedFor(peerId)).at(-1) ?? null;
	}

	/** Compares the served key with the pin and pins it when there is none yet. */
	async observe(peerId, served) {
		const check = checkPin(await this.pinFor(peerId), served);
		if (check === 'new') await this.trust(peerId, served);
		return check;
	}

	/** Makes `key` the current pin for `peerId`; earlier keys stay trusted for the messages sealed under them. */
	async trust(peerId, key) {
		const me = this.#account();
		const list = await this.#store.pins(me, peerId);
		if (list.at(-1)?.publicKey === key) return;
		await this.#store.putPins(me, peerId, [...list.filter((pin) => pin.publicKey !== key), pinned(key)]);
		this.#onchange();
	}

	/** The text of a conversation's newest sealed message, when it opens under the pinned key that is still the served one; else `null`. */
	async preview(conversation) {
		if (this.status !== 'ready' || conversation.last.ciphertext === null) return null;
		const pin = await this.pinFor(conversation.user.id);
		if (pin === null || pin !== conversation.peerKey) return null;
		const line = await this.line(conversation.last, conversation.user.id, [pin]);
		return 'text' in line ? line.text : null;
	}

	/**
	 * What `message` shows. The peer-side key the message names must be one of `trusted`; any other
	 * key leaves the 'peer-key' placeholder and nothing is opened. Opens each message once.
	 */
	async line(message, peerId, trusted) {
		if (message.ciphertext === null) return { text: message.text };
		const mine = this.#identity;
		if (mine === null) return SEALED;
		const named = keysOf(message.ciphertext);
		// A malformed message gets the newest pin so `open` reports it as unreadable.
		const peerSide = named === null ? null : message.mine ? named.recipient : named.sender;
		const theirs = peerSide === null ? (trusted.at(-1) ?? null) : trusted.includes(peerSide) ? peerSide : null;
		const id = `${message.id}|${theirs}`;
		const cached = this.#lines[id];
		if (cached) return cached;
		const own = this.#account();
		const line = await openLine({
			sealed: message.ciphertext,
			mine,
			theirs,
			sentByMe: message.mine,
			senderId: message.mine ? own : peerId,
			recipientId: message.mine ? peerId : own
		});
		if (this.#identity === mine) this.#lines[id] = line;
		return line;
	}
}

// ---------------------------------------------------------------------------------------------
// Forget

/** Drops the keys and pins of the given accounts from this device; a store that fails is skipped. */
export async function forgetAccounts(userIds, store = browserStore()) {
	await Promise.all(userIds.map((id) => store.forget(id).catch(() => undefined)));
}

// ---------------------------------------------------------------------------------------------
// Words for the person

const strings = () => {
	try {
		return JSON.parse(document.getElementById('tuita-l10n')?.textContent ?? '{}');
	} catch {
		return {};
	}
};

/** Fills `{name}` slots of a client string. */
export const fill = (template, values = {}) => String(template).replace(/\{(\w+)\}/g, (hole, name) => (name in values ? String(values[name]) : hole));

const PLACEHOLDER_KEY = {
	sealed: 'e2ee_line_sealed',
	'own-key': 'e2ee_line_own_key',
	'peer-key': 'e2ee_line_peer_key',
	unreadable: 'e2ee_line_unreadable',
	unsupported: 'e2ee_line_unsupported'
};

/** The words that stand in for a message that did not open. */
export const placeholderText = (placeholder, table = strings()) => table[PLACEHOLDER_KEY[placeholder]] ?? '';

/** Why the composer is off, for the kinds that need no prompt of their own. */
export function blockText(block, handle, table = strings()) {
	switch (block.kind) {
		case 'server':
			return block.reason;
		case 'peer-off':
			return fill(table.e2ee_block_peer_off, { handle });
		case 'self-off':
		case 'setup':
			return table.e2ee_block_self_off;
		case 'downgrade':
			return fill(table.e2ee_block_downgrade, { handle });
		case 'unlock':
			return table.e2ee_block_unlock;
		case 'changed':
			return fill(table.e2ee_block_changed, { handle });
		default:
			return table.e2ee_block_error;
	}
}

/** What to tell a person whose passphrase or key setup failed. */
export function keyErrorText(error, table = strings()) {
	if (error instanceof E2eeError) {
		if (error.reason === 'passphrase') return table.e2ee_unlock_wrong;
		if (error.reason === 'unsupported') return table.e2ee_unsupported;
	}
	if (error instanceof Error && 'status' in error && error.message) return error.message;
	return table.e2ee_failed;
}

// ---------------------------------------------------------------------------------------------
// Text of an opened message

/** Shown URL: no scheme or `www.`, cut at 32 characters. */
export function shortUrl(url) {
	const shown = url.replace(/^https?:\/\/(www\.)?/, '').replace(/\/$/, '');
	return shown.length > 32 ? `${shown.slice(0, 31)}…` : shown;
}

const TOKEN =
	/(https?:\/\/[^\s<>"]+[^\s<>".,:;'!?)\]])|(^|[^\w@])@([A-Za-z0-9_]{1,15})(?:@([A-Za-z0-9-]+(?:\.[A-Za-z0-9-]+)*(?::\d{1,5})?))?(?![\w@])|(^|[^\p{L}\p{N}_&])#([\p{L}\p{N}_]{1,50})/gu;

/** Splits plain text into `{type: 'text'|'link'|'mention'|'hashtag', ...}` segments, as the server does for posts. */
export function parseText(text) {
	const out = [];
	const plain = (s) => {
		if (!s) return;
		const last = out.at(-1);
		if (last?.type === 'text') last.text += s;
		else out.push({ type: 'text', text: s });
	};
	let last = 0;
	for (const hit of text.matchAll(TOKEN)) {
		plain(text.slice(last, hit.index));
		if (hit[1]) out.push({ type: 'link', url: hit[1] });
		else if (hit[3]) {
			plain(hit[2]);
			out.push({ type: 'mention', handle: hit[3], domain: hit[4]?.toLowerCase() ?? null });
		} else {
			plain(hit[5]);
			out.push({ type: 'hashtag', tag: hit[6].toLowerCase(), text: hit[6] });
		}
		last = hit.index + hit[0].length;
	}
	plain(text.slice(last));
	return out;
}

const link = (href, label, external) => {
	const a = document.createElement('a');
	a.href = href;
	a.textContent = label;
	if (external) {
		a.target = '_blank';
		a.rel = 'noopener noreferrer nofollow';
	}
	return a;
};

/** The nodes of `text` as a post shows them: plain runs, mentions, hashtags and links. */
export function richNodes(text) {
	return parseText(text).map((segment) => {
		switch (segment.type) {
			case 'mention': {
				const at = segment.domain ? `@${segment.handle}@${segment.domain}` : `@${segment.handle}`;
				return link(`/${at}`, at, false);
			}
			case 'hashtag':
				return link(`/tags/${encodeURIComponent(segment.tag)}`, `#${segment.text}`, false);
			case 'link':
				return link(segment.url, shortUrl(segment.url), true);
			default:
				return document.createTextNode(segment.text);
		}
	});
}

// ---------------------------------------------------------------------------------------------
// The page

const toast = (detail) => document.dispatchEvent(new CustomEvent('tuita-toast', { detail }));

async function api(method, path, body) {
	let res;
	try {
		res = await fetch(`/api/v1${path}`, {
			method,
			credentials: 'same-origin',
			headers: body === undefined ? { accept: 'application/json' } : { accept: 'application/json', 'content-type': 'application/json' },
			body: body === undefined ? undefined : JSON.stringify(body)
		});
	} catch {
		throw new Error(strings().composer_send_failed ?? 'Network error');
	}
	if (!res.ok) {
		let message = res.statusText || `HTTP ${res.status}`;
		try {
			const failure = await res.json();
			if (typeof failure.error === 'string' && failure.error) message = failure.error;
		} catch {
			// Non-JSON error bodies fall back to the status line.
		}
		throw Object.assign(new Error(message), { status: res.status });
	}
	return res.json();
}

const serverApi = {
	current: () => api('GET', '/me/message-key'),
	put: (body) => api('PUT', '/me/message-key', body)
};

export const keys = new Keys(browserStore(), serverApi, () => {
	if (typeof document !== 'undefined') document.dispatchEvent(new CustomEvent('tuita-e2ee'));
});

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

/** Runs `run` now, and again after every change inside `root`, of its key attributes and of the keys; one run at a time. */
function watch(root, run) {
	let running = false;
	let again = false;
	const go = async () => {
		if (running) {
			again = true;
			return;
		}
		running = true;
		try {
			do {
				again = false;
				await run();
			} while (again && root.isConnected);
		} catch {
			// The next change tries again.
		} finally {
			running = false;
		}
	};
	const observer = new MutationObserver(() => void go());
	observer.observe(root, { childList: true, subtree: true, attributes: true, attributeFilter: ['data-peer-key', 'data-mode'] });
	const onchange = () => void go();
	document.addEventListener('tuita-e2ee', onchange);
	void go();
	return () => {
		observer.disconnect();
		document.removeEventListener('tuita-e2ee', onchange);
	};
}

const shown = new WeakMap();

/** Puts `line` into a bubble (`p.bubble[data-ciphertext]`) or a preview (`[data-preview]`). */
function showLine(el, line) {
	if (shown.get(el) === line) return;
	shown.set(el, line);
	const text = el.querySelector('[data-text]');
	const holder = el.querySelector('[data-ph]');
	const lock = el.querySelector('[data-lock]');
	const opened = 'text' in line;
	if (opened) text.replaceChildren(...richNodes(line.text));
	else if (holder) holder.textContent = placeholderText(line.placeholder);
	text.hidden = !opened;
	if (holder) holder.hidden = opened;
	if (lock) lock.hidden = opened;
	el.classList.toggle('hidden', !opened);
}

const messageOf = (el) => ({ id: el.dataset.mid, text: '', ciphertext: el.dataset.ciphertext, mine: el.dataset.mine === 'true' });

function mountLines(root) {
	return watch(root, async () => {
		const me = root.dataset.me;
		const peer = root.dataset.peerId;
		if (!me || !peer) return;
		await keys.ensure(me);
		const key = root.dataset.peerKey || null;
		let trusted = [];
		try {
			if (keys.status === 'ready' && root.dataset.mode === 'sealed' && key) await keys.observe(peer, key);
			trusted = keys.status === 'ready' ? await keys.trustedFor(peer) : [];
		} catch {
			trusted = [];
		}
		for (const el of root.querySelectorAll('[data-ciphertext]')) {
			showLine(el, await keys.line(messageOf(el), peer, trusted));
		}
	});
}

function mountPreviews(root) {
	return watch(root, async () => {
		const me = root.dataset.me;
		if (!me) return;
		await keys.ensure(me);
		for (const el of root.querySelectorAll('[data-preview]')) {
			const opened = await keys.preview({
				user: { id: el.dataset.peerId },
				peerKey: el.dataset.peerKey || null,
				last: messageOf({ dataset: { mid: el.dataset.mid, ciphertext: el.dataset.preview, mine: el.dataset.mine } })
			});
			const holder = el.querySelector('[data-ph]');
			const text = el.querySelector('[data-text]');
			if (opened === null) {
				text.hidden = true;
				if (holder) holder.hidden = false;
			} else if (text.textContent !== opened) {
				text.textContent = opened;
				text.hidden = false;
				if (holder) holder.hidden = true;
			}
		}
	});
}

function mountSettings(root) {
	const me = root.dataset.me;
	const apply = () => {
		for (const el of root.querySelectorAll('[data-when]')) el.hidden = !el.dataset.when.split(' ').includes(keys.status);
	};
	document.addEventListener('tuita-e2ee', apply);
	apply();
	if (me) void keys.ensure(me);
	return () => document.removeEventListener('tuita-e2ee', apply);
}

const fieldMessage = (input, message) => {
	const field = input.closest('.field');
	if (!field) return;
	field.querySelector('.message.error')?.remove();
	const hint = field.querySelector('.message');
	if (message) {
		const p = document.createElement('p');
		p.className = 'message error';
		p.id = `${input.id}-error`;
		p.textContent = message;
		field.append(p);
		field.classList.add('invalid');
		input.setAttribute('aria-invalid', 'true');
		input.setAttribute('aria-describedby', p.id);
		if (hint) hint.hidden = true;
	} else {
		field.classList.remove('invalid');
		input.removeAttribute('aria-invalid');
		if (hint) {
			hint.hidden = false;
			input.setAttribute('aria-describedby', hint.id);
		} else input.removeAttribute('aria-describedby');
	}
};

const submitters = new WeakSet();

const fields = (prompt) => prompt.querySelectorAll('input[type="password"]');

/** Shows the short or mismatch problem under the first field; returns it. */
function showProblem(prompt) {
	if (prompt.dataset.prompt === 'unlock') return null;
	const [first, second] = fields(prompt);
	const problem = passphraseProblem(first.value, second.value);
	const table = strings();
	fieldMessage(first, problem === 'short' ? table.e2ee_passphrase_short : problem === 'mismatch' ? table.e2ee_passphrase_mismatch : '');
	return problem;
}

async function submitPrompt(form) {
	const prompt = form.closest('[data-prompt]');
	const kind = prompt.dataset.prompt;
	const [first, second] = fields(prompt);
	const failed = kind === 'unlock' ? first : (second ?? first);
	const table = strings();
	prompt.dataset.touched = '';
	const problem = showProblem(prompt);
	const button = form.querySelector('button[type="submit"]');
	if (submitters.has(form) || problem || !first.value) return;
	submitters.add(form);
	button.disabled = true;
	button.setAttribute('aria-busy', 'true');
	fieldMessage(failed, '');
	try {
		await keys[kind](first.value);
		first.value = '';
		if (second) second.value = '';
		delete prompt.dataset.touched;
		if (prompt.dataset.done) toast(table[prompt.dataset.done]);
		form.closest('dialog')?.close();
	} catch (error) {
		fieldMessage(failed, keyErrorText(error, table) || table.e2ee_failed);
	} finally {
		submitters.delete(form);
		button.disabled = false;
		button.removeAttribute('aria-busy');
	}
}

if (typeof document !== 'undefined' && !document.documentElement.dataset.e2eeBound) {
	document.documentElement.dataset.e2eeBound = '';
	globalThis.tuitaE2ee = Object.freeze({
		keys,
		conversationState,
		sendMessage,
		safetyNumber,
		blockText,
		keyErrorText,
		blockedStates: ['setup', 'unlock', 'changed']
	});

	mount('[data-e2ee-lines]', mountLines);
	mount('[data-e2ee-previews]', mountPreviews);
	mount('[data-island="e2ee-settings"]', mountSettings);

	document.addEventListener('submit', (event) => {
		const form = event.target instanceof Element ? event.target.closest('[data-prompt] form') : null;
		if (!form) return;
		event.preventDefault();
		void submitPrompt(form);
	});
	document.addEventListener('input', (event) => {
		const prompt = event.target instanceof Element ? event.target.closest('[data-prompt][data-touched]') : null;
		if (prompt) showProblem(prompt);
	});

	document.addEventListener('click', (event) => {
		const button = event.target instanceof Element ? event.target.closest('[data-act^="e2ee-"]') : null;
		if (!button) return;
		const table = strings();
		if (button.dataset.act === 'e2ee-retry') {
			if (keys.userId) void keys.load(keys.userId);
		} else if (button.dataset.act === 'e2ee-forget') {
			keys
				.forget()
				.then(() => toast(table.e2ee_forget_done))
				.catch((error) => toast(keyErrorText(error, table)));
		}
	});

	// Signing out removes the account's keys from the device first: the click is held back until
	// the store has dropped them (at most a second), then replayed for the control's own handler.
	const released = new WeakSet();
	document.addEventListener(
		'click',
		(event) => {
			const control = event.target instanceof Element ? event.target.closest('[data-forget-keys]') : null;
			if (!control || released.has(control)) return;
			event.stopImmediatePropagation();
			event.preventDefault();
			const ids = (control.dataset.forgetKeys ?? '').split(/\s+/).filter(Boolean);
			const release = () => {
				if (released.has(control)) return;
				released.add(control);
				control.click();
				released.delete(control);
			};
			setTimeout(release, 1000);
			forgetAccounts(ids).finally(release);
		},
		true
	);
}
