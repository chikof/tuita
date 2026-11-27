// Known-answer vectors for assets/js/pull.js.
const eq = (actual, expected, what) => {
	if (JSON.stringify(actual) !== JSON.stringify(expected)) {
		throw new Error(`${what}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
	}
};

export async function run(mod) {
	const { MAX_PULL, THRESHOLD, pullDistance, shouldRefresh } = mod;
	// damps the drag, never goes negative and stops at the maximum
	eq(pullDistance(100, 140), 20, 'half of the finger travel');
	eq(pullDistance(100, 60), 0, 'pushing up pulls nothing');
	eq(pullDistance(0, 1000), MAX_PULL, 'a long drag stops at the maximum');
	// refreshes from the threshold on
	eq(shouldRefresh(THRESHOLD - 1), false, 'just short');
	eq(shouldRefresh(pullDistance(0, THRESHOLD / 0.5)), true, 'a drag of the threshold over the resistance');
	eq([THRESHOLD, MAX_PULL], [64, 96], 'the spec constants');
}
