// Keyboard shortcuts. One `keydown` listener on `window` at load.
// Page contract: `<dialog id="shortcuts">` (the `?` help), `[data-compose="new"]` (opens the composer),
// `[data-island="stream"][data-handle]` (the viewer; shortcuts are off without it),
// `#main input[type="search"]`, and `#main article[data-post]` with `data-href` and
// `[data-action="like|reply|repost|bookmark"]` buttons.

/** `g` then one of these. `p` goes to the viewer's profile. */
export const GO = {
	h: '/',
	e: '/explore',
	n: '/notifications',
	m: '/messages',
	b: '/bookmarks',
	l: '/lists',
	s: '/settings'
};

/** Window in which the key after `g` counts. */
export const CHORD_MS = 1000;

const TYPING = 'input, textarea, select, [contenteditable]';

/** A `keydown` handler implementing the table above. */
export function createShortcuts(env) {
	const now = env.now ?? Date.now;
	let chordAt = -Infinity;
	let focused = null;

	const posts = () => [...document.querySelectorAll('#main article[data-post]')];

	function move(step) {
		const all = posts();
		if (!all.length) return;
		const at = focused ? all.indexOf(focused) : -1;
		const next = at === -1 ? (step > 0 ? 0 : all.length - 1) : Math.max(0, Math.min(all.length - 1, at + step));
		focused = all[next];
		focused.focus({ preventScroll: true });
		focused.scrollIntoView({ block: 'center', behavior: 'smooth' });
	}

	function act(name) {
		if (focused?.isConnected) focused.querySelector(`[data-action="${name}"]`)?.click();
	}

	return (event) => {
		const target = event.target instanceof Element ? event.target : null;
		const handle = env.me();
		if (target?.closest(TYPING)) return;
		if (event.metaKey || event.ctrlKey || event.altKey || !handle || env.modal()) return;

		const key = event.key;
		if (now() - chordAt < CHORD_MS && (key === 'p' || key in GO)) {
			chordAt = -Infinity;
			event.preventDefault();
			env.navigate(key === 'p' ? `/@${handle}` : GO[key]);
			return;
		}

		switch (key) {
			case 'g':
				chordAt = now();
				return;
			case 'j':
			case 'k':
				event.preventDefault();
				move(key === 'j' ? 1 : -1);
				return;
			case 'n':
				event.preventDefault();
				env.compose();
				return;
			case 'l':
				act('like');
				return;
			case 'r':
				event.preventDefault();
				act('reply');
				return;
			case 't':
				act('repost');
				return;
			case 'b':
				act('bookmark');
				return;
			case 'o':
			case 'Enter': {
				if (target?.closest('a, button')) return;
				const href = focused?.isConnected ? focused.dataset.href : undefined;
				if (href) env.navigate(href);
				return;
			}
			case '/':
				event.preventDefault();
				if (!env.search()) env.navigate('/explore');
				return;
			case '?':
				event.preventDefault();
				env.help();
				return;
		}
	};
}

if (typeof document !== 'undefined' && !document.documentElement.dataset.shortcutsBound) {
	document.documentElement.dataset.shortcutsBound = '';
	window.addEventListener(
		'keydown',
		createShortcuts({
			navigate: (path) => location.assign(path),
			compose: () => document.querySelector('[data-compose="new"]')?.click(),
			help: () => {
				const dialog = document.getElementById('shortcuts');
				if (dialog && !dialog.open) dialog.showModal();
			},
			me: () => document.querySelector('[data-island="stream"]')?.dataset.handle || null,
			modal: () => document.querySelector('dialog[open]') !== null,
			search: () => {
				const field = document.querySelector('#main input[type="search"]');
				field?.focus();
				return field !== null;
			}
		})
	);
}
