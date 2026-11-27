// Known-answer vectors for assets/js/composer.js: per-account drafts. (Recent
// searches belong to explore.js.)
const eq = (actual, expected, what) => {
	if (JSON.stringify(actual) !== JSON.stringify(expected)) {
		throw new Error(`${what}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
	}
};

const ADA = 'aaaaaaaa-0000-7000-8000-000000000001';
const BOB = 'bbbbbbbb-0000-7000-8000-000000000002';

function memory(entries) {
	const map = new Map(Object.entries(entries));
	return {
		get length() {
			return map.size;
		},
		key: (i) => [...map.keys()][i] ?? null,
		getItem: (key) => map.get(key) ?? null,
		setItem: (key, value) => void map.set(key, value),
		removeItem: (key) => void map.delete(key),
		clear: () => map.clear()
	};
}

const keys = (storage) => Array.from({ length: storage.length }, (_, i) => storage.key(i));

export async function run(mod) {
	eq(mod.draftKey(ADA, 'new'), `tuita:draft:${ADA}:new`, 'a draft key names the account and the slot');
	eq(mod.draftKey(ADA, 'r42') === mod.draftKey(BOB, 'r42'), false, 'accounts do not share drafts');

	const storage = memory({
		[mod.draftKey(ADA, 'new')]: 'a',
		[mod.draftKey(ADA, 'r1')]: 'a',
		[`tuita:recent-searches:${ADA}`]: '["x"]',
		[mod.draftKey(BOB, 'new')]: 'b',
		'tuita:prefs': '{}'
	});
	mod.purgeDrafts(storage, ADA);
	eq(keys(storage), [`tuita:recent-searches:${ADA}`, mod.draftKey(BOB, 'new'), 'tuita:prefs'], 'one account is purged, searches and others stay');

	const old = memory({
		'tuita:draft:new': 'old',
		'tuita:draft:r12': 'old',
		'tuita:draft:q9': 'old',
		'tuita:recent-searches': '["old"]',
		[mod.draftKey(ADA, 'new')]: 'kept',
		'tuita:prefs': '{}'
	});
	mod.purgeUnscopedDrafts(old);
	eq(keys(old), ['tuita:recent-searches', mod.draftKey(ADA, 'new'), 'tuita:prefs'], 'drafts written before accounts are removed');

	let ran = false;
	mod.quietly(() => {
		ran = true;
		throw new Error('storage is full');
	});
	eq(ran, true, 'quietly runs the action and swallows its failure');
}
