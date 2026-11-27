// Known-answer vectors for assets/js/follow.js.
const eq = (actual, expected, what) => {
	if (JSON.stringify(actual) !== JSON.stringify(expected)) {
		throw new Error(`${what}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
	}
};

export async function run(mod) {
	eq(mod.followView({ following: false, followPending: false }), 'none', 'no relation');
	eq(mod.followView({ following: false, followPending: true }), 'pending', 'a request');
	eq(mod.followView({ following: true, followPending: false }), 'following', 'a follow');

	eq(mod.followStep('none', true), 'pending', 'a remote account is requested');
	eq(mod.followStep('none', false), 'following', 'a local account is followed at once');
	eq(mod.followStep('pending', true), 'none', 'a request is cancelled');
	eq(mod.followStep('following', true), 'none', 'a remote follow is undone');
	eq(mod.followStep('following', false), 'none', 'a local follow is undone');

	eq(mod.fill('Followed {handle}', { handle: '@ada' }), 'Followed @ada', 'a slot is filled');
	const followers = { 0: '{count} Followers', 1: '{count} Follower', 2: '{count} Followers' };
	eq(mod.pluralText(followers, 1, 'en'), '1 Follower', 'one');
	eq(mod.pluralText(followers, 3, 'en'), '3 Followers', 'many');
}
