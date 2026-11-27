// Known-answer vectors for assets/js/settings.js: the region name of a country code and the copy fallback.
const eq = (actual, expected, what) => {
	if (JSON.stringify(actual) !== JSON.stringify(expected)) {
		throw new Error(`${what}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
	}
};

export async function run(mod) {
	eq(mod.regionName('de', 'en'), 'Germany', 'a lower case code is named in English');
	eq(mod.regionName('DE', 'ja'), 'ドイツ', 'and in Japanese');
	eq(mod.regionName('ZZ', 'en'), 'Unknown Region', 'an unassigned code reads as unknown');
	eq(mod.regionName('not a code', 'en'), 'not a code', 'text that is no code stays as it is');
	eq(typeof mod.copyText, 'function', 'copyText is exported');
}
