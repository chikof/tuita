// Known-answer vectors for assets/js/shell.js.
const assert = (ok, what) => {
	if (!ok) throw new Error(`assertion failed: ${what}`);
};
const eq = (actual, expected, what) => {
	if (JSON.stringify(actual) !== JSON.stringify(expected)) {
		throw new Error(`${what}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
	}
};

const memory = (seed = {}) => {
	const map = new Map(Object.entries(seed));
	return { getItem: (k) => map.get(k) ?? null, setItem: (k, v) => map.set(k, v), map };
};
const broken = {
	getItem() { throw new Error('denied'); },
	setItem() { throw new Error('denied'); }
};

export async function run(mod) {
	assert(mod.canGoBack('https://a.example/x', 'https://a.example') === true, 'same origin goes back');
	assert(mod.canGoBack('https://b.example/x', 'https://a.example') === false, 'other origin');
	assert(mod.canGoBack('', 'https://a.example') === false, 'empty referrer');
	assert(mod.canGoBack('not a url', 'https://a.example') === false, 'garbage referrer');

	eq(mod.nextIndex('ArrowDown', 0, 3), 1, 'down');
	eq(mod.nextIndex('ArrowDown', 2, 3), 0, 'down wraps');
	eq(mod.nextIndex('ArrowUp', 0, 3), 2, 'up wraps');
	eq(mod.nextIndex('ArrowUp', 2, 3), 1, 'up');
	eq(mod.nextIndex('ArrowUp', -1, 3), 2, 'up from nothing');
	eq(mod.nextIndex('ArrowDown', -1, 3), 0, 'down from nothing');
	eq(mod.nextIndex('Home', 2, 3), 0, 'home');
	eq(mod.nextIndex('End', 0, 3), 2, 'end');
	eq(mod.nextIndex('a', 0, 3), -1, 'other key');
	eq(mod.nextIndex('Home', 0, 0), -1, 'empty menu');

	assert(mod.dragCloses(81) && !mod.dragCloses(80) && !mod.dragCloses(0), 'drag threshold is past 80px');

	eq(mod.dismissKey('a1'), 'tuita:ann:a1', 'dismiss key');
	const store = memory();
	assert(mod.isDismissed(store, 'a1') === false, 'fresh');
	mod.dismiss(store, 'a1');
	eq(store.map.get('tuita:ann:a1'), '1', 'stored');
	assert(mod.isDismissed(store, 'a1') === true, 'remembered');
	assert(mod.isDismissed(broken, 'a1') === false, 'throwing storage is not dismissed');
	mod.dismiss(broken, 'a1');
	eq(mod.undismissed([{ id: 'a1' }, { id: 'a2' }], store), [{ id: 'a2' }], 'filters dismissed in order');
}
