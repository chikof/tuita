// Known-answer vectors for assets/js/dither.js. The numeric vectors are
// the ones the Rust copies (`ui/hash.rs`, `ui/mask.rs`) pin, so both sides draw the same bytes.
const assert = (ok, what) => {
	if (!ok) throw new Error(`assertion failed: ${what}`);
};
const eq = (actual, expected, what) => {
	if (JSON.stringify(actual) !== JSON.stringify(expected)) {
		throw new Error(`${what}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
	}
};
const rows = (flat, n) => Array.from({ length: n }, (_, y) => Array.from({ length: n }, (_, x) => flat[y * n + x]));
const rects = (mask) => (mask.match(/%3Crect /g) ?? []).length;
const frame = () => new Promise((resolve) => requestAnimationFrame(() => resolve()));
const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

export async function run(mod) {
	const { bayer, bayerRanks, fnv1a, mulberry32, rampMask, seeded, fieldOf, nextShown, optionsOf } = mod;

	eq(rows(bayerRanks(4), 4), [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]], 'bayer 4');
	eq(rows(bayerRanks(2), 2), [[0, 2], [3, 1]], 'bayer 2');
	for (const n of [2, 4, 8]) {
		eq([...bayerRanks(n)].sort((a, b) => a - b), Array.from({ length: n * n }, (_, i) => i), `bayer ${n} ranks every cell once`);
	}
	const t = bayer(4);
	assert(Math.abs(t[0] - 0.5 / 16) < 1e-6 && Math.abs(t[1] - 8.5 / 16) < 1e-6 && Math.abs(t[15] - 5.5 / 16) < 1e-6, 'thresholds are (rank + 0.5) / n²');

	assert(fnv1a('') === 0x811c9dc5, 'fnv1a empty');
	assert(fnv1a('a') === 0xe40c292c, 'fnv1a a');
	assert(fnv1a('héllo😀') === 4_113_148_722, 'fnv1a hashes UTF-16 code units');
	const rnd = mulberry32(fnv1a('ada'));
	eq([rnd(), rnd(), rnd()], [0.9841111886780709, 0.9519213286694139, 0.36480955220758915], 'mulberry32 stream');

	eq([fnv1a(rampMask(0, 1, 12, 'x')), fnv1a(rampMask(1, 0, 12, 'x')), fnv1a(rampMask(0, 1, 12, 'y'))], [2_402_834_565, 2_124_622_963, 1_613_875_545], 'ramp masks match the Rust copy');
	const lit = (mask, axis, index) =>
		[...mask.matchAll(/%3Crect x='(\d+)' y='(\d+)'/g)].filter((m) => Number(m[axis === 'x' ? 1 : 2]) === index).length;
	const ramp = rampMask(0, 1, 8, 'x');
	assert(lit(ramp, 'x', 0) < lit(ramp, 'x', 7), 'ramps go from sparse to dense');
	assert(lit(ramp, 'x', 7) === 4 && lit(ramp, 'x', 0) <= 1, 'ramp ends');
	assert(rampMask(0, 1, 8, 'x') === ramp && ramp.startsWith('url("data:image/svg+xml,') && !ramp.includes('<'), 'ramps are memoised css urls');
	assert(rects(ramp) > 0, 'ramps draw cells');

	const a = seeded('ada');
	const b = seeded('ada');
	for (const [x, y, time] of [[0, 0, 0], [0.3, 0.7, 2], [1, 1, 9]]) assert(a(x, y, time, 1.5) === b(x, y, time, 1.5), 'seeded is deterministic');
	for (const handle of ['ada', 'bob', 'carol', 'dave', 'eve', 'frank', 'grace']) {
		const v = seeded(handle)(0.4, 0.6, 3, 1);
		assert(v >= -0.01 && v <= 1.01, `seeded(${handle}) stays in range`);
	}

	assert(fieldOf({ field: 'seeded', seed: 'ada' })(0.4, 0.6, 3, 1) === a(0.4, 0.6, 3, 1), 'data-field=seeded is seeded(data-seed)');
	assert(fieldOf({})(0.2, 0.2, 1, 1) === fieldOf({ field: 'waves' })(0.2, 0.2, 1, 1), 'waves is the default field');
	assert(fieldOf({ field: 'ripples', seed: '7' })(0.3, 0.3, 1, 1) === mod.ripples(7)(0.3, 0.3, 1, 1), 'a numeric seed is used as is');
	assert(fieldOf({ field: 'noise', seed: 'x' })(0.3, 0.3, 1, 1) === mod.noise(fnv1a('x'))(0.3, 0.3, 1, 1), 'a text seed is hashed');
	eq(
		(({ field, ...rest }) => rest)(optionsOf({ cell: '5', matrix: '8', animate: 'false', stillTime: '2', speed: '0.5' })),
		{ cell: 5, matrix: 8, animate: false, stillTime: 2, speed: 0.5 },
		'options come from data attributes'
	);
	eq((({ field, ...rest }) => rest)(optionsOf({})), { cell: 6, matrix: 4, animate: true, stillTime: 1, speed: 1 }, 'option defaults');

	eq([nextShown(0, 100), nextShown(95, 100), nextShown(100, 0), nextShown(3, 3)], [12, 96, 88, 3], 'the number eases a ninth of the gap, at least one step');

	// An `empty_state` canvas painted by the live island: lit pixels in the ink colour, still frame.
	const art = document.createElement('div');
	art.style.cssText = 'position:relative;width:160px;height:160px';
	art.innerHTML =
		'<canvas aria-hidden="true" data-island="dither-field" data-field="seeded" data-seed="bookmarks" data-cell="5" data-opacity="0.9" style="position:absolute;inset:0;width:100%;height:100%;color:rgb(10,200,30)"></canvas>';
	document.body.append(art);
	const canvas = art.querySelector('canvas');
	for (let i = 0; i < 20 && canvas.width !== 32; i++) await frame();
	eq([canvas.width, canvas.height], [32, 32], 'the canvas is sized to the box in cells');
	const { data } = canvas.getContext('2d').getImageData(0, 0, canvas.width, canvas.height);
	let litCells = 0;
	for (let i = 0; i < data.length; i += 4) {
		if (data[i + 3] === 255) {
			litCells++;
			assert(data[i] === 10 && data[i + 1] === 200 && data[i + 2] === 30, 'lit cells use the ink');
		}
	}
	assert(litCells > 0 && litCells < canvas.width * canvas.height, `the field is dithered, not blank or solid (${litCells} lit)`);
	art.remove();

	// A counting number starts below its value and settles on the rendered text.
	const counter = document.createElement('div');
	counter.innerHTML =
		'<span data-island="dither-number" data-value="1234" data-format="plain" data-interval="10"><span aria-hidden="true"><span data-shown="">1234</span></span></span>';
	document.body.append(counter);
	await frame();
	const shown = counter.querySelector('[data-shown]');
	assert(shown.textContent !== '1234', `counting starts below the value (shows ${shown.textContent})`);
	for (let i = 0; i < 300 && shown.textContent !== '1234'; i++) await sleep(20);
	eq(shown.textContent, '1234', 'the count ends on the rendered text');
	counter.remove();

	// Dither text sizes its mask to the rendered height: one row per 2px.
	const text = document.createElement('span');
	text.dataset.island = 'dither-text';
	text.dataset.from = '1';
	text.dataset.to = '0.45';
	text.dataset.rows = '24';
	text.style.cssText = 'display:inline-block;font-size:40px;line-height:1.05';
	text.textContent = '404';
	document.body.append(text);
	await frame();
	await frame();
	const rowsWanted = Math.max(4, Math.round(text.clientHeight / 2));
	eq(text.style.getPropertyValue('--rows'), String(rowsWanted), 'the mask has one row per 2px of height');
	assert(text.style.maskImage.includes('data:image/svg+xml'), 'dither text is masked');
	text.remove();
}
