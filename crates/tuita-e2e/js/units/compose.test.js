// Known-answer vectors for assets/js/composer.js: poll drafts, post length and more. (`attachBlock` lives with the media tray: see upload.test.js.)
const eq = (actual, expected, what) => {
	if (JSON.stringify(actual) !== JSON.stringify(expected)) {
		throw new Error(`${what}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
	}
};

export async function run(mod) {
	// poll drafts
	const draft = { ...mod.newPoll(), options: [' Yes ', '', 'yes', 'No'] };
	eq(mod.pollInput(draft), { options: ['Yes', 'yes', 'No'], multiple: false, durationHours: 24 }, 'blank answers drop, the rest is trimmed');
	eq(mod.pollReady(draft), false, 'Yes and yes are the same answer');
	eq(mod.pollReady({ ...draft, options: ['Yes', 'No'] }), true, 'two different answers');
	eq(mod.pollReady(mod.newPoll()), false, 'a new poll has no answers');
	eq(mod.pollReady({ ...draft, options: ['a', 'b', 'c', 'd', 'e'] }), false, 'five answers are too many');

	// post bodies
	const base = { text: ' hi ', cw: ' ', sensitive: true, policy: 'everyone', poll: null };
	const plain = mod.postBody({ ...base, mediaIds: [] });
	eq([plain.text, plain.cw, plain.sensitive], ['hi', '', false], 'sensitive needs media');
	const withMedia = mod.postBody({ ...base, mediaIds: ['m1'] });
	eq([withMedia.sensitive, withMedia.mediaIds], [true, ['m1']], 'sensitive with media');
	eq(
		mod.postBody({ ...base, mediaIds: [], replyTo: 'p1', quoteOf: 'p2' }),
		{ text: 'hi', cw: '', sensitive: false, replyTo: 'p1', quoteOf: 'p2', replyPolicy: 'everyone', mediaIds: [], poll: null },
		'the full request body'
	);
	eq(
		mod.extraBody(' more ', 'following'),
		{ text: 'more', cw: '', sensitive: false, replyPolicy: 'following', mediaIds: [] },
		'thread continuations are text only'
	);

	// counting
	eq(mod.postLength('😀😀'), 2, 'scalar values');
	eq(mod.postLength('a https://example.com/very/long/path/that/goes/on b'), 4 + 23, 'a URL counts 23');
	eq(mod.postLength('see https://example.com/a. ok'), 4 + 23 + 4, 'trailing punctuation is not part of the URL');
}
