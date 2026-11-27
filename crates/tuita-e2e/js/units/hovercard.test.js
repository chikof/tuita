// Known-answer vectors for assets/js/hovercard.js.
const eq = (actual, expected, what) => {
	if (JSON.stringify(actual) !== JSON.stringify(expected)) {
		throw new Error(`${what}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
	}
};

/** A clock the test advances by hand. */
function fakeTimers() {
	let now = 0;
	let next = 1;
	const queue = new Map();
	const timers = {
		set: (run, ms) => {
			const id = next++;
			queue.set(id, { at: now + ms, run });
			return id;
		},
		clear: (id) => void queue.delete(id)
	};
	const advance = (ms) => {
		const end = now + ms;
		for (;;) {
			const due = [...queue.entries()].filter(([, t]) => t.at <= end).sort((a, b) => a[1].at - b[1].at)[0];
			if (!due) break;
			queue.delete(due[0]);
			now = due[1].at;
			due[1].run();
		}
		now = end;
	};
	return { timers, advance, pending: () => queue.size };
}

export async function run(mod) {
	const { createHoverCard, DELAYS } = mod;
	eq(DELAYS, { open: 450, focus: 600, close: 250 }, 'delays');
	const setup = () => {
		const clock = fakeTimers();
		const seen = [];
		const card = createHoverCard((target) => seen.push(target), DELAYS, clock.timers);
		return { ...clock, card, seen };
	};

	{
		const { card, advance, seen } = setup();
		card.enter('a');
		advance(449);
		eq(seen, [], 'not before 450 ms');
		advance(1);
		eq(seen, ['a'], 'opens at 450 ms');
	}
	{
		const { card, advance, seen, pending } = setup();
		card.enter('a');
		advance(300);
		card.leave('a');
		advance(1000);
		eq(seen, [], 'leaving first never opens');
		eq(pending(), 0, 'no timer left');
	}
	{
		const { card, advance, seen } = setup();
		card.enter('a');
		advance(450);
		card.leave('a');
		advance(249);
		eq(card.shown, 'a', 'still open at 249 ms');
		advance(1);
		eq(seen, ['a', null], 'closes 250 ms after leave');
	}
	{
		const { card, advance, seen } = setup();
		card.enter('a');
		advance(450);
		card.leave('a');
		advance(100);
		card.enterCard();
		advance(1000);
		eq(seen, ['a'], 'stays open on the card');
		card.leaveCard();
		advance(250);
		eq(seen, ['a', null], 'closes after leaving the card');
	}
	{
		const { card, advance, seen } = setup();
		card.enter('a');
		advance(450);
		card.leave('a');
		card.enterCard();
		card.leaveCard();
		card.enter('a');
		advance(1000);
		eq(seen, ['a'], 'stays open when the pointer returns to the trigger');
	}
	{
		const { card, advance, seen } = setup();
		card.enter('a');
		advance(450);
		card.leave('a');
		card.enter('b');
		advance(250);
		eq(seen, ['a', null], 'old card closes');
		advance(200);
		eq(seen, ['a', null, 'b'], 'second trigger opens');
	}
	{
		const { card, advance, seen } = setup();
		card.enter('a');
		advance(450);
		card.leave('a');
		card.enter('b');
		advance(100);
		card.leave('b');
		advance(1000);
		eq(seen, ['a', null], 'old card closes though the next trigger never opened');
	}
	{
		const { card, advance, seen } = setup();
		card.focus('a');
		advance(599);
		eq(seen, [], 'focus not before 600 ms');
		advance(1);
		eq(seen, ['a'], 'focus opens at 600 ms');
	}
	{
		const { card, advance, seen } = setup();
		card.focus('a');
		advance(600);
		card.blur('a');
		eq(seen, ['a', null], 'blur closes at once');
		card.enter('a');
		advance(450);
		card.focus('a');
		card.blur('a');
		eq(card.shown, 'a', 'blur under the pointer keeps it open');
	}
	{
		const { card, advance, seen } = setup();
		card.focus('a');
		advance(300);
		card.blur('a');
		advance(1000);
		eq(seen, [], 'blur cancels a pending focus open');
	}
	{
		const { card, advance, seen, pending } = setup();
		card.enter('a');
		advance(450);
		card.close();
		card.leave('a');
		advance(1000);
		eq(seen, ['a', null], 'close() is final');
		eq(pending(), 0, 'no timer left after close()');
	}

	const realFetch = window.fetch;
	const urls = [];
	window.fetch = (url) => {
		urls.push(url);
		const none = String(url).endsWith('/none');
		return Promise.resolve(new Response(none ? null : '<div></div>', { status: none ? 204 : 200 }));
	};
	try {
		mod.forgetHoverCards();
		const [first, second] = await Promise.all([mod.loadHoverCard('ada'), mod.loadHoverCard('ada')]);
		eq([first, second], ['<div></div>', '<div></div>'], 'shared body');
		await mod.loadHoverCard('ada');
		eq(urls, ['/_tuita/hovercard/ada'], 'one request for concurrent and repeated hovers');
		await mod.loadHoverCard('ada@remote.example');
		eq(urls[1], '/_tuita/hovercard/ada%40remote.example', 'remote account is encoded');
		eq(await mod.loadHoverCard('none'), null, 'a non-OK answer shows nothing');
	} finally {
		window.fetch = realFetch;
		mod.forgetHoverCards();
	}
}
