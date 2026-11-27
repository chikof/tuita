// The dither engine: canvas fields, the animated dither number and the dither-text mask.
// The maths below are copies of the
// Rust side (`ui/hash.rs`, `ui/mask.rs`) and must stay bit-identical to it.
//
// Markup contract (rendered by `ui::dither_field`, `ui::dither_number`, `ui::dither_text`):
//
//   <canvas aria-hidden="true" data-island="dither-field" data-field="waves" ...>
//     data-field   waves (default) | radial | linear | noise | ripples | orb | seeded
//     data-seed    seeded: any string (the field kind and its parameters come from its FNV-1a hash)
//                  noise, ripples: a decimal number is used as is, any other text is hashed
//     data-cx data-cy   radial centre, 0..1 (default 0.5)
//     data-angle        linear angle in degrees (default 0)
//     data-cell    CSS pixels per dither cell (default 6)
//     data-matrix  Bayer matrix size 2 | 4 | 8 (default 4)
//     data-animate "false" draws one still frame (default animated)
//     data-speed   multiplier on the demo's 0.72 time units per second (default 1)
//     data-still-time  time of the still frame (default 1)
//   The canvas takes its ink from its CSS `color`; `data-opacity` is informational (the server
//   also writes it as inline `opacity`).
//
//   <span data-island="dither-number" data-value="1234" data-format="plain|compact" data-interval="40">
//     <span aria-hidden="true"><span data-shown>1234</span></span><span class="sr-only">1234</span>
//   </span>
//   The `[data-shown]` text counts up from 0 to `data-value` (unless motion is reduced) and ends
//   on the server-rendered text.
//
//   <span class="text" data-island="dither-text" data-from="1" data-to="0.45" style="mask-image:...">
//   One mask row per 2px of height: the island recomputes `mask-image` and `--rows` whenever the
//   element's height changes (the server renders the mask for a 48px tall element).

function mount(selector, init) {
	const live = new Map();
	const scan = (root) => {
		if (root.nodeType !== 1) return;
		const found = root.matches(selector) ? [root, ...root.querySelectorAll(selector)] : root.querySelectorAll(selector);
		for (const el of found) if (!live.has(el)) live.set(el, init(el) ?? null);
	};
	const sweep = () => { for (const [el, stop] of live) if (!el.isConnected) { stop?.(); live.delete(el); } };
	scan(document.documentElement);
	new MutationObserver((records) => {
		for (const r of records) for (const n of r.addedNodes) scan(n);
		sweep();
	}).observe(document.documentElement, { childList: true, subtree: true });
}

/** 32-bit FNV-1a over the UTF-16 code units of `input`. */
export function fnv1a(input) {
	let h = 0x811c9dc5;
	for (let i = 0; i < input.length; i++) {
		h ^= input.charCodeAt(i);
		h = Math.imul(h, 0x01000193);
	}
	return h >>> 0;
}

/** Deterministic PRNG returning floats in [0, 1). */
export function mulberry32(seed) {
	let a = seed >>> 0;
	return () => {
		a = (a + 0x6d2b79f5) >>> 0;
		let t = a;
		t = Math.imul(t ^ (t >>> 15), t | 1);
		t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
		return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
	};
}

const ranksCache = new Map();
const thresholdCache = new Map();

/** Bayer rank matrix (row-major), built recursively: M(2n) = [[4M, 4M+2], [4M+3, 4M+1]]. */
export function bayerRanks(n) {
	const cached = ranksCache.get(n);
	if (cached) return cached;
	let ranks;
	if (n === 1) {
		ranks = new Uint16Array(1);
	} else {
		const h = n / 2;
		const prev = bayerRanks(h);
		ranks = new Uint16Array(n * n);
		for (let y = 0; y < n; y++) {
			for (let x = 0; x < n; x++) {
				const quadrant = y < h ? (x < h ? 0 : 2) : x < h ? 3 : 1;
				ranks[y * n + x] = 4 * prev[(y % h) * h + (x % h)] + quadrant;
			}
		}
	}
	ranksCache.set(n, ranks);
	return ranks;
}

/** Thresholds `(rank + 0.5) / n²`, row-major. A cell is lit when its tone exceeds its threshold. */
export function bayer(n) {
	const cached = thresholdCache.get(n);
	if (cached) return cached;
	const ranks = bayerRanks(n);
	const out = new Float32Array(ranks.length);
	for (let i = 0; i < ranks.length; i++) out[i] = (ranks[i] + 0.5) / ranks.length;
	thresholdCache.set(n, out);
	return out;
}

