// Known-answer vectors for assets/js/scheduled.js (the same window rules the composer uses).
const eq = (actual, expected, what) => {
	if (JSON.stringify(actual) !== JSON.stringify(expected)) {
		throw new Error(`${what}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
	}
};

export async function run(mod) {
	const date = new Date(2026, 10, 9, 14, 5);
	eq(mod.toLocalInput(date), '2026-11-09T14:05', 'datetime-local value');
	eq(mod.fromLocalInput('2026-11-09T14:05'), date.toISOString(), 'round trip');
	eq(mod.fromLocalInput(''), null, 'empty is no time');

	const now = Date.parse('2026-11-09T10:00:00Z');
	eq(mod.inWindow('2026-11-09T10:00:30Z', now), false, 'under a minute ahead');
	eq(mod.inWindow('2026-11-09T10:01:00Z', now), true, 'a minute ahead');
	eq(mod.inWindow('2027-02-07T10:00:00Z', now), true, '90 days ahead');
	eq(mod.inWindow('2027-02-07T10:00:01Z', now), false, 'past 90 days');

	const bounds = mod.localBounds(now);
	eq(bounds.min, mod.toLocalInput(new Date(now + 120_000)), 'min is two minutes ahead');
	eq(bounds.max, mod.toLocalInput(new Date(now + 90 * 86_400_000 - 60_000)), 'max is a minute inside the window');

	eq(mod.localTime('nope', 'en'), null, 'a bad instant has no local time');
	eq(mod.localTime('2026-11-09T10:00:00Z', 'en') === new Date('2026-11-09T10:00:00Z').toLocaleString('en'), true, 'the page language formats it');
}
