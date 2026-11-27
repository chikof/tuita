// Known-answer vectors for assets/js/cropper.js.
const assert = (ok, what) => {
	if (!ok) throw new Error(`assertion failed: ${what}`);
};
const eq = (actual, expected, what) => {
	if (JSON.stringify(actual) !== JSON.stringify(expected)) {
		throw new Error(`${what}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
	}
};
const close = (a, b, what, digits = 6) => assert(Math.abs(a - b) < 10 ** -digits, `${what}: ${a} vs ${b}`);

const fnv1a = (input) => {
	let h = 0x811c9dc5;
	for (let i = 0; i < input.length; i++) {
		h ^= input.charCodeAt(i);
		h = Math.imul(h, 0x01000193);
	}
	return h >>> 0;
};

export async function run(mod) {
	const { MAX_ZOOM, baseScale, centred, clampOffset, fitFrame, moveBy, resize, rotateClockwise, scaleOf, toCropRect, zoomAround, zoomCentre, avatarClip } = mod;

	const under = (s, fx, fy) => ({ x: (fx - s.tx) / scaleOf(s), y: (fy - s.ty) / scaleOf(s) });
	const covers = (s, what) => {
		const k = scaleOf(s);
		assert(s.tx <= 1e-9 && s.ty <= 1e-9, `${what}: offset not positive`);
		assert(s.tx + s.iw * k >= s.fw - 1e-9, `${what}: right edge uncovered`);
		assert(s.ty + s.ih * k >= s.fh - 1e-9, `${what}: bottom edge uncovered`);
	};

	close(baseScale(4000, 3000, 300, 300), 0.1, 'baseScale square frame');
	close(baseScale(3000, 4000, 300, 300), 0.1, 'baseScale portrait');
	close(baseScale(4000, 3000, 600, 200), 0.15, 'baseScale wide frame');

	const c0 = centred(4000, 3000, 300, 300);
	for (const [dx, dy] of [[1e6, 1e6], [-1e6, -1e6], [1e6, -1e6], [-1e6, 1e6]]) covers(moveBy(c0, dx, dy), `drag ${dx},${dy}`);
	const tight = centred(600, 200, 300, 100);
	const pinnedA = moveBy(tight, 50, 50);
	const pinnedB = moveBy(tight, -50, -50);
	assert(pinnedA.tx === 0 && pinnedA.ty === 0 && pinnedB.tx === 0 && pinnedB.ty === 0, 'a barely covering image is pinned');
	covers(clampOffset({ ...centred(400, 300, 100, 100), zoom: 2, tx: 500, ty: -500 }), 'stale offset');

	close(c0.tx, -(4000 * 0.1 - 300) / 2, 'centred overflowing axis');
	close(c0.ty, 0, 'centred other axis');
	assert(c0.zoom === 1 && c0.rotate === 0, 'centred starts at zoom 1, rotate 0');

	let z = zoomAround(c0, 2, 150, 150);
	const [fx, fy] = [220, 90];
	const before = under(z, fx, fy);
	const zoomed = zoomAround(z, 3.5, fx, fy);
	const after = under(zoomed, fx, fy);
	close(after.x, before.x, 'zoom keeps focal x');
	close(after.y, before.y, 'zoom keeps focal y');
	covers(zoomed, 'zoomed');

	const sq = centred(1000, 1000, 200, 200);
	assert(zoomAround(sq, 100, 0, 0).zoom === MAX_ZOOM, 'zoom clamps high');
	assert(zoomAround(sq, 0.2, 0, 0).zoom === 1, 'zoom clamps low');
	assert(zoomCentre(sq, 2).zoom === 2, 'zoomCentre');
	z = zoomAround(centred(1000, 800, 200, 200), 4, 10, 10);
	z = zoomAround(z, 1, 10, 10);
	covers(z, 'zoomed out');
	assert(z.zoom === 1, 'zoomed out to 1');

	eq(toCropRect(centred(4000, 3000, 300, 300)), { x: 500, y: 0, w: 3000, h: 3000, rotate: 0 }, 'landscape rect');
	eq(toCropRect(centred(3000, 4000, 300, 300)), { x: 0, y: 500, w: 3000, h: 3000, rotate: 0 }, 'portrait rect');
	eq(toCropRect(centred(3000, 3000, 450, 150)), { x: 0, y: 1000, w: 3000, h: 1000, rotate: 0 }, 'banner rect');
	eq(toCropRect(zoomCentre(centred(4000, 3000, 300, 300), 2)), { x: 1250, y: 750, w: 1500, h: 1500, rotate: 0 }, 'zoomed rect');
	assert(toCropRect(moveBy(centred(4000, 3000, 300, 300), 10, 0)).x === 400, 'rect follows a drag');

	const odd = toCropRect(zoomAround(centred(1001, 777, 123, 41), 1.37, 55.5, 7.3));
	for (const v of [odd.x, odd.y, odd.w, odd.h]) assert(Number.isInteger(v), 'integers');
	assert(odd.x >= 0 && odd.y >= 0 && odd.x + odd.w <= 1001 && odd.y + odd.h <= 777, 'inside the image');
	eq(toCropRect(centred(1, 1, 300, 300)), { x: 0, y: 0, w: 1, h: 1, rotate: 0 }, 'tiny image');
	assert(toCropRect(zoomCentre(centred(2, 2, 300, 300), MAX_ZOOM)).w >= 1, 'at least one pixel');

	const wide = centred(5000, 700, 300, 100);
	for (const zoom of [1, 2.5, MAX_ZOOM]) {
		for (const d of [-1e6, 1e6]) {
			const r = toCropRect(moveBy(zoomCentre(wide, zoom), d, d));
			assert(r.x >= 0 && r.y >= 0 && r.x + r.w <= 5000 && r.y + r.h <= 700, `extreme zoom ${zoom} drag ${d}`);
		}
	}

	const turned = rotateClockwise(zoomAround(centred(4000, 3000, 300, 300), 3, 20, 20));
	assert(turned.iw === 3000 && turned.ih === 4000 && turned.zoom === 1 && turned.rotate === 90, 'rotate swaps and resets');
	eq(toCropRect(turned), { x: 0, y: 500, w: 3000, h: 3000, rotate: 90 }, 'rotated rect');
	covers(turned, 'rotated');
	let spin = centred(4000, 3000, 300, 100);
	for (const turn of [90, 180, 270, 0]) {
		spin = rotateClockwise(spin);
		assert(spin.rotate === turn, `rotation wraps to ${turn}`);
	}
	eq(spin, centred(4000, 3000, 300, 100), 'four turns come back');

	const zoomedIn = zoomAround(centred(4000, 3000, 300, 300), 2.5, 100, 200);
	const centre = under(zoomedIn, 150, 150);
	const grown = resize(zoomedIn, 450, 450);
	const centreAfter = under(grown, 225, 225);
	close(centreAfter.x, centre.x, 'resize keeps centre x');
	close(centreAfter.y, centre.y, 'resize keeps centre y');
	covers(grown, 'resized');
	eq(toCropRect(resize(c0, 600, 600)), toCropRect(c0), 'proportional resize keeps the rect');

	eq(fitFrame(400, 400, 1, 20), { w: 360, h: 360 }, 'fitFrame square');
	eq(fitFrame(400, 400, 3, 20), { w: 360, h: 120 }, 'fitFrame banner');
	eq(fitFrame(400, 200, 1, 20), { w: 160, h: 160 }, 'fitFrame height bound');

	// FNV-1a of the clip strings the Rust side (`ui::avatar::avatar_clip`) pins as well.
	for (const [shape, size, hash] of [
		['circle', 24, 1_385_578_626], ['circle', 40, 4_262_497_958], ['circle', 96, 1_674_221_384],
		['square', 24, 1_702_449_770], ['square', 40, 891_136_910], ['square', 96, 1_949_923_734],
		['squircle', 24, 1_009_482_006], ['squircle', 40, 2_293_843_278], ['squircle', 96, 2_835_309_624],
		['octagon', 24, 3_202_210_622], ['octagon', 40, 1_543_386_194], ['octagon', 96, 3_333_118_258]
	]) {
		assert(fnv1a(avatarClip(shape, size)) === hash, `avatarClip ${shape} ${size}`);
	}
}
