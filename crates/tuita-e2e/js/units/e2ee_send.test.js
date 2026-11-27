// Vectors for sendMessage in assets/js/e2ee.js.
const eq = (actual, expected, what) => {
	if (JSON.stringify(actual) !== JSON.stringify(expected)) {
		throw new Error(`${what}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
	}
};
const ok = (value, what) => {
	if (!value) throw new Error(what);
};
const rejectsWith = async (promise, status, what) => {
	try {
		await promise;
	} catch (error) {
		eq(error.status, status, what);
		return;
	}
	throw new Error(`${what}: did not reject`);
};

const ME = 'aaaaaaaa-0000-7000-8000-000000000001';
const PEER = 'bbbbbbbb-0000-7000-8000-000000000002';

export async function run(mod) {
	const { createIdentity, open, sendMessage } = mod;
	const [mine, bob, bobAgain] = await Promise.all([
		createIdentity('correct horse battery staple'),
		createIdentity('correct horse battery staple'),
		createIdentity('correct horse battery staple')
	]);
	const message = (body) => ({
		id: '1',
		text: body.text ?? '',
		ciphertext: body.ciphertext ?? null,
		createdAt: '2026-11-11T00:00:00Z',
		mine: true,
		readAt: null,
		emojis: []
	});
	const conflict = () => Object.assign(new Error('Encryption keys changed. Reload and try again.'), { status: 409 });
	const sealedView = (peerKey) => ({ mode: 'sealed', blockedReason: null, peerKey });

	function setup(options) {
		const posted = [];
		const pins = [];
		let pinned = options.pinned ?? null;
		let fetched = 0;
		let rejections = options.rejectFirst ? 1 : 0;
		const deps = {
			me: { id: ME, identity: mine, keys: options.keys ?? 'ready' },
			peer: { id: PEER, remote: options.remote ?? false },
			pinned: async () => pinned,
			pin: async (key) => {
				pins.push(key);
				pinned = key;
			},
			post: async (body) => {
				posted.push(body);
				if (rejections-- > 0) throw conflict();
				return message(body);
			},
			fetchView: async () => {
				fetched++;
				return options.refreshed ?? options.view;
			}
		};
		return { deps, posted, pins, fetched: () => fetched };
	}

	// seals to the peer key and posts only the ciphertext
	let run = setup({ view: sealedView(bob.publicKey), pinned: bob.publicKey });
	let result = await sendMessage('hello', sealedView(bob.publicKey), run.deps);
	ok(result.sent, 'sent');
	eq(run.posted.length, 1, 'one post');
	eq(Object.keys(run.posted[0]), ['ciphertext'], 'only the ciphertext leaves');
	eq(
		await open({ sealed: run.posted[0].ciphertext ?? '', mine: bob, theirs: mine.publicKey, sentByMe: false, senderId: ME, recipientId: PEER }),
		'hello',
		'the peer opens it'
	);

	// pins a key seen for the first time before sealing
	run = setup({ view: sealedView(bob.publicKey), pinned: null });
	await sendMessage('hi', sealedView(bob.publicKey), run.deps);
	eq(run.pins, [bob.publicKey], 'first key pinned');

	// plain to a remote person
	const plainView = { mode: 'plain', blockedReason: null, peerKey: null };
	run = setup({ view: plainView, remote: true });
	await sendMessage('hi', plainView, run.deps);
	eq(run.posted, [{ text: 'hi' }], 'plain text to a remote person');

	// never downgrades
	run = setup({ view: plainView, pinned: bob.publicKey });
	result = await sendMessage('hi', plainView, run.deps);
	eq(run.posted, [], 'nothing posted after a pin (local)');
	eq([result.sent, result.state.block.kind], [false, 'downgrade'], 'downgrade refused (local)');
	run = setup({ view: plainView, remote: true, pinned: bob.publicKey });
	result = await sendMessage('hi', plainView, run.deps);
	eq(run.posted, [], 'nothing posted after a pin (labelled remote)');
	eq([result.sent, result.state.block.kind], [false, 'downgrade'], 'downgrade refused (labelled remote)');

	// changed key and locked keys do not send
	const changed = setup({ view: sealedView(bobAgain.publicKey), pinned: bob.publicKey });
	result = await sendMessage('hi', sealedView(bobAgain.publicKey), changed.deps);
	eq([result.sent, result.state.block.kind], [false, 'changed'], 'changed key blocks');
	const locked = setup({ view: sealedView(bob.publicKey), keys: 'locked' });
	result = await sendMessage('hi', sealedView(bob.publicKey), locked.deps);
	eq([result.sent, result.state.block.kind], [false, 'unlock'], 'locked keys block');
	eq([...changed.posted, ...locked.posted], [], 'nothing posted');

	// a 409 refreshes the view and retries once
	run = setup({ view: sealedView(bob.publicKey), pinned: bob.publicKey, rejectFirst: true });
	result = await sendMessage('hi', sealedView(bob.publicKey), run.deps);
	ok(result.sent, 'sent after a conflict');
	eq(run.fetched(), 1, 'view refreshed once');
	eq(run.posted.length, 2, 'posted twice');

	run = setup({ view: sealedView(bob.publicKey), pinned: bob.publicKey, rejectFirst: true, refreshed: sealedView(bobAgain.publicKey) });
	result = await sendMessage('hi', sealedView(bob.publicKey), run.deps);
	eq([result.sent, result.state.block.kind], [false, 'changed'], 'a new key in the refreshed view blocks');
	eq(run.posted.length, 1, 'only the first attempt posted');

	// gives up after the second 409
	run = setup({ view: sealedView(bob.publicKey), pinned: bob.publicKey });
	let calls = 0;
	run.deps.post = async () => {
		calls++;
		throw conflict();
	};
	await rejectsWith(sendMessage('hi', sealedView(bob.publicKey), run.deps), 409, 'second conflict');
	eq(calls, 2, 'two attempts');

	// other errors pass through
	run = setup({ view: sealedView(bob.publicKey), pinned: bob.publicKey });
	run.deps.post = async () => {
		throw Object.assign(new Error('Too many'), { status: 429 });
	};
	await rejectsWith(sendMessage('hi', sealedView(bob.publicKey), run.deps), 429, 'other errors');
	eq(run.fetched(), 0, 'no refresh for other errors');
}
