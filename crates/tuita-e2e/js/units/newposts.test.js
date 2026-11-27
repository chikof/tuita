// Known-answer vectors for assets/js/newposts.js: the URLs it asks the server for.
const eq = (actual, expected, what) => {
	if (JSON.stringify(actual) !== JSON.stringify(expected)) {
		throw new Error(`${what}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
	}
};

export async function run(mod) {
	eq(mod.POLL_MS, 30_000, 'polling every 30 s');
	eq(mod.pillUrl('following', 'a b'), '/_tuita/new-posts?feed=following&since=a%20b', 'the pill url is encoded');
	eq(
		mod.pillUrl('foryou'),
		'/_tuita/new-posts?feed=foryou&since=00000000-0000-0000-0000-000000000000',
		'an empty timeline counts from the nil id'
	);
	eq(mod.feedUrl('/', 'foryou'), '/?feed=foryou', 'the first page of a feed is the home page');
}