const maskCache = new Map();

const dataUrl = (svg) =>
	`url("data:image/svg+xml,${svg.replace(/</g, '%3C').replace(/>/g, '%3E').replace(/#/g, '%23')}")`;

const svgOf = (w, h, rects) =>
	`<svg xmlns='http://www.w3.org/2000/svg' width='${w}' height='${h}' viewBox='0 0 ${w} ${h}' shape-rendering='crispEdges'>${rects}</svg>`;

const rect = (x, y) => `<rect x='${x}' y='${y}' width='1' height='1'/>`;

/**
 * A dithered ramp from `from` to `to` (tones 0..1) over `steps` cells along `axis` ('x' or 'y'),
 * n cells across, as a CSS `url()`. Stretch it with `mask-size`, repeat along the other axis.
 */
export function rampMask(from, to, steps, axis, n = 4) {
	const key = `r${axis}${n}:${steps}:${from.toFixed(3)}:${to.toFixed(3)}`;
	const cached = maskCache.get(key);
	if (cached) return cached;
	const thresholds = bayer(n);
	let rects = '';
	for (let i = 0; i < steps; i++) {
		const tone = from + ((to - from) * (i + 0.5)) / steps;
		for (let j = 0; j < n; j++) {
			const [x, y] = axis === 'x' ? [i, j] : [j, i];
			if (tone > thresholds[(y % n) * n + (x % n)]) rects += rect(x, y);
		}
	}
	const url = dataUrl(axis === 'x' ? svgOf(steps, n, rects) : svgOf(n, steps, rects));
	maskCache.set(key, url);
	return url;
}

// A field is (x, y, t, aspect) => tone 0..1 at normalised position and time `t`
// (about 0.72 per second at speed 1); `aspect` is width / height.

const clamp01 = (v) => (v < 0 ? 0 : v > 1 ? 1 : v);

/** Cells across the reference hero (500px at cell 6) the demo constants were tuned for. */
const REF = 84;

const wavesWith = (fx, fy, fd, sx, sy) => (x, y, t, aspect) => {
	const X = x * REF;
	const Y = (y * REF) / aspect;
	return (
		0.5 +
		0.28 * Math.sin(X * fx + t * sx) +
		0.22 * Math.sin(Y * fy - t * sy) +
		0.18 * Math.sin((X + Y) * fd + t)
	);
};

/** The demo's flowing luminance field. */
export const waves = wavesWith(0.11, 0.14, 0.07, 2.1, 1.4);

export const radial =
	(cx = 0.5, cy = 0.5) =>
	(x, y, t, aspect) => {
		const d = Math.hypot((x - cx) * aspect, y - cy);
		return clamp01(1.05 - d * 1.5 + 0.08 * Math.sin(t * 2.4));
	};

export const linear = (angleDeg) => {
	const a = (angleDeg * Math.PI) / 180;
	const ca = Math.cos(a);
	const sa = Math.sin(a);
	return (x, y, t, aspect) => {
		const p = (x - 0.5) * aspect * ca + (y - 0.5) * sa;
		return clamp01(0.5 + p * 0.9 + 0.1 * Math.sin(t * 1.3 + p * 4));
	};
};

const lattice = (seed, gx, gy) => {
	let h = (Math.imul(gx, 0x27d4eb2d) ^ Math.imul(gy, 0x165667b1) ^ seed) >>> 0;
	h = Math.imul(h ^ (h >>> 15), 0x85ebca6b);
	h = Math.imul(h ^ (h >>> 13), 0xc2b2ae35);
	return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
};

const valueNoise = (seed, x, y) => {
	const gx = Math.floor(x);
	const gy = Math.floor(y);
	const fx = x - gx;
	const fy = y - gy;
	const u = fx * fx * (3 - 2 * fx);
	const v = fy * fy * (3 - 2 * fy);
	const top = lattice(seed, gx, gy) * (1 - u) + lattice(seed, gx + 1, gy) * u;
	const bottom = lattice(seed, gx, gy + 1) * (1 - u) + lattice(seed, gx + 1, gy + 1) * u;
	return top * (1 - v) + bottom * v;
};

/** Two-octave 2D value noise drifting with time. */
export const noise = (seed) => (x, y, t, aspect) => {
	const X = x * aspect * 5 + t * 0.35;
	const Y = y * 5 - t * 0.2;
	return clamp01(0.65 * valueNoise(seed, X, Y) + 0.35 * valueNoise(seed ^ 0x9e3779b9, X * 2.1, Y * 2.1));
};

