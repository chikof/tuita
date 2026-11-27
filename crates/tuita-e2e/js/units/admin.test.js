// Known-answer vectors for assets/js/admin.js: the emote form's checks and the toast templates.
const eq = (actual, expected, what) => {
	if (JSON.stringify(actual) !== JSON.stringify(expected)) {
		throw new Error(`${what}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
	}
};

export async function run(mod) {
	eq(['a', 'ab', ' ab ', 'a_b', 'a-b', 'a'.repeat(32), 'a'.repeat(33)].map(mod.validShortcode), [false, true, true, true, false, true, false], 'shortcodes are 2 to 32 letters, digits or underscores');
	eq(mod.fileProblem({ type: 'image/png', size: 100 }), null, 'a small png');
	eq(mod.fileProblem({ type: 'text/plain', size: 5 }), 'type', 'text is not a picture');
	eq(mod.fileProblem({ type: 'image/gif', size: 256 * 1024 + 1 }), 'size', 'over 256 KB');
	eq(mod.fileProblem({ type: 'image/webp', size: 256 * 1024 }), null, 'exactly 256 KB');
	eq(mod.fill('Code {code} created and {code}', 'code', 'ABC'), 'Code ABC created and ABC', 'every slot is filled');
}
