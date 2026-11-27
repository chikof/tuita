// Known-answer vectors for assets/js/notifications.js: which live notifications change a list.
const eq = (actual, expected, what) => {
	if (JSON.stringify(actual) !== JSON.stringify(expected)) {
		throw new Error(`${what}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
	}
};

export async function run(mod) {
	eq(mod.MENTION_KINDS, ['reply', 'mention', 'quote'], 'the mention kinds');
	for (const kind of ['like', 'repost', 'follow', 'reply', 'mention', 'quote', 'poll']) {
		eq(mod.affects('all', kind), true, `the all tab shows ${kind}`);
	}
	for (const kind of ['reply', 'mention', 'quote']) eq(mod.affects('mentions', kind), true, `mentions shows ${kind}`);
	for (const kind of ['like', 'repost', 'follow', 'follow_request', 'poll', 'affiliation']) {
		eq(mod.affects('mentions', kind), false, `mentions skips ${kind}`);
	}
}
