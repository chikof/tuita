// Known-answer vectors for assets/js/lists.js: the editor's request body and the member count line.
const eq = (actual, expected, what) => {
	if (JSON.stringify(actual) !== JSON.stringify(expected)) {
		throw new Error(`${what}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
	}
};

export async function run(mod) {
	eq(mod.listBody('  Friends ', ' Close ones\n', true), { name: 'Friends', description: 'Close ones', private: true }, 'trimmed body');
	const entry = { 0: '{compact} members', 1: '{compact} member', 2: '{compact} members', 5: '{compact} members', 1000000: '{compact} members' };
	eq(mod.pluralText(entry, 1, 'en'), '{compact} member', 'English one');
	eq(mod.fill(mod.pluralText(entry, 12_000, 'en'), { compact: '12K' }), '12K members', 'compact count fills in');
	eq(mod.fill('Muted {handle}', {}), 'Muted {handle}', 'unknown slots stay');
}
