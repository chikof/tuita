// Known-answer vectors for assets/js/share.js.
const eq = (actual, expected, what) => {
	if (JSON.stringify(actual) !== JSON.stringify(expected)) {
		throw new Error(`${what}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
	}
};

export async function run(mod) {
	eq(mod.mediaFileName({ id: 'abc', kind: 'image', url: '/media/abc.webp' }), 'abc.webp', 'id and URL extension');
	eq(
		mod.mediaFileName({ id: 'v1', kind: 'video', url: 'https://cdn.example/media/v1.MP4?x=1' }),
		'v1.mp4',
		'extension lowercased, query ignored'
	);
	eq(mod.mediaFileName({ id: 'a', kind: 'image', url: '/media/a' }), 'a.jpg', 'image fallback');
	eq(mod.mediaFileName({ id: 'a', kind: 'video', url: '/media/a' }), 'a.mp4', 'video fallback');
}
