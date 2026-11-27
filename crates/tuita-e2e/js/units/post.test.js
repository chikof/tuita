// Known-answer vectors for assets/js/post.js: the optimistic toggles, the post length and the
// plural client strings.
const eq = (actual, expected, what) => {
	if (JSON.stringify(actual) !== JSON.stringify(expected)) {
		throw new Error(`${what}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
	}
};

export async function run(mod) {
	const { flip, stateOf, postLength, pluralText, fill, FLAGS } = mod;
	const base = { liked: false, likes: 2, reposted: false, reposts: 5, bookmarked: false };

	// optimistic toggle
	eq(flip(base, 'like'), { ...base, liked: true, likes: 3 }, 'liking raises the count');
	eq(flip({ ...base, liked: true }, 'like'), { ...base, liked: false, likes: 1 }, 'unliking lowers it');
	eq(flip(base, 'repost'), { ...base, reposted: true, reposts: 6 }, 'reposting raises the count');
	eq(flip(base, 'bookmark'), { ...base, bookmarked: true }, 'a bookmark has no counter');
	eq(Object.keys(FLAGS), ['like', 'repost', 'bookmark'], 'the three toggles');
	eq(
		stateOf({
			viewer: { liked: true, reposted: false, bookmarked: true },
			counts: { likes: 4, reposts: 2, quotes: 3, replies: 0, views: 0 }
		}),
		{ liked: true, likes: 4, reposted: false, reposts: 5, bookmarked: true },
		'reposts and quotes are shown together'
	);

	// postLength
	eq(postLength('hello'), 5, 'plain text');
	eq(postLength('see https://example.com/a/very/long/path?x=1 now'), 'see  now'.length + 23, 'a link counts 23');
	eq(postLength('😀😀'), 2, 'code points, not UTF-16 units');

	// the plural client strings
	const entry = { 0: '{count} left', 1: '{count} left (one)', 2: '{count} left (two)', 5: '{count} left (many)', 1000000: '{count} left (other)' };
	eq(pluralText(entry, 1, 'en'), '1 left (one)', 'English one');
	eq(pluralText(entry, 7, 'en'), '7 left', 'English other uses a sample of the same category');
	eq(pluralText('Like ({count})', 3, 'en'), 'Like (3)', 'a plain string');
	eq(fill('Muted {handle}', { handle: '@a' }), 'Muted @a', 'fill');
	eq(fill('Muted {handle}', {}), 'Muted {handle}', 'unknown slots stay');
}
