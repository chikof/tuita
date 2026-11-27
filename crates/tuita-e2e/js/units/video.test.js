// Known-answer vectors for assets/js/video.js: formatClock and
// the player's pure maths.
const eq = (actual, expected, what) => {
	if (JSON.stringify(actual) !== JSON.stringify(expected)) {
		throw new Error(`${what}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
	}
};

export async function run(mod) {
	const clock = [0, 8, 15, 75, 599, 3599].map(mod.formatClock);
	eq(clock, ['0:00', '0:08', '0:15', '1:15', '9:59', '59:59'], 'under an hour reads m:ss');
	eq([3600, 3661, 36_000 + 125].map(mod.formatClock), ['1:00:00', '1:01:01', '10:02:05'], 'an hour or more reads h:mm:ss');
	eq([0.9, 14.99, 3599.99].map(mod.formatClock), ['0:00', '0:14', '59:59'], 'the fraction is dropped');
	eq([NaN, Infinity, -Infinity, -5].map(mod.formatClock), ['0:00', '0:00', '0:00', '0:00'], 'NaN, Infinity and negatives read 0:00');

	eq([NaN, Infinity, 0, 1].map(mod.isSeekable), [false, false, false, true], 'seekable durations');
	eq([mod.playedPercent(30, 120), mod.playedPercent(500, 120), mod.playedPercent(5, NaN)], [25, 100, 0], 'played percent');
	const ranges = [{ start: 0, end: 10 }, { start: 20, end: 50 }];
	eq([mod.bufferedPercent(ranges, 5, 100), mod.bufferedPercent(ranges, 25, 100), mod.bufferedPercent(ranges, 15, 100), mod.bufferedPercent(ranges, 5, NaN)], [10, 50, 0, 0], 'buffered percent');
	eq([mod.positionAt(150, 100, 200, 60), mod.positionAt(0, 100, 200, 60), mod.positionAt(999, 100, 200, 60), mod.positionAt(5, 0, 0, 60)], [15, 0, 60, 0], 'scrub position');

	eq([' ', 'k', 'K', 'm', 'f', 'ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown', 'x'].map((key) => mod.frameKey(key, true)), ['toggle', 'toggle', 'toggle', 'mute', 'fullscreen', 'back', 'forward', 'louder', 'quieter', null], 'frame keys');
	eq(mod.frameKey(' ', false), null, 'a space on a child control is its own');
	eq(['ArrowLeft', 'ArrowDown', 'ArrowRight', 'ArrowUp', 'Home', 'End', 'a'].map(mod.scrubKey), ['back', 'back', 'forward', 'forward', 'start', 'end', null], 'slider keys');
}
