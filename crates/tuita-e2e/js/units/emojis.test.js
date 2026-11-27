// Known-answer vectors for assets/js/composer.js. (The Unicode emoji list
// is a Rust constant: `compose::emoji` tests it.)
const eq = (actual, expected, what) => {
	if (JSON.stringify(actual) !== JSON.stringify(expected)) {
		throw new Error(`${what}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
	}
};

const emote = (shortcode, category = null) => ({ shortcode, url: `/media/${shortcode}.png`, category });

export async function run(mod) {
	// emoteQuery
	eq(mod.emoteQuery(':par'), 'par', 'after a colon');
	eq(mod.emoteQuery('hello :Party_9'), 'Party_9', 'mixed case and digits');
	eq(mod.emoteQuery('(:ok'), 'ok', 'after a bracket');
	eq(mod.emoteQuery('line\n:ab'), 'ab', 'after a newline');
	eq(mod.emoteQuery('hi :'), '', 'a bare colon');
	for (const before of ['see https://example.com', 'at 12:30', '12:30:45', 'a:bc', '/:abc', ':abc: ', ':abc:de']) {
		eq(mod.emoteQuery(before), null, `${before} is not an emote`);
	}
	eq(mod.emoteQuery(`:${'a'.repeat(32)}`), 'a'.repeat(32), '32 characters');
	eq(mod.emoteQuery(`:${'a'.repeat(33)}`), null, '33 characters');

	// rankEmotes
	const list = ['sad_parrot', 'party_parrot', 'parrot', 'park', 'apart'].map((s) => emote(s));
	eq(mod.rankEmotes(list, 'par').map((e) => e.shortcode), ['party_parrot', 'parrot', 'park', 'sad_parrot', 'apart'], 'prefix matches first');
	eq(mod.rankEmotes([emote('Party_Parrot')], 'PARTY').length, 1, 'upper case query');
	eq(mod.rankEmotes([emote('party')], 'Pa').length, 1, 'upper case shortcode');
	const many = Array.from({ length: 12 }, (_, i) => emote(`blob_${i}`));
	eq(mod.rankEmotes(many, 'blob').length, 8, 'at most eight');
	eq(mod.rankEmotes(many, 'blob')[0].shortcode, 'blob_0', 'list order is kept');
	eq(mod.rankEmotes(list, 'p'), [], 'one letter is too few');
	eq(mod.rankEmotes(list, ''), [], 'no letters');

	// emoteInsertion
	eq(mod.emoteInsertion(':blob:', undefined), ':blob: ', 'a space at the end');
	eq(mod.emoteInsertion(':blob:', 'x'), ':blob: ', 'a space before a letter');
	eq(mod.emoteInsertion(':blob:', ' '), ':blob:', 'a space already follows');
	eq(mod.emoteInsertion(':blob:', '\n'), ':blob:', 'a newline already follows');
	eq(mod.emoteInsertion('🔥', 'x'), '🔥', 'Unicode emoji stay as they are');

	// groupEmotes
	const groups = mod.groupEmotes([emote('a', 'Cats'), emote('b', 'Cats'), emote('c', 'Dogs'), emote('d'), emote('e', '')]);
	eq(groups.map((g) => [g.category, g.items.map((e) => e.shortcode)]), [[null, ['d', 'e']], ['Cats', ['a', 'b']], ['Dogs', ['c']]], 'uncategorised first');
	eq(mod.groupEmotes([emote('a', 'Cats')]).map((g) => g.category), ['Cats'], 'no empty uncategorised section');
	eq(mod.groupEmotes([]), [], 'no emotes, no groups');
}
