// Known-answer vectors for assets/js/shortcuts.js.
const assert = (ok, what) => {
	if (!ok) throw new Error(`assertion failed: ${what}`);
};
const eq = (actual, expected, what) => {
	if (JSON.stringify(actual) !== JSON.stringify(expected)) {
		throw new Error(`${what}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
	}
};

class FakeElement {
	closest() {
		return null;
	}
}

const press = (handler, key, extra = {}) => {
	const event = {
		key,
		target: new FakeElement(),
		metaKey: false,
		ctrlKey: false,
		altKey: false,
		defaultPrevented: false,
		preventDefault() {
			this.defaultPrevented = true;
		},
		...extra
	};
	handler(event);
	return event;
};

const counter = () => {
	const calls = [];
	const fn = (...args) => (calls.push(args), false);
	fn.calls = calls;
	return fn;
};

export async function run(mod) {
	const realElement = globalThis.Element;
	globalThis.Element = FakeElement;
	try {
		let now = 0;
		let modal = false;
		const make = (over = {}) => ({
			navigate: counter(),
			compose: counter(),
			help: counter(),
			me: () => 'ada',
			modal: () => modal,
			search: counter(),
			now: () => now,
			...over
		});

		eq(mod.CHORD_MS, 1000, 'chord window');
		eq(mod.GO.h, '/', 'go table');

		let env = make();
		let handler = mod.createShortcuts(env);
		press(handler, 'g');
		press(handler, 'e');
		press(handler, 'g');
		press(handler, 'p');
		eq(env.navigate.calls, [['/explore'], ['/@ada']], 'g chords navigate');

		env = make();
		handler = mod.createShortcuts(env);
		now = 0;
		press(handler, 'g');
		now = 1500;
		press(handler, 'e');
		eq(env.navigate.calls, [], 'chord forgotten after a second');

		now = 0;
		env = make();
		handler = mod.createShortcuts(env);
		press(handler, 'n');
		press(handler, '?');
		press(handler, '/');
		eq(env.compose.calls.length, 1, 'n composes');
		eq(env.help.calls.length, 1, '? shows help');
		eq(env.navigate.calls, [['/explore']], '/ falls back to explore');

		env = make({ search: () => true });
		handler = mod.createShortcuts(env);
		press(handler, '/');
		eq(env.navigate.calls, [], '/ focuses search when present');

		env = make();
		handler = mod.createShortcuts(env);
		press(handler, 'n', { ctrlKey: true });
		modal = true;
		press(handler, 'n');
		modal = false;
		const signedOut = make({ me: () => null });
		press(mod.createShortcuts(signedOut), 'n');
		eq(env.compose.calls.length + signedOut.compose.calls.length, 0, 'off with modifier, dialog or signed out');

		const typing = Object.assign(new FakeElement(), { closest: () => ({}) });
		press(handler, 'n', { target: typing });
		eq(env.compose.calls.length, 0, 'off while typing');
	} finally {
		globalThis.Element = realElement;
	}
}
