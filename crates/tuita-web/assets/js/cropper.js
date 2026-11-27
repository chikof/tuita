// The image cropper and the slider island: pointer, wheel and key gestures over a picture under a fixed frame.
//
// Markup contract (rendered by `ui::cropper`):
//
//   <form class="cropper" data-island="cropper" data-kind="avatar|banner" data-shape="circle|square|squircle|octagon">
//     <div class="stage" role="application" tabindex="0">        the island fills it with the picture
//     <input type="range" data-role="zoom">                      zoom slider (its label/output follow `input` events)
//     <button data-act="zoom-out|zoom-in|rotate|reset|cancel">   tool buttons
//     <button type="submit">                                     Apply
//     <input type="hidden" name="crop">                          receives the CropRect as JSON
//
// Talking to the island, all on the form element:
//   in   `tuita-crop-load`    CustomEvent, detail { file: File }: show this picture
//   in   attribute `data-busy`  while the result is being sent: controls are off
//   out  `tuita-crop`         bubbling CustomEvent, detail = CropRect { x, y, w, h, rotate } in pixels of
//                             the image rotated by `rotate` (the server does the cropping); `ev.data` is
//                             the same JSON string for Topcoat handlers; the hidden input holds it too
//   out  `tuita-crop-cancel`  bubbling CustomEvent when the cancel button is pressed

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

/** Highest zoom, relative to the scale at which the image just covers the frame. */
export const MAX_ZOOM = 6;

const clamp = (v, lo, hi) => Math.min(hi, Math.max(lo, v));

/** Scale at which an `iw`×`ih` image just covers an `fw`×`fh` frame. */
export const baseScale = (iw, ih, fw, fh) => Math.max(fw / iw, fh / ih);

/**
 * A crop state is an image under a fixed frame, in CSS pixels: `iw`/`ih` are the image size AFTER
 * rotating by `rotate` degrees clockwise, `tx`/`ty` place the image's top-left corner relative to
 * the frame's top-left corner, so a covering image has `tx` in `[fw - iw * s, 0]`.
 */
export const scaleOf = (s) => baseScale(s.iw, s.ih, s.fw, s.fh) * s.zoom;

/** Keeps the frame covered by the image. */
export function clampOffset(s) {
	const k = scaleOf(s);
	return {
		...s,
		tx: Math.min(0, Math.max(s.fw - s.iw * k, s.tx)),
		ty: Math.min(0, Math.max(s.fh - s.ih * k, s.ty))
	};
}

/** Zoom 1, image centred under the frame. `iw`/`ih` are the size after `rotate`. */
export function centred(iw, ih, fw, fh, rotate = 0) {
	const k = baseScale(iw, ih, fw, fh);
	return clampOffset({ iw, ih, fw, fh, zoom: 1, tx: (fw - iw * k) / 2, ty: (fh - ih * k) / 2, rotate });
}

/** Moves the image by a screen delta. */
export const moveBy = (s, dx, dy) => clampOffset({ ...s, tx: s.tx + dx, ty: s.ty + dy });

/** Sets the zoom (clamped to `[1, MAX_ZOOM]`) keeping the frame point (`fx`, `fy`) still. */
export function zoomAround(s, zoom, fx, fy) {
	const before = scaleOf(s);
	const next = { ...s, zoom: clamp(zoom, 1, MAX_ZOOM) };
	const ratio = scaleOf(next) / before;
	return clampOffset({ ...next, tx: fx - (fx - s.tx) * ratio, ty: fy - (fy - s.ty) * ratio });
}

/** Zoom about the frame's centre. */
export const zoomCentre = (s, zoom) => zoomAround(s, zoom, s.fw / 2, s.fh / 2);

/** Quarter turn clockwise: swaps the image's width and height, resets zoom and centres. */
export const rotateClockwise = (s) => centred(s.ih, s.iw, s.fw, s.fh, (s.rotate + 90) % 360);

/** A new frame size; the image point under the frame's centre stays there. */
export function resize(s, fw, fh) {
	const k = scaleOf(s);
	const cx = (s.fw / 2 - s.tx) / k;
	const cy = (s.fh / 2 - s.ty) / k;
	const next = { ...s, fw, fh };
	const k2 = scaleOf(next);
	return clampOffset({ ...next, tx: fw / 2 - cx * k2, ty: fh / 2 - cy * k2 });
}

