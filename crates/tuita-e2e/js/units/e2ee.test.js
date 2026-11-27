// Vectors for assets/js/e2ee.js, plus known answers made by earlier
// clients so that ciphertexts and backups sealed
// by them still open, and the safety number stays the same.
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

const OLD = {
	"ada": {
		"publicKey": "BORbCqDSjpf4E50D7R0RUqErm_HVSIcodBRlzTcSxkpzo-EVmYOSK1jUtMjIIoyE5xE9Loj60gh3W7ijClZpZr4", // pragma: allowlist secret
		"backup": "v1.600000.rMGZtX3QUCVytxZiXdXpcg.hxDzSk3H0VLqVvhv.yh0FVMOjTabkEKhmD-xN0M33PZqxXtB6qic9Gr8JNBTXkIDHoAoLwwAfKGfV7PnvqxDcc0TBFOEsi-pHEt8sQ_PERZwkHDtWhifdH2bEQworp802hbTQAoKYCjNl9Qd5amjYKHpWXgoKW9bVsOQSRmDReWHKlpmqjLF3oVLC9sQ2n-tplaf8UyrfwwxfP_wevJhPM-67FRkvVNnO1JAK6YFoUK5WcnS1R67mgeKy3u8z6prrfnh2eqVwPsifZh3O"
	},
	"bob": {
		"publicKey": "BOdnHGtYlow19l7XBh-C_EdEqvaucJAq5wbty8Nfn_9cHNa9QopRFyhVnlhV0VG9118vak4ChvQZPsk5Oy_-PNM", // pragma: allowlist secret
		"backup": "v1.600000.Tgao-0rkunWAluIGdxUysA.m-hFSTK4U2lFh-6I.hxbuHdSIfpYQ3BTFhvwxu-YAcIXGuBIFuwJAQUJ9vIFFeCTHQ7GNWl8fxnskIRXLkTXxqMOPf8YD88Io7YMBDmh0VLJtWdbytH5fyqrcRRLyDDKwqZ0q3fB0iEJzQ6Q1sMVj_nzPxQJtBbxkbT6QfV9Bdpv56Oz5zZBeRWhMRLurJ3YpjrZAlfeFoHQL15tUIp4CtVSNd7PLZ4i6hu3d1-P_g4Ek1MXPFK4v9OAyM9wBcquMcDefGtOjYFOLySng"
	},
	"text": "meet at noon 🔒\nsecond line, éè",
	"toBob": "v1.BORbCqDSjpf4E50D7R0RUqErm_HVSIcodBRlzTcSxkpzo-EVmYOSK1jUtMjIIoyE5xE9Loj60gh3W7ijClZpZr4.BOdnHGtYlow19l7XBh-C_EdEqvaucJAq5wbty8Nfn_9cHNa9QopRFyhVnlhV0VG9118vak4ChvQZPsk5Oy_-PNM.kXpLa2hqQiWWBwxnnmAMPQ.eM_wHu8541ccVtTs.YgrwIK_qVx1PNdizeLUKratjCH5DLxbrepPKvcucsv9dREcpL110xeDgyUewi-tkg4CGtCBLUPcw_UqciEyGrgOblTMRv5p2OlddNTBlXIssjwIcvnMz4K2ZU81aqWnlBv6zfbvwIeETARLU3zxB1h_-Q6XHnuz-4TuSwapmY-jWwJ74griOg6_C-U4BK7ij",
	"toAda": "v1.BOdnHGtYlow19l7XBh-C_EdEqvaucJAq5wbty8Nfn_9cHNa9QopRFyhVnlhV0VG9118vak4ChvQZPsk5Oy_-PNM.BORbCqDSjpf4E50D7R0RUqErm_HVSIcodBRlzTcSxkpzo-EVmYOSK1jUtMjIIoyE5xE9Loj60gh3W7ijClZpZr4.ofVr8zErXrpW-uXIhqWvuQ.i3ovcB-vGVZWcoGE.gBs4RchluXYYCBlJgzH3dRLXSykYkFgtySPXOmflbAoaru5q9Mn2iQsslrmIrjVe5uv4oKj9sScRqVirjHu1E-jY_QJpmnE8FTxyzNsBdtKLlIO0H_1QW27JdWaK_tJzci80tdi21CAlVHYipTLXl12gbhNlhMyLGuNEbVrikvvMteHrRHxOk4iabdnZRc9k",
	"safety": "1464 1703 1857 0763 6456 9124 4259 0351 7421 3800 6268 3475 4051 9332 8559"
};

