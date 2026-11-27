// Keeps `<time datetime data-rel>` text current.
// Strings come from the `#tuita-l10n` JSON block: `time_now`, `time_minutes`, `time_hours`,
// `time_days` are plain entries (`"{n}m"`); a plural entry maps sample counts to text.

let cached;

/** The client strings the layout renders. */
export function clientStrings() {
	cached ??= JSON.parse(document.getElementById('tuita-l10n')?.textContent ?? '{}');
	return cached;
}

/** Fills `{name}` placeholders; a plural entry picks the sample in the same category as `vars.count`. */
export function text(entry, vars = {}, lang = document.documentElement.lang) {
	let template = entry;
	if (entry && typeof entry === 'object') {
		const rules = new Intl.PluralRules(lang);
		const want = rules.select(vars.count);
		const samples = Object.entries(entry);
		template = (samples.find(([sample]) => rules.select(Number(sample)) === want) ?? samples.at(-1))[1];
	}
	return String(template).replace(/\{(\w+)\}/g, (hole, name) =>
		name in vars ? (typeof vars[name] === 'number' ? new Intl.NumberFormat(lang).format(vars[name]) : String(vars[name])) : hole
	);
}

/** `now`, `5m`, `3h`, `2d`, then a short date. */
export function relTime(iso, now = Date.now(), strings = clientStrings(), lang = document.documentElement.lang) {
	const at = Date.parse(iso);
	const seconds = (now - at) / 1000;
	if (seconds < 45) return text(strings.time_now, {}, lang);
	if (seconds < 3600) return text(strings.time_minutes, { n: Math.floor(seconds / 60) }, lang);
	if (seconds < 86_400) return text(strings.time_hours, { n: Math.floor(seconds / 3600) }, lang);
	if (seconds < 7 * 86_400) return text(strings.time_days, { n: Math.floor(seconds / 86_400) }, lang);
	const date = new Date(at);
	const sameYear = date.getFullYear() === new Date(now).getFullYear();
	return new Intl.DateTimeFormat(lang, sameYear ? { month: 'short', day: 'numeric' } : { month: 'short', day: 'numeric', year: 'numeric' }).format(date);
}

export function refresh(root = document) {
	const strings = clientStrings();
	const lang = document.documentElement.lang;
	const now = Date.now();
	for (const el of root.querySelectorAll('time[data-rel][datetime]')) {
		const iso = el.getAttribute('datetime');
		if (Number.isNaN(Date.parse(iso))) continue;
		const next = relTime(iso, now, strings, lang);
		if (el.textContent !== next) el.textContent = next;
	}
}

if (typeof document !== 'undefined' && !document.documentElement.dataset.timeBound) {
	document.documentElement.dataset.timeBound = '';
	setInterval(refresh, 60_000);
	document.addEventListener('visibilitychange', () => document.visibilityState === 'visible' && refresh());
}
