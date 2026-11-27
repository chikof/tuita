// Known-answer vectors for assets/js/views.js.
const eq = (actual, expected, what) => {
	if (JSON.stringify(actual) !== JSON.stringify(expected)) {
		throw new Error(`${what}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
	}
};

export async function run(mod) {
	eq([mod.BATCH, mod.FLUSH_MS, mod.DWELL_MS], [50, 5000, 1000], 'constants');

	const setup = (size = 3) => {
		const sent = [];
		let run;
		let calls = 0;
		const later = (fn) => {
			calls++;
			run = fn;
			return () => (run = undefined);
		};
		const batcher = mod.createBatcher(async (ids) => void sent.push(ids), later, size, 5000);
		return { batcher, sent, calls: () => calls, fire: () => run?.() };
	};

	{
		const { batcher, sent, calls, fire } = setup();
		batcher.mark('a');
		batcher.mark('a');
		batcher.mark('b');
		eq(sent, [], 'nothing sent before the timer');
		eq(calls(), 1, 'one timer');
		fire();
		eq(sent, [['a', 'b']], 'sent once each, on the timer');
		batcher.mark('a');
		fire();
		eq(sent, [['a', 'b']], 'a seen id is not sent again');
	}
	{
		const { batcher, sent } = setup(2);
		batcher.mark('a');
		batcher.mark('b');
		batcher.mark('c');
		eq(sent, [['a', 'b']], 'full batch flushes at once');
		batcher.flush();
		eq(sent, [['a', 'b'], ['c']], 'flush sends the rest');
	}
}
