// Known-answer vectors for assets/js/time.js.
const assert = (ok, what) => {
	if (!ok) throw new Error(`assertion failed: ${what}`);
};
const eq = (actual, expected, what) => {
	if (JSON.stringify(actual) !== JSON.stringify(expected)) {
		throw new Error(`${what}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
	}
};

export async function run(mod) {
	const strings = {
		time_now: 'now',
		time_minutes: '{n}m',
		time_hours: '{n}h',
		time_days: '{n}d'
	};
	const now = Date.parse('2026-11-09T12:00:00Z');
	const rel = (iso, lang = 'en') => mod.relTime(iso, now, strings, lang);

	eq(rel('2026-11-09T11:59:30Z'), 'now', 'under 45 s');
	eq(rel('2026-11-09T11:59:16Z'), 'now', '44 s');
	eq(rel('2026-11-09T11:59:15Z'), '0m', '45 s floors to 0 minutes');
	eq(rel('2026-11-09T11:55:00Z'), '5m', 'minutes');
	eq(rel('2026-11-09T11:00:01Z'), '59m', 'under an hour');
	eq(rel('2026-11-09T11:00:00Z'), '1h', 'an hour');
	eq(rel('2026-11-09T09:00:00Z'), '3h', 'hours');
	eq(rel('2026-11-08T12:00:01Z'), '23h', 'under a day');
	eq(rel('2026-11-07T12:00:00Z'), '2d', 'days');
	eq(rel('2026-11-02T12:00:01Z'), '6d', 'under a week');
	eq(rel('2026-11-10T12:00:00Z'), 'now', 'the future reads as now');

	const week = new Date('2026-11-02T12:00:00Z');
	eq(
		rel('2026-11-02T12:00:00Z'),
		new Intl.DateTimeFormat('en', { month: 'short', day: 'numeric' }).format(week),
		'a week old reads as a short date without the year'
	);
	assert(!/2026/.test(rel('2026-11-02T12:00:00Z')), 'same year omits the year');
	assert(/2025/.test(rel('2025-06-01T12:00:00Z')), 'another year shows the year');
	eq(
		rel('2025-06-01T12:00:00Z', 'ja'),
		new Intl.DateTimeFormat('ja', { month: 'short', day: 'numeric', year: 'numeric' }).format(new Date('2025-06-01T12:00:00Z')),
		'dates follow the locale'
	);

	// Client strings: plain entries, plural entries, number formatting.
	eq(mod.text('{n} min', { n: 5 }, 'pl'), '5 min', 'plain entry');
	eq(mod.text('a {x} b {x}', { x: 'y' }, 'en'), 'a y b y', 'repeated placeholder');
	eq(mod.text('keep {other}', {}, 'en'), 'keep {other}', 'unknown placeholder stays');
	eq(mod.text('{n} people', { n: 1234 }, 'en'), '1,234 people', 'numbers use the locale');
	const en = { 0: '{count} items', 1: '{count} item', 2: '{count} items' };
	eq(mod.text(en, { count: 1 }, 'en'), '1 item', 'en one');
	eq(mod.text(en, { count: 7 }, 'en'), '7 items', 'en other');
	eq(mod.text(en, { count: 0 }, 'en'), '0 items', 'en zero is other');
	const pl = { 0: '{count} wpisów', 1: '{count} wpis', 2: '{count} wpisy', 5: '{count} wpisów', 1000000: '{count} wpisów' };
	eq(mod.text(pl, { count: 1 }, 'pl'), '1 wpis', 'pl one');
	eq(mod.text(pl, { count: 3 }, 'pl'), '3 wpisy', 'pl few');
	eq(mod.text(pl, { count: 12 }, 'pl'), '12 wpisów', 'pl many');
	eq(mod.text({ 0: '{count}個' }, { count: 4 }, 'ja'), '4個', 'ja has one category');
}