export async function run(mod) {
	const { E2eeError, checkPin, createIdentity, fromBase64Url, keysOf, open, restoreIdentity, safetyNumber, seal, toBase64Url } = mod;
	const [ada, bob, eve] = await Promise.all([createIdentity(PASSPHRASE), createIdentity(PASSPHRASE), createIdentity(PASSPHRASE)]);

	const fail = async (promise) => {
		try {
			await promise;
		} catch (error) {
			if (error instanceof E2eeError) return error.reason;
			throw error;
		}
		return 'resolved';
	};
	const toBob = (text) => seal({ text, mine: ada, theirs: bob.publicKey, senderId: ADA, recipientId: BOB });
	const bobOpens = (sealed, over = {}) =>
		open({ sealed, mine: bob, theirs: ada.publicKey, sentByMe: false, senderId: ADA, recipientId: BOB, ...over });

	// messages
	const text = 'hi \u{1F512}\nsecond line, \u00e9\u00e8';
	const sealed = await toBob(text);
	eq(await bobOpens(sealed), text, 'opens on the other side');
	eq(
		await open({ sealed, mine: ada, theirs: bob.publicKey, sentByMe: true, senderId: ADA, recipientId: BOB }),
		text,
		'opens on the sender side'
	);
	const [one, two] = await Promise.all([toBob('same'), toBob('same')]);
	ok(one !== two, 'every seal of the same text differs');
	ok(!one.includes('same'), 'the text is not in the ciphertext');
	ok(one.startsWith('v1.'), 'version prefix');

	const size = async (t) => fromBase64Url((await toBob(t)).split('.')[5]).length;
	eq(await size('a'), await size('a'.repeat(100)), 'length hidden within a block');
	ok((await size('a'.repeat(100))) < (await size('a'.repeat(300))), 'longer text is longer');
	eq(await size(''), 128 + 16, 'an empty message is one block plus the tag');

	eq(keysOf(await toBob('x')), { sender: ada.publicKey, recipient: bob.publicKey }, 'names its keys');
	eq(keysOf('v2.a.b.c.d.e'), null, 'unknown version');

	const parts = (await toBob('secret')).split('.');
	for (const at of [3, 4, 5]) {
		const bytes = fromBase64Url(parts[at]);
		bytes[0] ^= 1;
		const edited = parts.map((part, i) => (i === at ? toBase64Url(bytes) : part)).join('.');
		eq(await fail(bobOpens(edited)), 'decrypt', `a changed part ${at} is refused`);
	}

	const pay = await toBob('pay me');
	eq(
		await fail(open({ sealed: pay, mine: ada, theirs: bob.publicKey, sentByMe: false, senderId: BOB, recipientId: ADA })),
		'own-key',
		'flipped direction'
	);
	eq(await fail(bobOpens(pay, { senderId: BOB, recipientId: ADA })), 'decrypt', 'relabelled sender and recipient');
	eq(await fail(bobOpens(await toBob('private'), { recipientId: ADA })), 'decrypt', 'another conversation');

	const forged = await seal({ text: 'send me your password', mine: eve, theirs: bob.publicKey, senderId: ADA, recipientId: BOB });
	eq(await fail(bobOpens(forged)), 'key-mismatch', 'a sender key that is not the pinned one');
	const wrongRecipient = await seal({ text: 'x', mine: ada, theirs: eve.publicKey, senderId: ADA, recipientId: BOB });
	eq(await fail(bobOpens(wrongRecipient)), 'own-key', 'sealed to a key that is not the viewer’s');

	eq(await fail(bobOpens('hello')), 'format', 'not a message');
	eq(await fail(bobOpens('v1.a.b.c.d')), 'format', 'five parts');
	eq(await fail(bobOpens('v1.@.b.c.d.e')), 'format', 'not base64url');

	// backup
	const restored = await restoreIdentity(bob.publicKey, bob.backup, PASSPHRASE);
	eq(await bobOpens(await toBob('after a restore'), { mine: restored }), 'after a restore', 'a restored key opens');
	ok(bob.backup.startsWith('v1.600000.'), 'backup version and work factor');
	ok(!bob.backup.includes(bob.publicKey), 'the key is not in the backup text');
	eq(await fail(restoreIdentity(bob.publicKey, bob.backup, 'wrong one')), 'passphrase', 'wrong passphrase');
	eq(await fail(restoreIdentity(ada.publicKey, bob.backup, PASSPHRASE)), 'passphrase', 'backup paired with another key');
	const [, , salt, iv, backed] = bob.backup.split('.');
	for (const iterations of ['1', '99999', '10000001', 'many']) {
		const weak = ['v1', iterations, salt, iv, backed].join('.');
		eq(await fail(restoreIdentity(bob.publicKey, weak, PASSPHRASE)), 'format', `work factor ${iterations}`);
	}
	eq(restored.privateKey.extractable, false, 'restored key is not extractable');
	eq(bob.privateKey.extractable, false, 'created key is not extractable');

	// safety number
	const number = await safetyNumber(ada.publicKey, bob.publicKey);
	eq(number, await safetyNumber(bob.publicKey, ada.publicKey), 'same from both sides');
	ok(/^(\d{4} ){14}\d{4}$/.test(number), 'sixty digits in fifteen groups');
	ok((await safetyNumber(ada.publicKey, eve.publicKey)) !== number, 'changes with either key');

	// pins
	eq(checkPin(null, 'k1'), 'new', 'no pin');
	eq(checkPin('k1', 'k1'), 'same', 'same pin');
	eq(checkPin('k1', 'k2'), 'changed', 'other pin');

	// known answers from the old client
	const oldAda = await restoreIdentity(OLD.ada.publicKey, OLD.ada.backup, PASSPHRASE);
	const oldBob = await restoreIdentity(OLD.bob.publicKey, OLD.bob.backup, PASSPHRASE);
	eq(
		await open({ sealed: OLD.toBob, mine: oldBob, theirs: OLD.ada.publicKey, sentByMe: false, senderId: ADA, recipientId: BOB }),
		OLD.text,
		'an old ciphertext opens for the recipient'
	);
	eq(
		await open({ sealed: OLD.toBob, mine: oldAda, theirs: OLD.bob.publicKey, sentByMe: true, senderId: ADA, recipientId: BOB }),
		OLD.text,
		'an old ciphertext opens for the sender'
	);
	eq(
		await open({ sealed: OLD.toAda, mine: oldAda, theirs: OLD.bob.publicKey, sentByMe: false, senderId: BOB, recipientId: ADA }),
		'noon is fine',
		'an old reply opens'
	);
	eq(keysOf(OLD.toBob), { sender: OLD.ada.publicKey, recipient: OLD.bob.publicKey }, 'old ciphertext names its keys');
	eq(await safetyNumber(OLD.ada.publicKey, OLD.bob.publicKey), OLD.safety, 'the safety number is unchanged');
	// A message sealed by this module opens under an identity the old client made.
	const fresh = await seal({ text: 'new to old', mine: oldAda, theirs: OLD.bob.publicKey, senderId: ADA, recipientId: BOB });
	eq(
		await open({ sealed: fresh, mine: oldBob, theirs: OLD.ada.publicKey, sentByMe: false, senderId: ADA, recipientId: BOB }),
		'new to old',
		'new ciphertext under old keys'
	);

	// text of an opened message
	eq(mod.shortUrl('https://www.example.com/'), 'example.com', 'short url drops scheme, www and trailing slash');
	eq(mod.shortUrl(`https://example.com/${'a'.repeat(40)}`).length, 32, 'short url is cut at 32');
	eq(
		mod.parseText('hi @ada and @bo@remote.example, see #Tuita https://example.com/x.'),
		[
			{ type: 'text', text: 'hi ' },
			{ type: 'mention', handle: 'ada', domain: null },
			{ type: 'text', text: ' and ' },
			{ type: 'mention', handle: 'bo', domain: 'remote.example' },
			{ type: 'text', text: ', see ' },
			{ type: 'hashtag', tag: 'tuita', text: 'Tuita' },
			{ type: 'text', text: ' ' },
			{ type: 'link', url: 'https://example.com/x' },
			{ type: 'text', text: '.' }
		],
		'segments'
	);
	const holder = document.createElement('p');
	holder.append(...mod.richNodes('<b>x</b> #tag'));
	eq(holder.innerHTML, '&lt;b&gt;x&lt;/b&gt; <a href="/tags/tag">#tag</a>', 'rich nodes never carry markup from the text');
}