/** The frame's area in pixels of the rotated image, as the server's `crop` field wants it. */
export function toCropRect(s) {
	const k = scaleOf(s);
	const w = clamp(Math.round(s.fw / k), 1, Math.max(1, Math.round(s.iw)));
	const h = clamp(Math.round(s.fh / k), 1, Math.max(1, Math.round(s.ih)));
	return {
		x: clamp(Math.round(-s.tx / k), 0, Math.max(0, Math.round(s.iw) - w)),
		y: clamp(Math.round(-s.ty / k), 0, Math.max(0, Math.round(s.ih) - h)),
		w,
		h,
		rotate: s.rotate
	};
}

/** The largest frame of `aspect` (w/h) that fits `width`×`height` after `pad` on every side. */
export function fitFrame(width, height, aspect, pad) {
	let w = Math.max(1, width - pad * 2);
	let h = w / aspect;
	const room = Math.max(1, height - pad * 2);
	if (h > room) {
		h = room;
		w = h * aspect;
	}
	return { w, h };
}

const snap = (v, grid) => Math.round(v / grid) * grid;

function polygon(points) {
	const unique = points.filter((p, i) => i === 0 || p[0] !== points[i - 1][0] || p[1] !== points[i - 1][1]);
	return `polygon(${unique.map(([x, y]) => `${x}px ${y}px`).join(', ')})`;
}

/** Points of |x/a|^n + |y/a|^n = 1 sampled `count` times and snapped to the 2px grid. */
function superellipse(size, n, count) {
	const half = size / 2;
	return Array.from({ length: count }, (_, i) => {
		const t = (2 * Math.PI * i) / count;
		const c = Math.cos(t);
		const s = Math.sin(t);
		const x = half + half * Math.sign(c) * Math.abs(c) ** (2 / n);
		const y = half + half * Math.sign(s) * Math.abs(s) ** (2 / n);
		return [Math.min(size, snap(x, 2)), Math.min(size, snap(y, 2))];
	});
}

/** CSS `clip-path` for an avatar of `size` px. Every vertex lies on the 2px grid. */
export function avatarClip(shape, size) {
	switch (shape) {
		case 'circle':
			return polygon(superellipse(size, 2, 48));
		case 'squircle':
			return polygon(superellipse(size, 4, 32));
		case 'octagon': {
			const c = snap(size * 0.29, 2);
			return polygon([
				[c, 0], [size - c, 0], [size, c], [size, size - c],
				[size - c, size], [c, size], [0, size - c], [0, c]
			]);
		}
		default: {
			const c = Math.max(2, snap(size / 16, 2));
			const k = 2 * c;
			return polygon([
				[0, k], [c, k], [c, c], [k, c], [k, 0], [size - k, 0], [size - k, c], [size - c, c], [size - c, k],
				[size, k], [size, size - k], [size - c, size - k], [size - c, size - c], [size - k, size - c],
				[size - k, size], [k, size], [k, size - c], [c, size - c], [c, size - k], [0, size - k]
			]);
		}
	}
}

const ZOOM_STEP = 1.25;

const MOVES = {
	ArrowLeft: [1, 0],
	ArrowRight: [-1, 0],
	ArrowUp: [0, 1],
	ArrowDown: [0, -1]
};

