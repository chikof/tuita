// Known-answer vectors for assets/js/messages.js. Messages are
// `{ id, mine, createdAt, readAt }` (what a thread item carries), conversations `{ id, name, acct }`.
const eq = (actual, expected, what) => {
	if (JSON.stringify(actual) !== JSON.stringify(expected)) {
		throw new Error(`${what}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
	}
};

const msg = (id, at, mine = false, readAt = null) => ({ id, mine, createdAt: `2026-01-01T${at}Z`, readAt });
const convo = (id, handle, domain = null) => ({
	id,
	name: `Name ${handle}`,
	acct: domain ? `${handle}@${domain}` : handle
});

export async function run(mod) {
	// mergeMessages
	eq(
		mod.mergeMessages([msg('b', '10:01:00'), msg('c', '10:02:00')], [msg('a', '10:00:00'), msg('b', '10:01:00')]).map((x) => x.id),
		['a', 'b', 'c'],
		'dedupes and sorts by id'
	);
	eq(mod.mergeMessages([msg('10', '10:00:00')], [msg('9', '10:00:00')]).map((x) => x.id), ['9', '10'], 'longer ids come after shorter ones');
	eq(
		mod.mergeMessages([msg('1', '10:00:00')], [msg('1', '10:00:00', false, 'x')])[0].readAt,
		'x',
		'the later copy wins'
	);

	// groupBubbles
	eq(
		mod
			.groupBubbles([msg('a', '10:00:00'), msg('b', '10:02:59'), msg('c', '10:06:00'), msg('d', '10:06:10', true)])
			.map((b) => [b.first, b.last]),
		[[true, false], [false, true], [true, true], [true, true]],
		'one sender within 3 minutes is one run'
	);

	// seenId
	eq(
		mod.seenId([msg('a', '10:00:00', true, 'x'), msg('b', '10:01:00', true, 'x'), msg('c', '10:02:00', true), msg('d', '10:03:00', false, 'x')]),
		'b',
		'the last own message that was read'
	);
	eq(mod.seenId([msg('a', '10:00:00')]), null, 'nothing read');

	// mergeConversations
	eq(
		mod.mergeConversations([convo('1', 'a'), convo('2', 'b'), convo('3', 'c')], [convo('3', 'c'), convo('4', 'd')]).map((c) => c.id),
		['3', '4', '1', '2'],
		'fresh rows go to the top, the rest keep their order'
	);
	eq(
		mod.mergeConversations([{ ...convo('1', 'a'), unread: 1 }], [{ ...convo('1', 'a'), unread: 2 }])[0].unread,
		2,
		'the fresh copy of a row wins'
	);

	// filterConversations
	const list = [convo('1', 'ada'), convo('2', 'bob', 'remote.example')];
	if (mod.filterConversations(list, '  ') !== list) throw new Error('a blank query returns the list untouched');
	eq(mod.filterConversations(list, '@ADA').map((c) => c.id), ['1'], 'name or account, ignoring case and a leading @');
	eq(mod.filterConversations(list, 'bob@remote').map((c) => c.id), ['2'], 'a remote account matches by its full name');
	eq(mod.filterConversations(list, 'name').length, 2, 'the display name matches');
	eq(mod.filterConversations(list, 'zzz'), [], 'nothing matches');

	eq(mod.acctOf({ handle: 'ada', domain: null }), 'ada', 'local account name');
	eq(mod.acctOf({ handle: 'ada', domain: 'remote.example' }), 'ada@remote.example', 'remote account name');
}
