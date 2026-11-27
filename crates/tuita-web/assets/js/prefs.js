// Device-local appearance preferences. The storage shape is stable.
// Controls are `[data-pref="mode|ink|textScale|contrast|motion"]`: radios, a select, a range or
// number input, a checkbox (checked = its `value`, unchecked = the default), or a
// `button[data-value]` (reflected as aria-selected on role=tab, else aria-pressed).

export const INKS = ['bone', 'phosphor', 'amber', 'cyan', 'rose'];
export const MODES = ['system', 'light', 'dark'];
export const TEXT_SCALES = [0.9, 1, 1.1, 1.25];
export const CONTRASTS = ['normal', 'more'];
export const MOTIONS = ['system', 'reduce'];

export const PREFS_KEY = 'tuita:prefs';
export const DEFAULT_PREFS = {
	mode: 'system',
	ink: 'bone',
	textScale: 1,
	contrast: 'normal',
	motion: 'system'
};

const ALLOWED = {
	mode: MODES,
	ink: INKS,
	textScale: TEXT_SCALES,
	contrast: CONTRASTS,
	motion: MOTIONS
};

const pick = (allowed, value, fallback) => (allowed.includes(value) ? value : fallback);

export function parsePrefs(raw) {
	let data = {};
	try {
		data = JSON.parse(raw ?? '') ?? {};
	} catch {
		// Absent or corrupt storage falls back to defaults.
	}
	const out = {};
	for (const key of Object.keys(DEFAULT_PREFS)) out[key] = pick(ALLOWED[key], data[key], DEFAULT_PREFS[key]);
	return out;
}

function stored() {
	try {
		return localStorage.getItem(PREFS_KEY);
	} catch {
		return null;
	}
}

export const prefs = parsePrefs(typeof localStorage === 'undefined' ? null : stored());

/** Mirrors boot.js. */
export function applyPrefs(p, root = document.documentElement) {
	const dark = matchMedia('(prefers-color-scheme: dark)').matches;
	const calm = matchMedia('(prefers-reduced-motion: reduce)').matches;
	root.dataset.theme = p.mode === 'system' ? (dark ? 'dark' : 'light') : p.mode;
	root.dataset.ink = p.ink;
	root.dataset.contrast = p.contrast;
	if (p.motion === 'reduce' || calm) root.dataset.motion = 'reduce';
	else delete root.dataset.motion;
	root.style.setProperty('--text-scale', String(p.textScale));
}

/** Form values are strings; the text scale is a number and snaps to the nearest step. */
function coerce(key, raw) {
	if (key !== 'textScale') return raw;
	const n = Number(raw);
	if (!Number.isFinite(n)) return undefined;
	return TEXT_SCALES.reduce((a, b) => (Math.abs(b - n) < Math.abs(a - n) ? b : a));
}

export function setPref(key, value) {
	const next = coerce(key, value);
	if (!ALLOWED[key]?.includes(next)) return;
	// Another tab may have changed other keys since this page loaded.
	Object.assign(prefs, parsePrefs(stored()), { [key]: next });
	try {
		localStorage.setItem(PREFS_KEY, JSON.stringify({ ...prefs }));
	} catch {
		// Storage may be unavailable (private mode); the preference still applies this session.
	}
	applyPrefs(prefs);
}

function reflect(el) {
	const want = String(prefs[el.dataset.pref]);
	if (el instanceof HTMLButtonElement) {
		el.setAttribute(el.getAttribute('role') === 'tab' ? 'aria-selected' : 'aria-pressed', String(el.dataset.value === want));
	} else if (el instanceof HTMLInputElement && (el.type === 'radio' || el.type === 'checkbox')) {
		el.checked = el.value === want;
	} else if ('value' in el) {
		el.value = want;
	}
}

const reflectAll = (root, key) => {
	for (const el of root.querySelectorAll(key ? `[data-pref="${key}"]` : '[data-pref]')) reflect(el);
};

function chosen(el) {
	if (el instanceof HTMLInputElement && el.type === 'radio') return el.checked ? el.value : undefined;
	if (el instanceof HTMLInputElement && el.type === 'checkbox') return el.checked ? el.value : DEFAULT_PREFS[el.dataset.pref];
	if (el instanceof HTMLButtonElement) return el.dataset.value;
	return el.value;
}

function mount(selector, init) {
	const live = new Map();
	const scan = (root) => {
		if (root.nodeType !== 1) return;
		const found = root.matches(selector) ? [root, ...root.querySelectorAll(selector)] : root.querySelectorAll(selector);
		for (const el of found) if (!live.has(el)) live.set(el, init(el) ?? null);
	};
	const sweep = () => { for (const [el, stop] of live) if (!el.isConnected) { stop?.(); live.delete(el); } };
	scan(document.documentElement);
	new MutationObserver((records) => {
		for (const r of records) for (const n of r.addedNodes) scan(n);
		sweep();
	}).observe(document.documentElement, { childList: true, subtree: true });
}

if (typeof document !== 'undefined' && !document.documentElement.dataset.prefsBound) {
	document.documentElement.dataset.prefsBound = '';
	mount('[data-pref]', reflect);
	const set = (el) => {
		const value = chosen(el);
		if (value === undefined) return;
		setPref(el.dataset.pref, value);
		reflectAll(document, el.dataset.pref);
	};
	document.addEventListener('change', (event) => {
		const el = event.target instanceof Element ? event.target.closest('[data-pref]') : null;
		if (el && !(el instanceof HTMLButtonElement)) set(el);
	});
	document.addEventListener('click', (event) => {
		const el = event.target instanceof Element ? event.target.closest('button[data-pref][data-value]') : null;
		if (el) set(el);
	});
	// boot.js keeps the values it read at load; reapply the current ones when the system setting flips.
	for (const query of ['(prefers-color-scheme: dark)', '(prefers-reduced-motion: reduce)']) {
		matchMedia(query).addEventListener('change', () => applyPrefs(prefs));
	}
}
