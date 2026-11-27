// The video player island. Markup contract (`post/video_player.rs`): a root
// `[data-island="video"]` (`.player`) holds a `<video controls>` and a static control bar.
// Without this script the native controls are the player. The island removes `controls`, adds
// `live` to the root and mirrors the playback state as classes on the root: `paused`, `muted`,
// `fs`, `pip`, `loading`, `big`, `failed`, `quiet`, `can-pip`, `can-fs`; the CSS shows the control
// that matches. Controls are `[data-act="toggle|mute|pip|fullscreen|retry"]`, the seek slider is
// `.scrub` (`.fill`, `.buffer` inside), the volume `.vol input` and the speed menu items
// `[role="menuitem"][data-id="<rate>"]`. `data-autoplay` starts playback once live.

const SEEK_STEP = 5;
const VOLUME_STEP = 0.1;
const IDLE_MS = 2500;

/** The one video allowed to play: a second `play` pauses the first. */
let leader;

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

export const clamp = (value, low, high) => Math.min(high, Math.max(low, value));

/** A media position as `m:ss`, or `h:mm:ss` from one hour; NaN, Infinity and negatives read `0:00`. */
export function formatClock(seconds) {
	const total = Number.isFinite(seconds) && seconds > 0 ? Math.floor(seconds) : 0;
	const hours = Math.floor(total / 3600);
	const minutes = Math.floor((total % 3600) / 60);
	const rest = String(total % 60).padStart(2, '0');
	return hours > 0 ? `${hours}:${String(minutes).padStart(2, '0')}:${rest}` : `${minutes}:${rest}`;
}

/** Whether a duration can be sought in. */
export const isSeekable = (duration) => Number.isFinite(duration) && duration > 0;

/** `shown` seconds of `duration` as 0..100; 0 when the duration is unknown. */
export const playedPercent = (shown, duration) => (isSeekable(duration) ? clamp(shown / duration, 0, 1) * 100 : 0);

/** The end of the buffered range that holds `time`, as 0..100 of `duration`; 0 when none does. */
export function bufferedPercent(ranges, time, duration) {
	if (!isSeekable(duration)) return 0;
	const range = ranges.find((r) => r.start <= time && time <= r.end);
	return range ? clamp(range.end / duration, 0, 1) * 100 : 0;
}

/** The position in seconds under `clientX` on a rail that starts at `left` and is `width` wide. */
export const positionAt = (clientX, left, width, duration) => (width > 0 ? clamp((clientX - left) / width, 0, 1) * duration : 0);

/** What a key does on the player frame: an action name, or null. A space only counts on the frame itself. */
export function frameKey(key, onFrame) {
	switch (key) {
		case ' ': return onFrame ? 'toggle' : null;
		case 'k': case 'K': return 'toggle';
		case 'm': case 'M': return 'mute';
		case 'f': case 'F': return 'fullscreen';
		case 'ArrowLeft': return 'back';
		case 'ArrowRight': return 'forward';
		case 'ArrowUp': return 'louder';
		case 'ArrowDown': return 'quieter';
		default: return null;
	}
}

/** What a key does on the seek slider: `back`, `forward`, `start`, `end`, or null. */
export function scrubKey(key) {
	switch (key) {
		case 'ArrowLeft': case 'ArrowDown': return 'back';
		case 'ArrowRight': case 'ArrowUp': return 'forward';
		case 'Home': return 'start';
		case 'End': return 'end';
		default: return null;
	}
}

function positionText(current, total) {
	let template;
	try {
		template = JSON.parse(document.getElementById('tuita-l10n')?.textContent ?? '{}').video_position;
	} catch {
		template = undefined;
	}
	return (template ?? '{current} / {total}').replace('{current}', current).replace('{total}', total);
}

