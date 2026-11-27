// The /design gallery. Appearance prefs are plain `[data-pref]` buttons handled by prefs.js; this
// module only carries what the gallery adds on top: `[data-design-act="reset-banners"]` forgets
// every announcement dismissal (`tuita:ann:*`) and brings the dismissed banners back.

const ANN_PREFIX = 'tuita:ann:';

/** Every `tuita:ann:<id>` key, so the next page view shows the announcements again. */
export function forgetDismissals(store) {
	const keys = [];
	for (let i = 0; i < store.length; i++) {
		const key = store.key(i);
		if (key?.startsWith(ANN_PREFIX)) keys.push(key);
	}
	for (const key of keys) store.removeItem(key);
	return keys.length;
}

if (typeof document !== 'undefined') {
	document.addEventListener('click', (event) => {
		const button = event.target instanceof Element ? event.target.closest('[data-design-act]') : null;
		if (!button) return;
		if (button.dataset.designAct === 'reset-banners') {
			try {
				forgetDismissals(localStorage);
			} catch {
				// Storage may be unavailable; the banners still come back on reload.
			}
			location.reload();
		}
	});
}

// Gallery demos of the core groups (actions, forms, overlays, feedback, identity, dither). Plain
// hooks in the markup, no state of their own:
//   [data-design-act="toast"][data-message][data-label][data-href][data-counter]   raises a toast
//   [data-design-counter="<max>"]  a text field and a char counter: the counter follows the typing
//   [data-design-progress]         the range input drives the first progress bar
//   [data-design-act="roll"]       rolls every [data-design-target] number to a random 0..100
//   [data-design-clock]            hour, minute and second spans showing the browser's time
{
	const pad = (n) => String(n).padStart(2, '0');
	const paintSegments = (filled, rects) => rects.forEach((rect, i) => rect.classList.toggle('on', i < filled));

	const toastCounts = new WeakMap();
	const UNDO_HASH = '#toast-undo';

	function toast(detail) {
		document.dispatchEvent(new CustomEvent('tuita-toast', { bubbles: true, detail }));
	}

	function paintCounter(box) {
		const input = box.querySelector('[data-design-counter-input]');
		const counter = box.querySelector('.counter');
		if (!input || !counter) return;
		const max = Number(box.dataset.designCounter);
		const remaining = max - input.value.length;
		const over = remaining < 0;
		counter.classList.toggle('over', over);
		const n = Math.abs(remaining);
		counter.setAttribute('aria-label', `${n} ${n === 1 ? 'character' : 'characters'} ${over ? 'over the limit' : 'left'}`);
		const rects = [...counter.querySelectorAll('rect')];
		paintSegments(Math.min(rects.length, Math.ceil((Math.max(input.value.length, 0) / max) * rects.length)), rects);
		let number = counter.querySelector('.number');
		if (remaining <= 20) {
			if (!number) {
				number = document.createElement('span');
				number.className = 'number';
				number.setAttribute('aria-hidden', 'true');
				counter.append(number);
			}
			number.textContent = String(remaining);
		} else {
			number?.remove();
		}
	}

	function paintProgress(box, input) {
		const bar = box.querySelector('[role="progressbar"]');
		const fill = bar?.querySelector('.fill');
		if (!bar || !fill) return;
		bar.setAttribute('aria-valuenow', input.value);
		fill.style.width = `${input.value}%`;
	}

	/** Counts a dither number to `target` the way `dither.js` does: each tick closes a ninth of the gap. */
	function roll(el, target) {
		const shown = el.querySelector('[data-shown]');
		const spoken = el.querySelector('.sr-only');
		el.dataset.value = String(target);
		if (spoken) spoken.textContent = String(target);
		if (!shown) return;
		clearInterval(Number(el.dataset.rolling));
		const reduced = document.documentElement.dataset.motion === 'reduce' || matchMedia('(prefers-reduced-motion: reduce)').matches;
		if (reduced) {
			shown.textContent = String(target);
			return;
		}
		let now = Number(shown.textContent.replace(/\D/g, '')) || 0;
		const timer = setInterval(() => {
			const gap = target - now;
			now = Math.abs(gap) < 1 ? target : now + Math.sign(gap) * Math.max(1, Math.round(Math.abs(gap) / 9));
			shown.textContent = String(now);
			if (now === target) clearInterval(timer);
		}, 40);
		el.dataset.rolling = String(timer);
	}

	document.addEventListener('click', (event) => {
		const button = event.target instanceof Element ? event.target.closest('[data-design-act]') : null;
		if (!button) return;
		if (button.dataset.designAct === 'toast') {
			let message = button.dataset.message;
			if (button.hasAttribute('data-counter')) {
				const count = (toastCounts.get(button) ?? 0) + 1;
				toastCounts.set(button, count);
				message = `${message} (${count})`;
			}
			toast({ message, label: button.dataset.label, href: button.dataset.href });
		} else if (button.dataset.designAct === 'roll') {
			const target = Math.round(Math.random() * 100);
			for (const el of document.querySelectorAll('[data-design-target] [data-island="dither-number"]')) roll(el, target);
		}
	});

	// The toast's action follows `href`: the Undo action lands on a hash, which raises the second toast.
	addEventListener('hashchange', () => {
		if (location.hash !== UNDO_HASH) return;
		history.replaceState(null, '', `${location.pathname}${location.search}#toast`);
		toast('Restored');
	});

	document.addEventListener('input', (event) => {
		const target = event.target;
		if (!(target instanceof Element)) return;
		const counter = target.closest('[data-design-counter]');
		if (counter) paintCounter(counter);
		const progress = target.closest('[data-design-progress]');
		if (progress && target.matches('input[type="range"]')) paintProgress(progress, target);
	});

	const tick = () => {
		const now = new Date();
		for (const clock of document.querySelectorAll('[data-design-clock]')) {
			for (const [part, value] of [['h', now.getHours()], ['m', now.getMinutes()], ['s', now.getSeconds()]]) {
				const span = clock.querySelector(`[data-part="${part}"]`);
				if (span) span.textContent = pad(value);
			}
		}
	};
	if (document.querySelector('[data-design-clock]')) {
		tick();
		setInterval(tick, 1000);
	}
}

