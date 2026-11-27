// Known-answer vectors for assets/js/lightbox.js.
const assert = (ok, what) => {
	if (!ok) throw new Error(`assertion failed: ${what}`);
};
const eq = (actual, expected, what) => {
	if (JSON.stringify(actual) !== JSON.stringify(expected)) {
		throw new Error(`${what}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
	}
};

export async function run(mod) {
	const { GAP, MAX_HEIGHT, PEEK, aspectOf, carouselLayout, currentIndex, scrollTarget, scrolls, starts, wrapIndex } = mod;
	const portrait = { width: 800, height: 1200 };
	const landscape = { width: 1600, height: 900 };
	const ultrawide = { width: 3000, height: 1000 };
	const unknown = { width: null, height: null };

	// carouselLayout
	{
		const { height, widths } = carouselLayout([portrait, portrait, portrait], 600);
		eq(height, 430, 'two portrait items and a peek: height');
		eq(widths, [286, 286, 286], 'two portrait items and a peek: widths');
		const lefts = starts(widths);
		assert(lefts[2] < 600 && lefts[2] >= 600 - PEEK, 'the third item peeks');
	}
	{
		const { widths } = carouselLayout([portrait, portrait], 600);
		assert(widths.reduce((sum, width) => sum + width, 0) + GAP <= 600, 'two portrait items fit');
		eq(scrolls(widths, 600), false, 'two portrait items do not scroll');
	}
	{
		const { height, widths } = carouselLayout([ultrawide, ultrawide, ultrawide, ultrawide], 600);
		eq(height, 337, 'wide items crop to 16:9');
		eq(widths, [576, 576, 576, 576], 'the next one peeks');
		assert(widths.every((width) => width <= 600 - PEEK), 'wide items leave the peek');
	}
	{
		const { height, widths } = carouselLayout([landscape, portrait, landscape], 600);
		eq(height, 337, 'the first item sets the height');
		eq(widths[0], 576, 'landscape width');
		eq(widths[1], Math.floor(337 * (800 / 1200)), 'portrait width');
		eq(widths[2], 576, 'landscape width again');
		const mixed = carouselLayout([portrait, landscape], 600);
		eq(mixed.height, 448, 'portrait first, landscape second: height');
		eq(mixed.widths[1], 576, 'portrait first, landscape second: width');
	}
	{
		assert(Math.abs(aspectOf(unknown) - 4 / 3) < 1e-9, 'no size is 4:3');
		assert(Math.abs(aspectOf({ width: 100, height: null }) - 4 / 3) < 1e-9, 'no height is 4:3');
		assert(Math.abs(aspectOf({ width: 0, height: 0 }) - 4 / 3) < 1e-9, 'zero size is 4:3');
		const { height, widths } = carouselLayout([unknown, unknown], 600);
		eq(height, 450, 'unknown items: height');
		eq(widths, [576, 576], 'unknown items: widths');
	}
	{
		const thin = { width: 500, height: 2000 };
		eq(carouselLayout([thin, thin, thin], 1200).height, MAX_HEIGHT, 'the height is capped');
		const narrow = carouselLayout([portrait, portrait, portrait], 300);
		eq(narrow.height, Math.floor((300 - GAP - PEEK) / (2 * (2 / 3))), 'a narrow column shrinks the strip');
		assert(narrow.widths.every((width) => width < 300), 'narrow widths');
	}
	eq(carouselLayout([portrait, portrait], 0), { height: 0, widths: [0, 0] }, 'unmeasured column');
	eq(carouselLayout([], 600), { height: 0, widths: [] }, 'no items');

	// scrolling
	{
		const widths = [286, 286, 286, 286];
		const viewport = 600;
		const lefts = starts(widths);
		const max = widths.length * 286 + GAP * 3 - viewport;
		eq(scrollTarget(widths, 0, viewport, 1), lefts[1], 'step to the next start');
		eq(scrollTarget(widths, lefts[3], viewport, 1), max, 'step stops at the end');
		eq(scrollTarget(widths, max, viewport, 1), max, 'step stays at the end');
		eq(scrollTarget(widths, lefts[2], viewport, -1), lefts[1], 'step back');
		eq(scrollTarget(widths, 0, viewport, -1), 0, 'step back stops at zero');
		eq(currentIndex(widths, 0, viewport), 0, 'first item');
		eq(currentIndex(widths, lefts[1], viewport), 1, 'second item');
		eq(currentIndex(widths, max, viewport), 3, 'the far end names the last item');
		eq(scrolls(widths, viewport), true, 'the strip overflows');
		eq(scrolls([200, 200], viewport), false, 'a short strip does not');
		eq(currentIndex([200, 200], 0, viewport), 0, 'a short strip is at its first item');
	}

	// wrapIndex (state/lightbox)
	eq(wrapIndex(0, -1, 4), 3, 'wraps below zero');
	eq(wrapIndex(3, 1, 4), 0, 'wraps past the end');
	eq(wrapIndex(1, 1, 4), 2, 'steps');
	eq(wrapIndex(0, 1, 0), 0, 'no items');
}