/** Concentric rings radiating from a seeded centre. */
export const ripples = (seed) => {
	const rnd = mulberry32(seed);
	const cx = 0.25 + rnd() * 0.5;
	const cy = 0.25 + rnd() * 0.5;
	const k = 14 + rnd() * 10;
	return (x, y, t, aspect) => {
		const d = Math.hypot((x - cx) * aspect, y - cy);
		return clamp01(0.5 + 0.5 * Math.sin(d * k - t * 2.2)) * clamp01(1.25 - d);
	};
};

/** A shaded sphere with a light circling it. */
export const orb = (x, y, t, aspect) => {
	const px = (x - 0.5) * aspect * 2;
	const py = (y - 0.5) * 2;
	const r2 = px * px + py * py;
	if (r2 > 0.64) return 0.04;
	const z = Math.sqrt(0.64 - r2);
	const lx = Math.cos(t * 0.8) * 0.6;
	const ly = Math.sin(t * 0.8) * 0.4 - 0.3;
	return clamp01(0.12 + 0.95 * Math.max(0, (px * lx + py * ly + z * 0.7) / 0.8));
};

/** Picks one of the fields above with parameters drawn from an FNV-1a hash of `handle`. */
export function seeded(handle) {
	const hash = fnv1a(handle);
	const rnd = mulberry32(hash);
	switch (hash % 6) {
		case 0:
			return wavesWith(0.07 + rnd() * 0.1, 0.07 + rnd() * 0.1, 0.04 + rnd() * 0.06, 1 + rnd() * 2, 1 + rnd() * 2);
		case 1:
			return radial(0.2 + rnd() * 0.6, 0.2 + rnd() * 0.6);
		case 2:
			return linear(rnd() * 360);
		case 3:
			return noise(hash);
		case 4:
			return ripples(hash);
		default:
			return orb;
	}
}

/** The seed of a noise or ripples field: a decimal number as is, any other text hashed. */
const seedNumber = (text) => (/^\d+$/.test(text) ? Number(text) >>> 0 : fnv1a(text));

/** The field a `<canvas data-island="dither-field">` asks for. */
export function fieldOf(data) {
	const seed = data.seed ?? '';
	switch (data.field ?? 'waves') {
		case 'radial':
			return radial(Number(data.cx ?? 0.5), Number(data.cy ?? 0.5));
		case 'linear':
			return linear(Number(data.angle ?? 0));
		case 'noise':
			return noise(seedNumber(seed));
		case 'ripples':
			return ripples(seedNumber(seed));
		case 'orb':
			return orb;
		case 'seeded':
			return seeded(seed);
		default:
			return waves;
	}
}

const TIME_PER_SECOND = 0.72;

function readInk(canvas) {
	const channels = getComputedStyle(canvas).color.match(/[\d.]+/g)?.map(Number) ?? [];
	return [channels[0] ?? 242, channels[1] ?? 240, channels[2] ?? 234];
}

const motionReduced = () => document.documentElement.dataset.motion === 'reduce';