function mountCropper(root) {
	const stage = root.querySelector('.stage');
	const slider = root.querySelector('input[data-role="zoom"]');
	const apply = root.querySelector('button[type="submit"]');
	const hidden = root.querySelector('input[name="crop"]');
	const act = (name) => root.querySelector(`[data-act="${name}"]`);
	if (!stage) return;

	const kind = root.dataset.kind === 'banner' ? 'banner' : 'avatar';
	const shape = root.dataset.shape ?? 'circle';
	const aspect = kind === 'avatar' ? 1 : 3;

	stage.insertAdjacentHTML(
		'beforeend',
		'<div class="layer" aria-hidden="true" style="visibility:hidden"><div class="sheet"><img alt="" draggable="false"></div></div>' +
			'<div class="veil" aria-hidden="true"></div>' +
			'<div class="frame" aria-hidden="true" hidden><div class="lit"><div class="sheet"><img alt="" draggable="false"></div></div></div>'
	);
	const layer = stage.querySelector('.layer');
	const frameEl = stage.querySelector('.frame');
	const lit = stage.querySelector('.lit');
	const sheets = [...stage.querySelectorAll('.sheet')];
	const imgs = [...stage.querySelectorAll('img')];

	let box = { w: 0, h: 0 };
	let natural = null;
	let crop = null;
	let url = '';
	let busy = root.hasAttribute('data-busy');
	let frame = { w: 0, h: 0 };
	let offX = 0;
	let offY = 0;

	const busyOrIdle = () => busy || !crop;

	const sync = () => {
		const zoom = crop ? Math.round(crop.zoom * 100) / 100 : 1;
		const off = busyOrIdle();
		act('zoom-out')?.toggleAttribute('disabled', off || zoom <= 1);
		act('zoom-in')?.toggleAttribute('disabled', off || zoom >= MAX_ZOOM);
		act('rotate')?.toggleAttribute('disabled', off);
		act('reset')?.toggleAttribute('disabled', off);
		act('cancel')?.toggleAttribute('disabled', busy);
		apply?.toggleAttribute('disabled', off);
		if (apply) {
			if (busy) apply.setAttribute('aria-busy', 'true');
			else apply.removeAttribute('aria-busy');
		}
		if (slider) {
			slider.disabled = off;
			if (Number(slider.value) !== zoom) {
				slider.value = String(zoom);
				slider.dispatchEvent(new Event('input', { bubbles: true }));
			}
		}
	};

	const paint = () => {
		layer.style.left = `${offX}px`;
		layer.style.top = `${offY}px`;
		layer.style.visibility = crop ? '' : 'hidden';
		frameEl.hidden = !crop;
		if (crop && natural) {
			frameEl.style.left = `${offX}px`;
			frameEl.style.top = `${offY}px`;
			frameEl.style.width = `${frame.w}px`;
			frameEl.style.height = `${frame.h}px`;
			lit.style.clipPath = kind === 'avatar' ? avatarClip(shape, Math.round(frame.w)) : '';
			const transform = `translate(${crop.tx}px, ${crop.ty}px) scale(${scaleOf(crop)})`;
			for (const sheet of sheets) {
				sheet.style.width = `${crop.iw}px`;
				sheet.style.height = `${crop.ih}px`;
				sheet.style.transform = transform;
			}
			for (const img of imgs) {
				img.style.width = `${natural.w}px`;
				img.style.height = `${natural.h}px`;
				img.style.left = `${(crop.iw - natural.w) / 2}px`;
				img.style.top = `${(crop.ih - natural.h) / 2}px`;
				img.style.transform = `rotate(${crop.rotate}deg)`;
			}
		}
		sync();
	};

	const layout = () => {
		frame = fitFrame(box.w, box.h, aspect, Math.max(16, box.w * 0.06));
		offX = (box.w - frame.w) / 2;
		offY = (box.h - frame.h) / 2;
		if (natural && box.w >= 2) {
			crop = crop ? resize(crop, frame.w, frame.h) : centred(natural.w, natural.h, frame.w, frame.h);
		}
		paint();
	};

	const set = (next) => {
		crop = next;
		paint();
	};

	const observer = new ResizeObserver(([entry]) => {
		box = { w: entry.contentRect.width, h: entry.contentRect.height };
		layout();
	});
	observer.observe(stage);

	imgs[0].addEventListener('load', () => {
		natural = { w: imgs[0].naturalWidth, h: imgs[0].naturalHeight };
		layout();
	});

	const load = (event) => {
		const file = event.detail?.file;
		if (!file) return;
		if (url) URL.revokeObjectURL(url);
		url = URL.createObjectURL(file);
		natural = null;
		crop = null;
		for (const img of imgs) img.src = url;
		paint();
	};
	root.addEventListener('tuita-crop-load', load);

	const toFrame = (clientX, clientY) => {
		const r = stage.getBoundingClientRect();
		return { x: clientX - r.left - offX, y: clientY - r.top - offY };
	};

	const pointers = new Map();
	let pinch = null;
	const spread = () => {
		const [a, b] = [...pointers.values()];
		return Math.hypot(a.x - b.x, a.y - b.y);
	};

	stage.addEventListener('pointerdown', (event) => {
		if (busy || !crop) return;
		stage.setPointerCapture(event.pointerId);
		pointers.set(event.pointerId, { x: event.clientX, y: event.clientY });
		stage.classList.add('dragging');
		if (pointers.size === 2) pinch = { dist: spread() || 1, zoom: crop.zoom };
	});

	stage.addEventListener('pointermove', (event) => {
		const prev = pointers.get(event.pointerId);
		if (!prev || !crop) return;
		const next = { x: event.clientX, y: event.clientY };
		pointers.set(event.pointerId, next);
		if (pointers.size === 1) {
			set(moveBy(crop, next.x - prev.x, next.y - prev.y));
		} else if (pinch && pointers.size === 2) {
			const [a, b] = [...pointers.values()];
			const mid = toFrame((a.x + b.x) / 2, (a.y + b.y) / 2);
			set(zoomAround(crop, pinch.zoom * (spread() / pinch.dist), mid.x, mid.y));
		}
	});

	const release = (event) => {
		pointers.delete(event.pointerId);
		if (pointers.size < 2) pinch = null;
		if (!pointers.size) stage.classList.remove('dragging');
	};
	stage.addEventListener('pointerup', release);
	stage.addEventListener('pointercancel', release);

	stage.addEventListener(
		'wheel',
		(event) => {
			event.preventDefault();
			if (busy || !crop) return;
			const at = toFrame(event.clientX, event.clientY);
			const delta = event.deltaMode === 1 ? event.deltaY * 16 : event.deltaY;
			set(zoomAround(crop, crop.zoom * Math.exp(-delta * 0.0015), at.x, at.y));
		},
		{ passive: false }
	);

	stage.addEventListener('keydown', (event) => {
		if (busy || !crop) return;
		const move = MOVES[event.key];
		if (move) {
			event.preventDefault();
			const step = event.shiftKey ? 40 : 10;
			set(moveBy(crop, move[0] * step, move[1] * step));
		} else if (event.key === '+' || event.key === '=') {
			event.preventDefault();
			set(zoomCentre(crop, crop.zoom * ZOOM_STEP));
		} else if (event.key === '-' || event.key === '_') {
			event.preventDefault();
			set(zoomCentre(crop, crop.zoom / ZOOM_STEP));
		}
	});

	root.addEventListener('click', (event) => {
		const button = event.target instanceof Element ? event.target.closest('[data-act]') : null;
		if (!button || !root.contains(button)) return;
		const name = button.dataset.act;
		if (name === 'cancel') {
			root.dispatchEvent(new CustomEvent('tuita-crop-cancel', { bubbles: true }));
			return;
		}
		if (busy || !crop) return;
		if (name === 'zoom-in') set(zoomCentre(crop, crop.zoom * ZOOM_STEP));
		else if (name === 'zoom-out') set(zoomCentre(crop, crop.zoom / ZOOM_STEP));
		else if (name === 'rotate') set(rotateClockwise(crop));
		else if (name === 'reset' && natural) set(centred(natural.w, natural.h, crop.fw, crop.fh));
	});

	slider?.addEventListener('input', () => {
		if (!crop) return;
		const zoom = Number(slider.value);
		if (Math.round(crop.zoom * 100) / 100 !== zoom) set(zoomCentre(crop, zoom));
	});

	root.addEventListener('submit', (event) => {
		event.preventDefault();
		if (!crop || busy) return;
		const rect = toCropRect(crop);
		const json = JSON.stringify(rect);
		if (hidden) hidden.value = json;
		const out = new CustomEvent('tuita-crop', { bubbles: true, detail: rect });
		out.data = json;
		root.dispatchEvent(out);
	});

	const watch = new MutationObserver(() => {
		busy = root.hasAttribute('data-busy');
		sync();
	});
	watch.observe(root, { attributes: true, attributeFilter: ['data-busy'] });
	sync();

	return () => {
		observer.disconnect();
		watch.disconnect();
		if (url) URL.revokeObjectURL(url);
	};
}

mount('[data-island="cropper"]', mountCropper);

// Slider fill and value. `ui::slider` renders `<div data-island="slider" style="--pct:..">` around a
// range input and an `<output>`; both follow the input here. Any `input` event counts, so a script
// that sets `input.value` can dispatch one to redraw. The cropper's zoom control is one of these.
function mountSlider(root) {
	const input = root.querySelector('input[type="range"]');
	const output = root.querySelector('output');
	if (!input) return;
	const sync = () => {
		const min = Number(input.min || 0);
		const max = Number(input.max || 100);
		const pct = max === min ? 0 : ((Number(input.value) - min) / (max - min)) * 100;
		root.style.setProperty('--pct', `${pct}%`);
		if (output) output.textContent = input.value;
	};
	input.addEventListener('input', sync);
	return () => input.removeEventListener('input', sync);
}

mount('[data-island="slider"]', mountSlider);