// The media and encryption sections of the /design gallery. The components are the ones of the
// app and their islands (`cropper.js`, `upload.js`, `lightbox.js`, `video.js`) run them; this
// adds the state the gallery demos need. Hooks, all `data-design-*`:
//
//   [data-design-sample="w,h,hue,name"]   holds a cropper form that gets that generated picture
//   [data-design-rect]                    shows the rectangle of the last `tuita-crop`
//   [data-design-tray="w,h,hue,name;.."]  holds the media tray that gets those pictures
//   [data-design-dropped]                 lists the names of the files dropped on its dropzone
//   [data-design-sandbox]                 a demo area: its forms never submit, `[data-design-open=key]`
//                                         buttons add a copy of `template[data-design-template=key]`
//                                         to it and open it, as `post.js` and `lightbox.js` do with
//                                         the fragments of the server
//   [data-design-poll]                    the poll editor demo (add and remove answers or the poll)
//   [data-design-emoji]                   an emoji picker that closes on a pick
//   [data-design-schedule]                a schedule picker with its presets and a value readout
//   [data-design-block="ready"]           a chat footer whose composer is on
//   [data-design-number]                  the safety number shown by `#safety-number`
(() => {
	const all = (selector, root = document) => [...root.querySelectorAll(selector)];

	const strings = () => {
		try {
			return JSON.parse(document.getElementById('tuita-l10n')?.textContent ?? '{}');
		} catch {
			return {};
		}
	};

	// ---- generated pictures --------------------------------------------------------------------

	function sampleFile(width, height, hue, name) {
		const canvas = document.createElement('canvas');
		canvas.width = width;
		canvas.height = height;
		const g = canvas.getContext('2d');
		const sky = g.createLinearGradient(0, 0, 0, height);
		sky.addColorStop(0, `hsl(${hue} 80% 70%)`);
		sky.addColorStop(1, `hsl(${(hue + 40) % 360} 85% 45%)`);
		g.fillStyle = sky;
		g.fillRect(0, 0, width, height);
		g.fillStyle = `hsl(${(hue + 180) % 360} 90% 85%)`;
		g.beginPath();
		g.arc(width * 0.72, height * 0.28, height * 0.12, 0, Math.PI * 2);
		g.fill();
		g.fillStyle = `hsl(${(hue + 200) % 360} 45% 25%)`;
		g.beginPath();
		g.moveTo(0, height);
		g.lineTo(0, height * 0.7);
		g.lineTo(width * 0.25, height * 0.5);
		g.lineTo(width * 0.45, height * 0.68);
		g.lineTo(width * 0.7, height * 0.42);
		g.lineTo(width, height * 0.72);
		g.lineTo(width, height);
		g.fill();
		return new Promise((resolve) =>
			canvas.toBlob((blob) => resolve(new File([blob], name, { type: 'image/png' })), 'image/png')
		);
	}

	const sampleOf = (spec) => {
		const [width, height, hue, name] = spec.split(',');
		return sampleFile(Number(width), Number(height), Number(hue), name);
	};

	for (const holder of all('[data-design-sample]')) {
		const form = holder.querySelector('form[data-island="cropper"]');
		if (!form) continue;
		void sampleOf(holder.dataset.designSample).then((file) =>
			form.dispatchEvent(new CustomEvent('tuita-crop-load', { detail: { file } }))
		);
	}

	document.addEventListener('tuita-crop', (event) => {
		if (!(event.target instanceof Element) || !event.target.closest('[data-design-sample]')) return;
		const out = document.querySelector('[data-design-rect]');
		if (out) out.textContent = event.data ?? JSON.stringify(event.detail);
	});

	// The tray uploads through XHR; a request that never answers keeps each picture "uploading",
	// and nothing reaches the server.
	class PendingRequest {
		upload = {};
		open() {}
		setRequestHeader() {}
		send() {}
		abort() {
			this.onabort?.();
		}
	}

	for (const holder of all('[data-design-tray]')) {
		const tray = holder.querySelector('[data-island="media-tray"]');
		if (!tray) continue;
		void Promise.all(holder.dataset.designTray.split(';').map(sampleOf)).then((files) => {
			if (typeof tray.attach !== 'function') return;
			const real = window.XMLHttpRequest;
			window.XMLHttpRequest = PendingRequest;
			try {
				tray.attach(files);
			} finally {
				window.XMLHttpRequest = real;
			}
		});
	}

	document.addEventListener('tuita-files', (event) => {
		const zone = event.target instanceof Element ? event.target.closest('[data-c="dropzone"]') : null;
		const out = zone?.querySelector('[data-design-dropped]');
		if (!out) return;
		const names = (event.detail?.files ?? []).map((file) => file.name);
		out.textContent = names.length ? names.join(', ') : 'nothing dropped yet';
	});

	// ---- composer attachments ------------------------------------------------------------------

	for (const input of all('[data-design-block="ready"] input[disabled]')) input.disabled = false;

	function syncPoll(root) {
		const options = all('.option', root);
		const add = root.querySelector('[data-role="add-option"]');
		if (add) add.hidden = options.every((option) => !option.hidden);
	}

	function pollClick(root, target) {
		const editor = root.querySelector('[data-design-poll-editor]');
		const again = root.querySelector('[data-design-poll-add]');
		const act = target.closest('[data-act]')?.dataset.act ?? target.closest('[data-design-act]')?.dataset.designAct;
		if (act === 'add-option') {
			const next = all('.option', root).find((option) => option.hidden);
			if (next) next.hidden = false;
		} else if (act === 'remove-option') {
			const option = target.closest('.option');
			if (!option) return;
			option.hidden = true;
			const field = option.querySelector('input');
			if (field) field.value = '';
		} else if (act === 'remove-poll') {
			editor.hidden = true;
			again.hidden = false;
			return;
		} else if (act === 'add-poll') {
			editor.hidden = false;
			again.hidden = true;
		} else {
			return;
		}
		syncPoll(root);
	}

	// ---- emoji and schedule pickers ------------------------------------------------------------

	const pad = (n) => String(n).padStart(2, '0');
	const localInput = (date) =>
		`${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}`;

	/** In an hour, 20:00 today (when at least 2 h away), 09:00 tomorrow, in a week at 09:00. */
	function presets(now = new Date()) {
		const at = (days, hour) => {
			const d = new Date(now);
			d.setDate(d.getDate() + days);
			d.setHours(hour, 0, 0, 0);
			return d;
		};
		const list = [{ id: 'hour', at: new Date(now.getTime() + 3_600_000) }];
		const evening = at(0, 20);
		if (evening.getTime() - now.getTime() >= 2 * 3_600_000) list.push({ id: 'evening', at: evening });
		list.push({ id: 'tomorrow', at: at(1, 9) }, { id: 'week', at: at(7, 9) });
		return list;
	}

	for (const root of all('[data-design-schedule]')) {
		const panel = root.querySelector('[data-role="schedule-panel"]')?.closest('[popover]');
		const readout = root.querySelector('[data-design-schedule-value]');
		if (!panel || !readout) continue;
		const custom = panel.querySelector('[data-field="custom"]');
		const setButton = panel.querySelector('[data-act="schedule-set"]');
		const clear = panel.querySelector('[data-role="clear"]');
		const failure = panel.querySelector('[data-role="failure"]');
		const template = panel.querySelector('template[data-tpl="preset"]');

		const choose = (iso) => {
			readout.textContent = iso ?? 'not scheduled';
			clear.hidden = iso === null;
			failure.hidden = true;
			panel.hidePopover();
		};

		panel.addEventListener('toggle', (event) => {
			if (event.newState !== 'open') return;
			const table = strings();
			const buttons = presets().map(({ id, at }) => {
				const button = template.content.firstElementChild.cloneNode(true);
				button.querySelector('[data-role="name"]').textContent = table[`schedule_${id}`] ?? id;
				button.querySelector('[data-role="when"]').textContent = at.toLocaleString([], {
					dateStyle: 'medium',
					timeStyle: 'short'
				});
				button.dataset.at = at.toISOString();
				return button;
			});
			panel.querySelector('[data-role="presets"]').replaceChildren(...buttons);
		});
		custom.addEventListener('input', () => (setButton.disabled = !custom.value));
		panel.addEventListener('click', (event) => {
			const target = event.target instanceof Element ? event.target : null;
			const preset = target?.closest('[data-role="presets"] button');
			if (preset) return choose(preset.dataset.at);
			if (target?.closest('[data-act="schedule-set"]')) {
				const picked = new Date(custom.value);
				if (!Number.isNaN(picked.getTime())) choose(picked.toISOString());
			} else if (target?.closest('[data-act="schedule-clear"]')) {
				choose(null);
			}
		});
	}

	// ---- dialogs and the lightbox --------------------------------------------------------------

	function openCopy(key, index) {
		const template = document.querySelector(`template[data-design-template="${key}"]`);
		const host = template?.parentElement;
		const dialog = template?.content.firstElementChild?.cloneNode(true);
		if (!host || !dialog) return;
		for (const old of all(':scope > dialog', host)) old.remove();
		if (index !== undefined) dialog.dataset.index = String(index);
		host.append(dialog);
		dialog.addEventListener('close', () => dialog.remove(), { once: true });
		if (!dialog.open) dialog.showModal();
	}

	function openReschedule(card) {
		const dialog = document.getElementById('reschedule');
		const field = document.getElementById('reschedule-when');
		if (!dialog || !field) return;
		const at = new Date(card?.dataset.publishAt ?? '');
		if (!Number.isNaN(at.getTime())) field.value = localInput(at);
		dialog.showModal();
	}

	function showNumber(button) {
		const number = button.closest('[data-design-number]')?.dataset.designNumber;
		const list = document.querySelector('#safety-number ol.number');
		if (!number || !list) return;
		list.replaceChildren(
			...number
				.split(' ')
				.filter(Boolean)
				.map((group) => Object.assign(document.createElement('li'), { textContent: group }))
		);
	}

	document.addEventListener(
		'click',
		(event) => {
			const target = event.target instanceof Element ? event.target : null;
			if (!target) return;
			const opener = target.closest('[data-design-open]');
			if (opener) return openCopy(opener.dataset.designOpen);
			// Every picture of the gallery opens the demo viewer: the ids of its posts are made up.
			const picture = target.closest('[data-act="lightbox"]');
			if (picture && document.querySelector('template[data-design-template="lightbox"]')) {
				event.preventDefault();
				event.stopImmediatePropagation();
				return openCopy('lightbox', Number(picture.dataset.index ?? 0));
			}
			const resched = target.closest('[data-design-sandbox] [data-act="sched-resched"]');
			if (resched) return openReschedule(resched.closest('[data-publish-at]'));
			const safety = target.closest('[commandfor="safety-number"]');
			if (safety) showNumber(safety);
			const poll = target.closest('[data-design-poll]');
			if (poll) pollClick(poll, target);
			const emoji = target.closest('[data-design-emoji] button[data-emoji]');
			if (emoji) emoji.closest('[popover]')?.hidePopover();
		},
		true
	);

	// A demo form never reaches the API: it only closes its dialog. The passphrase forms are
	// left to `e2ee.js`, which has no account here and answers with its failure text.
	document.addEventListener(
		'submit',
		(event) => {
			const form = event.target instanceof Element ? event.target : null;
			if (!form?.closest('[data-design-sandbox]') || form.closest('[data-prompt]')) return;
			event.preventDefault();
			event.stopImmediatePropagation();
			form.closest('dialog')?.close();
		},
		true
	);
})();