/** Dithers `field` onto `canvas` with ink from the canvas's computed `color`. */
export function mountField(canvas, opts) {
	const { field, cell = 6, matrix = 4, animate = true, stillTime = 1, speed = 1 } = opts;
	const ctx = canvas.getContext('2d');
	if (!ctx) return { destroy() {} };

	const thresholds = bayer(matrix);
	canvas.style.imageRendering = 'pixelated';

	let cols = 0;
	let rows = 0;
	let image = null;
	let ink = readInk(canvas);
	let t = stillTime;
	let last = 0;
	let raf = 0;
	let onScreen = true;

	const draw = () => {
		if (!image) return;
		const d = image.data;
		const aspect = cols / rows;
		let i = 0;
		for (let y = 0; y < rows; y++) {
			const row = (y % matrix) * matrix;
			for (let x = 0; x < cols; x++) {
				const tone = field((x + 0.5) / cols, (y + 0.5) / rows, t, aspect);
				d[i] = ink[0];
				d[i + 1] = ink[1];
				d[i + 2] = ink[2];
				d[i + 3] = tone > thresholds[row + (x % matrix)] ? 255 : 0;
				i += 4;
			}
		}
		ctx.putImageData(image, 0, 0);
	};

	const running = () => animate && !motionReduced() && onScreen && !document.hidden;

	const frame = (now) => {
		raf = 0;
		if (!running()) return;
		t += ((now - last) / 1000) * TIME_PER_SECOND * speed;
		last = now;
		draw();
		raf = requestAnimationFrame(frame);
	};

	const sync = () => {
		if (running()) {
			if (!raf) {
				last = performance.now();
				raf = requestAnimationFrame(frame);
			}
		} else {
			if (raf) cancelAnimationFrame(raf);
			raf = 0;
			if (!animate || motionReduced()) {
				t = stillTime;
				draw();
			}
		}
	};

	const resize = () => {
		const rect = canvas.getBoundingClientRect();
		cols = Math.max(1, Math.ceil(rect.width / cell));
		rows = Math.max(1, Math.ceil(rect.height / cell));
		canvas.width = cols;
		canvas.height = rows;
		image = ctx.createImageData(cols, rows);
		draw();
	};

	const resizeObserver = new ResizeObserver(resize);
	resizeObserver.observe(canvas);

	const intersection = new IntersectionObserver((entries) => {
		onScreen = entries[entries.length - 1]?.isIntersecting ?? true;
		sync();
	});
	intersection.observe(canvas);

	const theme = new MutationObserver(() => {
		ink = readInk(canvas);
		draw();
	});
	theme.observe(document.documentElement, {
		attributes: true,
		attributeFilter: ['data-theme', 'data-ink', 'data-contrast']
	});

	const motion = new MutationObserver(sync);
	motion.observe(document.documentElement, { attributes: true, attributeFilter: ['data-motion'] });

	document.addEventListener('visibilitychange', sync);
	resize();
	sync();

	return {
		destroy() {
			if (raf) cancelAnimationFrame(raf);
			raf = 0;
			resizeObserver.disconnect();
			intersection.disconnect();
			theme.disconnect();
			motion.disconnect();
			document.removeEventListener('visibilitychange', sync);
		}
	};
}

/** The `FieldOptions` a canvas's data attributes describe. */
export function optionsOf(data) {
	const num = (value, fallback) => (value === undefined || Number.isNaN(Number(value)) ? fallback : Number(value));
	return {
		field: fieldOf(data),
		cell: num(data.cell, 6),
		matrix: [2, 4, 8].includes(Number(data.matrix)) ? Number(data.matrix) : 4,
		animate: data.animate !== 'false',
		stillTime: num(data.stillTime, 1),
		speed: num(data.speed, 1)
	};
}

/** The demo easing: each tick closes a ninth of the gap, at least one step. */
export function nextShown(shown, target) {
	const gap = target - shown;
	if (gap === 0) return target;
	const step = Math.max(1, Math.ceil(Math.abs(gap) / 9));
	return gap > 0 ? Math.min(target, shown + step) : Math.max(target, shown - step);
}

const formatters = {
	plain: () => String,
	compact: (lang) => {
		const format = new Intl.NumberFormat(lang, { notation: 'compact', maximumFractionDigits: 1 });
		return (n) => format.format(n);
	}
};

function mountNumber(root) {
	const out = root.querySelector('[data-shown]');
	const target = Number(root.dataset.value);
	if (!out || !Number.isFinite(target) || motionReduced()) return;
	const settled = out.textContent;
	const format = (formatters[root.dataset.format] ?? formatters.plain)(document.documentElement.lang || undefined);
	const interval = Number(root.dataset.interval) || 40;
	let shown = 0;
	out.textContent = format(shown);
	const timer = setInterval(() => {
		shown = nextShown(shown, target);
		if (shown === target) {
			clearInterval(timer);
			out.textContent = settled;
		} else {
			out.textContent = format(shown);
		}
	}, interval);
	return () => clearInterval(timer);
}

/** One mask row per `--cell` (2px) of height, so cells stay square at any font size. */
const TEXT_CELL = 2;

function mountText(el) {
	const from = Number(el.dataset.from ?? 1);
	const to = Number(el.dataset.to ?? 0.45);
	let rows = Number(el.dataset.rows) || 0;
	const sync = () => {
		const next = Math.max(4, Math.round(el.clientHeight / TEXT_CELL));
		if (next === rows || el.clientHeight === 0) return;
		rows = next;
		const mask = rampMask(from, to, rows, 'y');
		el.style.maskImage = mask;
		el.style.webkitMaskImage = mask;
		el.style.setProperty('--rows', String(rows));
	};
	const observer = new ResizeObserver(sync);
	observer.observe(el);
	sync();
	return () => observer.disconnect();
}

mount('[data-island="dither-field"]', (canvas) => mountField(canvas, optionsOf(canvas.dataset)).destroy);
mount('[data-island="dither-number"]', mountNumber);
mount('[data-island="dither-text"]', mountText);