function init(root) {
	const video = root.querySelector('video');
	if (!video) return;
	const part = (name) => root.querySelector(`[data-part="${name}"]`);
	const bar = root.querySelector('.bar');
	const scrub = root.querySelector('.scrub');
	const buffer = root.querySelector('.buffer');
	const fill = root.querySelector('.fill');
	const vol = root.querySelector('.vol');
	const volume = root.querySelector('.vol input');
	const rateLabel = root.querySelector('.rate');
	const now = part('now');
	const total = part('total');
	const motion = () => document.documentElement.dataset.motion;

	let started = false;
	let finished = false;
	let failed = false;
	let awake = false;
	let menuOpen = false;
	let scrubTo = null;
	let resume = 0;
	let timer;

	const buffered = () => Array.from({ length: video.buffered.length }, (_, i) => ({ start: video.buffered.start(i), end: video.buffered.end(i) }));

	function render() {
		const { duration } = video;
		const seekable = isSeekable(duration);
		const shown = scrubTo ?? video.currentTime;
		const level = video.muted ? 0 : video.volume;
		const loading = !failed && !video.paused && (video.readyState < 3 || video.seeking);
		const barShown = video.paused || awake || menuOpen || scrubTo !== null;
		const flags = {
			paused: video.paused,
			muted: level === 0,
			fs: document.fullscreenElement === root,
			pip: document.pictureInPictureElement === video,
			loading,
			big: !failed && !loading && (!started || finished),
			failed,
			quiet: !video.paused && !barShown
		};
		for (const [name, on] of Object.entries(flags)) root.classList.toggle(name, on);
		bar.classList.toggle('shown', barShown);

		now.textContent = formatClock(shown);
		total.textContent = formatClock(duration);
		scrub.classList.toggle('off', !seekable);
		scrub.setAttribute('aria-disabled', String(!seekable));
		scrub.setAttribute('aria-valuemax', String(seekable ? Math.floor(duration) : 0));
		scrub.setAttribute('aria-valuenow', String(Math.floor(shown)));
		scrub.setAttribute('aria-valuetext', positionText(formatClock(shown), formatClock(duration)));
		fill.style.width = `${playedPercent(shown, duration)}%`;
		buffer.style.width = `${bufferedPercent(buffered(), video.currentTime, duration)}%`;

		vol.style.setProperty('--pct', `${level * 100}%`);
		if (document.activeElement !== volume) volume.value = String(level);

		const rate = video.playbackRate;
		rateLabel.textContent = rate === 1 ? '' : `${rate}×`;
		for (const item of root.querySelectorAll('[role="menuitem"][data-id]')) {
			const current = Number(item.dataset.id) === rate;
			const mark = item.querySelector('.meta');
			if (current && !mark) item.insertAdjacentHTML('beforeend', '<span class="meta">●</span>');
			else if (!current) mark?.remove();
		}
	}

	function wake() {
		awake = true;
		clearTimeout(timer);
		timer = setTimeout(() => { awake = false; render(); }, IDLE_MS);
		render();
	}

	function sleep() {
		clearTimeout(timer);
		awake = false;
		render();
	}

	const play = () => void video.play().catch(() => {});
	const toggle = () => (video.paused ? play() : video.pause());

	function seekBy(delta) {
		if (!isSeekable(video.duration)) return;
		video.currentTime = clamp(video.currentTime + delta, 0, video.duration);
	}

	function setVolume(delta) {
		if (delta > 0) video.muted = false;
		video.volume = clamp(video.volume + delta, 0, 1);
		if (video.volume === 0) video.muted = true;
	}

	function toggleMute() {
		if (video.muted || video.volume === 0) {
			video.muted = false;
			if (video.volume === 0) video.volume = 0.5;
		} else {
			video.muted = true;
		}
	}

	async function toggleFullscreen() {
		try {
			if (document.fullscreenElement) await document.exitFullscreen();
			else if (root.requestFullscreen) await root.requestFullscreen();
			else video.webkitEnterFullscreen?.();
		} catch {
			// The browser refused (no user gesture, policy): nothing to show.
		}
	}

	async function togglePicture() {
		try {
			if (document.pictureInPictureElement === video) await document.exitPictureInPicture();
			else await video.requestPictureInPicture();
		} catch {
			// Metadata not loaded yet or the browser refused: nothing to show.
		}
	}

	function retry() {
		resume = video.currentTime;
		failed = false;
		video.load();
		play();
		render();
	}

	const actions = {
		toggle,
		mute: toggleMute,
		pip: togglePicture,
		fullscreen: toggleFullscreen,
		retry
	};

	root.addEventListener('click', (event) => {
		const target = event.target instanceof Element ? event.target : null;
		const control = target?.closest('[data-act]');
		if (control && root.contains(control)) {
			actions[control.dataset.act]?.();
			return;
		}
		const item = target?.closest('[role="menuitem"][data-id]');
		if (item) {
			video.playbackRate = Number(item.dataset.id);
			return;
		}
		if (target === video) {
			if (event.pointerType === 'touch' && !video.paused && !bar.classList.contains('shown')) wake();
			else toggle();
		}
	});

	root.addEventListener('keydown', (event) => {
		if (event.ctrlKey || event.metaKey || event.altKey) return;
		const target = event.target;
		if (target.closest('input, [role="menu"]')) return;
		const action = frameKey(event.key, target === root);
		if (!action) return;
		if (action === 'toggle') toggle();
		else if (action === 'mute') toggleMute();
		else if (action === 'fullscreen') void toggleFullscreen();
		else if (action === 'back') seekBy(-SEEK_STEP);
		else if (action === 'forward') seekBy(SEEK_STEP);
		else if (action === 'louder') setVolume(VOLUME_STEP);
		else setVolume(-VOLUME_STEP);
		event.preventDefault();
		wake();
	});

	root.addEventListener('pointermove', (event) => event.pointerType !== 'touch' && wake());
	root.addEventListener('pointerleave', (event) => event.pointerType !== 'touch' && sleep());
	root.addEventListener('toggle', (event) => {
		menuOpen = event.newState === 'open';
		render();
	}, true);

	const at = (event) => {
		const box = scrub.getBoundingClientRect();
		return positionAt(event.clientX, box.left, box.width, video.duration);
	};
	scrub.addEventListener('pointerdown', (event) => {
		if (!isSeekable(video.duration) || event.button !== 0) return;
		scrub.setPointerCapture(event.pointerId);
		scrubTo = at(event);
		video.currentTime = scrubTo;
		render();
	});
	scrub.addEventListener('pointermove', (event) => {
		if (scrubTo === null) return;
		scrubTo = at(event);
		video.currentTime = scrubTo;
		render();
	});
	const release = () => {
		scrubTo = null;
		render();
	};
	scrub.addEventListener('pointerup', release);
	scrub.addEventListener('pointercancel', release);
	scrub.addEventListener('keydown', (event) => {
		if (!isSeekable(video.duration) || event.ctrlKey || event.metaKey || event.altKey) return;
		const action = scrubKey(event.key);
		if (!action) return;
		if (action === 'back') seekBy(-SEEK_STEP);
		else if (action === 'forward') seekBy(SEEK_STEP);
		else video.currentTime = action === 'start' ? 0 : video.duration;
		event.preventDefault();
		event.stopPropagation();
		wake();
	});

	volume.addEventListener('input', () => {
		const next = Number(volume.value);
		video.volume = next;
		video.muted = next === 0;
	});

	video.addEventListener('play', () => {
		if (leader && leader !== video) leader.pause();
		leader = video;
		started = true;
		finished = false;
	});
	video.addEventListener('ended', () => {
		finished = true;
		render();
	});
	video.addEventListener('error', () => {
		failed = true;
		render();
	});
	video.addEventListener('loadedmetadata', () => {
		if (resume > 0) video.currentTime = resume;
		resume = 0;
	});
	for (const name of ['play', 'pause', 'playing', 'waiting', 'seeking', 'seeked', 'canplay', 'timeupdate', 'durationchange', 'loadedmetadata', 'progress', 'volumechange', 'ratechange', 'emptied']) {
		video.addEventListener(name, render);
	}
	const onFullscreen = () => render();
	const onPicture = () => render();
	document.addEventListener('fullscreenchange', onFullscreen);
	video.addEventListener('enterpictureinpicture', onPicture);
	video.addEventListener('leavepictureinpicture', onPicture);

	video.removeAttribute('controls');
	root.classList.toggle('can-fs', Boolean(document.fullscreenEnabled || 'webkitEnterFullscreen' in video));
	root.classList.toggle('can-pip', Boolean(document.pictureInPictureEnabled && !video.disablePictureInPicture));
	root.classList.add('live');
	// An error that fired before this script ran leaves the element in the failed state.
	failed = video.error !== null;
	render();

	if ('autoplay' in root.dataset && !(video.loop && motion() === 'reduce')) play();

	return () => {
		clearTimeout(timer);
		document.removeEventListener('fullscreenchange', onFullscreen);
		if (leader === video) leader = undefined;
	};
}

if (typeof document !== 'undefined' && !document.documentElement.dataset.videoBound) {
	document.documentElement.dataset.videoBound = '';
	mount('[data-island="video"]', init);
}
