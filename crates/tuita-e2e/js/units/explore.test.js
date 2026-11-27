// Known-answer vectors for assets/js/explore.js: withRecent and the recent-search storage.
const eq = (actual, expected, what) => {
	if (JSON.stringify(actual) !== JSON.stringify(expected)) {
		throw new Error(`${what}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
	}
};

const storage = (items) => ({ getItem: (key) => (key in items ? items[key] : null) });

export async function run(mod) {
	eq(mod.withRecent(['b', 'Rust', 'c'], 'rust'), ['rust', 'b', 'c'], 'newest first, case-insensitive duplicate dropped');
	const nine = ['1', '2', '3', '4', '5', '6', '7', '8'];
	eq(mod.withRecent(nine, '0'), ['0', '1', '2', '3', '4', '5', '6', '7'], 'eight kept');

	eq(mod.recentKey('u1'), 'tuita:recent-searches:u1', 'the key is per account');
	eq(mod.readRecent(storage({ k: '["a","b"]' }), 'k'), ['a', 'b'], 'a stored list');
	eq(mod.readRecent(storage({}), 'k'), [], 'nothing stored');
	eq(mod.readRecent(storage({ k: 'not json' }), 'k'), [], 'damaged JSON');
	eq(mod.readRecent(storage({ k: '{"a":1}' }), 'k'), [], 'not a list');
	eq(mod.readRecent(storage({ k: '["a",1,null,"b"]' }), 'k'), ['a', 'b'], 'only strings');
	const many = JSON.stringify(Array.from({ length: 12 }, (_, i) => String(i)));
	eq(mod.readRecent(storage({ k: many }), 'k').length, 8, 'at most eight');
}
