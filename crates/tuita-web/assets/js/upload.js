// Uploads: XHR with progress, the composer's media tray, the dropzone drag states, multipart forms
// and cropped image uploads.//
// Strings come from `#tuita-l10n`: composer_err_{type,image_size,video_size,max_images,exclusive},
// composer_upload_failed, composer_remove_{image,video}, composer_alt_{image,video,label},
// composer_uploading_video, retry, image_field_{not_image,too_large,saved}.
//
// Island contracts (all `mount`ed, so they also work on nodes added later):
//
// `[data-island="media-tray"]` (rendered by `compose::media_tray`): the files of the post being written.
//   attributes  data-max-video-mb  largest video in MiB
//               data-poll="true"   set by the composer while it has a poll (media and polls exclude each other)
//   children    template[data-tpl="item" | "progress" | "retry"]
//   properties  el.tray    the `MediaTray` (`items`, `images`, `video`, `busy`, `failed`, `ids`, `finalize()`, `clear()`)
//               el.attach(files)   adds files and toasts a refusal for each rejected one
//   out         `tuita-tray`  bubbling CustomEvent after any change, detail
//               { count, images, video, busy, failed }; `ev.data` is the same JSON
//
// `[data-island="dropzone"]` (`ui::dropzone`): while a file drag is over it sets `data-over`, and `data-rejected`
//   when a dragged type is not in `data-accept` (a comma separated MIME list; `image/*` works; empty accepts
//   anything). It does nothing while `data-disabled` is set. A drop dispatches a bubbling `tuita-files` event
//   with `detail.files` (an array of File) on the zone.
//
// `[data-island="multipart-form"]`: a <form> whose submit posts its FormData by XHR to `data-action` (method
//   `data-method`, POST by default). While it runs the form has `aria-busy="true"` and its buttons are disabled; an
//   element `[data-role="progress"]` inside gets `value` (0..100) when it is a <progress>. On failure a toast shows
//   the API's error (or `data-failure`, or the generic text); on success the page reloads.
//
// `[data-island="crop-upload"]`: crops a chosen picture, uploads it with the crop and reloads the page.
//   attributes  data-purpose="avatar|banner"
//   children    input[type=file][data-role="pick"]   the chooser; a `[data-act="pick"]` button inside the island opens it
//               dialog holding the `ui::cropper` form (`cropper.js` runs it)
//   A chosen image is checked (type, 10 MB), sent to the cropper with `tuita-crop-load` and the dialog opens;
//   `tuita-crop` uploads with `purpose` and the rect, then a bubbling `tuita-uploaded` event (detail: the
//   MediaDto) fires, the cookie `tuita_flash=image_field_saved` makes the next page toast it, and the page reloads.

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

export const MAX_IMAGES = 4;
export const MAX_IMAGE_BYTES = 10 * 1024 * 1024;
export const IMAGE_TYPES = ['image/png', 'image/jpeg', 'image/gif', 'image/webp'];
export const VIDEO_TYPES = ['video/mp4', 'video/webm', 'video/quicktime'];
export const ACCEPT_IMAGES = IMAGE_TYPES.join(',');
export const ACCEPT_VIDEO = VIDEO_TYPES.join(',');
export const ACCEPT_MEDIA = `${ACCEPT_IMAGES},${ACCEPT_VIDEO}`;

let cachedStrings;
const strings = () => (cachedStrings ??= JSON.parse(document.getElementById('tuita-l10n')?.textContent ?? '{}'));

/** Fills `{name}` placeholders. */
function fill(template, vars = {}) {
	return String(template).replace(/\{(\w+)\}/g, (hole, name) => (name in vars ? String(vars[name]) : hole));
}

const say = (message) => document.dispatchEvent(new CustomEvent('tuita-toast', { detail: message }));

/** An API failure: the HTTP status (0 when the request never got an answer) and the server's message. */
export class ApiError extends Error {
	constructor(status, message) {
		super(message);
		this.name = 'ApiError';
		this.status = status;
	}
}

function problem(status, text, fallback) {
	try {
		const body = JSON.parse(text);
		if (typeof body.error === 'string' && body.error) return new ApiError(status, body.error);
	} catch {
		// Not JSON: the fallback names the failure.
	}
	return new ApiError(status, fallback);
}

const failureText = () => strings().composer_upload_failed ?? 'Upload failed.';

/**
 * `POST /api/v1/media` as multipart, through XHR because only it reports upload progress. Resolves to the
 * MediaDto. Rejects with `ApiError` (server message) or a `DOMException` `AbortError` when `signal` fires.
 * `make` builds the request object; tests stand in for it.
 */
export function uploadMedia(file, options = {}, fallback = failureText(), make = () => new XMLHttpRequest()) {
	return new Promise((resolve, reject) => {
		const { purpose = 'post', alt, poster, crop, onprogress, signal } = options;
		if (signal?.aborted) return reject(new DOMException('Aborted', 'AbortError'));
		const form = new FormData();
		form.append('file', file, file instanceof File ? file.name : 'upload');
		form.append('purpose', purpose);
		if (alt) form.append('alt', alt);
		if (poster) form.append('poster', poster, 'poster');
		if (crop) form.append('crop', JSON.stringify(crop));

		const xhr = make();
		const abort = () => xhr.abort();
		signal?.addEventListener('abort', abort, { once: true });
		const done = () => signal?.removeEventListener('abort', abort);

		xhr.open('POST', '/api/v1/media');
		xhr.setRequestHeader('accept', 'application/json');
		xhr.upload.onprogress = (event) => {
			if (event.lengthComputable && event.total > 0) onprogress?.(event.loaded / event.total);
		};
		xhr.onload = () => {
			done();
			if (xhr.status >= 200 && xhr.status < 300) {
				try {
					return resolve(JSON.parse(xhr.responseText));
				} catch {
					return reject(new ApiError(xhr.status, fallback));
				}
			}
			reject(problem(xhr.status, xhr.responseText, fallback));
		};
		xhr.onerror = () => {
			done();
			reject(new ApiError(0, fallback));
		};
		xhr.onabort = () => {
			done();
			reject(new DOMException('Aborted', 'AbortError'));
		};
		xhr.send(form);
	});
}

/** Sets the alt text of an upload that is not attached to a post yet. */
export async function describeMedia(id, alt) {
	const res = await fetch(`/api/v1/media/${encodeURIComponent(id)}`, {
		method: 'PATCH',
		credentials: 'same-origin',
		headers: { accept: 'application/json', 'content-type': 'application/json' },
		body: JSON.stringify({ alt })
	});
	if (!res.ok) throw problem(res.status, await res.text(), failureText());
	return res.json();
}

/** `'image'`, `'video'` or `null` for a file the composer cannot use. */
export const kindOf = (file) => (IMAGE_TYPES.includes(file.type) ? 'image' : VIDEO_TYPES.includes(file.type) ? 'video' : null);

/** Why a file of `kind` cannot join the post now, or `null` when it can. */
export function attachBlock(have, kind) {
	if (have.poll) return 'exclusive';
	if (kind === 'video') return have.video || have.images > 0 ? 'exclusive' : null;
	if (have.video) return 'exclusive';
	return have.images >= MAX_IMAGES ? 'max-images' : null;
}

const LONG_EDGE = 1280;
const GIVE_UP_MS = 5000;

/**
 * A still from a video, taken in the browser so the server never decodes video. Resolves to `undefined` when the
 * browser cannot read the file in time; the player then shows its own first frame.
 */
export function grabPoster(file) {
	return new Promise((resolve) => {
		const url = URL.createObjectURL(file);
		const video = document.createElement('video');
		video.muted = true;
		video.preload = 'metadata';
		video.playsInline = true;
		const finish = (blob) => {
			clearTimeout(timer);
			URL.revokeObjectURL(url);
			video.removeAttribute('src');
			video.load();
			resolve(blob);
		};
		const timer = setTimeout(finish, GIVE_UP_MS);
		video.onerror = () => finish();
		video.onloadeddata = () => {
			video.currentTime = Math.min(0.5, (video.duration || 1) / 2);
		};
		video.onseeked = () => {
			try {
				const scale = Math.min(1, LONG_EDGE / Math.max(video.videoWidth, video.videoHeight));
				const canvas = document.createElement('canvas');
				canvas.width = Math.max(1, Math.round(video.videoWidth * scale));
				canvas.height = Math.max(1, Math.round(video.videoHeight * scale));
				canvas.getContext('2d')?.drawImage(video, 0, 0, canvas.width, canvas.height);
				canvas.toBlob(
					(webp) => {
						if (webp?.type === 'image/webp') return finish(webp);
						canvas.toBlob((jpeg) => finish(jpeg ?? undefined), 'image/jpeg', 0.82);
					},
					'image/webp',
					0.82
				);
			} catch {
				finish();
			}
		};
		video.src = url;
	});
}

/**
 * Files attached to a post being written. Each one uploads at once, so a post can only be sent when every item is
 * `done`. Alt text is saved with the post (`finalize`), not per keystroke. `onchange(item)` runs after every
 * change; `item` is the one that changed, or `null` when the list changed.
 */
export class MediaTray {
	items = [];
	#next = 1;
	#deps;
	#onchange;

	constructor(deps, onchange = () => {}) {
		this.#deps = deps;
		this.#onchange = onchange;
	}

	get images() {
		return this.items.filter((i) => i.kind === 'image').length;
	}
	get video() {
		return this.items.some((i) => i.kind === 'video');
	}
	get busy() {
		return this.items.some((i) => i.status === 'uploading');
	}
	get failed() {
		return this.items.some((i) => i.status === 'error');
	}
	get ids() {
		return this.items.flatMap((i) => (i.media ? [i.media.id] : []));
	}

	/** Adds the acceptable files and starts their uploads; returns a refusal for each of the others. */
	add(files) {
		const refusals = [];
		for (const file of files) {
			const kind = kindOf(file);
			if (!kind) {
				refusals.push({ code: 'type' });
				continue;
			}
			const block = attachBlock({ images: this.images, video: this.video, poll: this.#deps.hasPoll() }, kind);
			if (block) {
				refusals.push({ code: block });
				continue;
			}
			if (kind === 'image' && file.size > MAX_IMAGE_BYTES) {
				refusals.push({ code: 'image-size' });
				continue;
			}
			const mb = this.#deps.maxVideoMb();
			if (kind === 'video' && file.size > mb * 1024 * 1024) {
				refusals.push({ code: 'video-size', mb });
				continue;
			}
			const item = {
				key: this.#next++,
				kind,
				file,
				preview: this.#deps.preview(file),
				alt: '',
				progress: 0,
				status: 'uploading',
				media: null,
				error: '',
				control: new AbortController()
			};
			this.items.push(item);
			this.#onchange(null);
			void this.#send(item.key);
		}
		return refusals;
	}

	#find(key) {
		return this.items.find((i) => i.key === key);
	}

	async #send(key) {
		const item = this.#find(key);
		if (!item) return;
		item.status = 'uploading';
		item.progress = 0;
		item.error = '';
		this.#onchange(item);
		try {
			const poster = item.kind === 'video' ? await this.#deps.poster(item.file) : undefined;
			const media = await this.#deps.upload(item.file, {
				poster,
				signal: item.control.signal,
				onprogress: (fraction) => {
					const current = this.#find(key);
					if (current) {
						current.progress = fraction;
						this.#onchange(current);
					}
				}
			});
			const current = this.#find(key);
			if (!current) return;
			current.media = media;
			current.status = 'done';
			current.progress = 1;
			this.#onchange(current);
		} catch (error) {
			if (error instanceof DOMException && error.name === 'AbortError') return;
			const current = this.#find(key);
			if (!current) return;
			current.status = 'error';
			current.error = error instanceof Error ? error.message : String(error);
			this.#onchange(current);
		}
	}

	retry(key) {
		const item = this.#find(key);
		if (item?.status !== 'error') return;
		item.control = new AbortController();
		void this.#send(key);
	}

	remove(key) {
		const item = this.#find(key);
		if (!item) return;
		item.control.abort();
		this.#deps.revoke(item.preview);
		this.items = this.items.filter((i) => i.key !== key);
		this.#onchange(null);
	}

	setAlt(key, alt) {
		const item = this.#find(key);
		if (item) item.alt = alt;
	}

	/** Saves alt text that differs from the uploaded copy; resolves to the ids in order. */
	async finalize() {
		await Promise.all(
			this.items.map(async (item) => {
				if (item.media && item.alt !== item.media.alt) {
					item.media = await this.#deps.describe(item.media.id, item.alt);
				}
			})
		);
		return this.ids;
	}

	clear() {
		for (const item of this.items) {
			item.control.abort();
			this.#deps.revoke(item.preview);
		}
		this.items = [];
		this.#onchange(null);
	}
}

/** The words for one refusal of `MediaTray.add`. */
export function refusalText(refusal, table = strings()) {
	switch (refusal.code) {
		case 'type':
			return table.composer_err_type;
		case 'image-size':
			return table.composer_err_image_size;
		case 'video-size':
			return fill(table.composer_err_video_size, { mb: refusal.mb });
		case 'max-images':
			return table.composer_err_max_images;
		default:
			return table.composer_err_exclusive;
	}
}

const el = (tag, className, attrs = {}) => {
	const node = document.createElement(tag);
	if (className) node.className = className;
	for (const [name, value] of Object.entries(attrs)) node.setAttribute(name, value);
	return node;
};

const clone = (root, name) => root.querySelector(`:scope > template[data-tpl="${name}"]`).content.firstElementChild.cloneNode(true);

function mountTray(root) {
	const table = strings();
	const rows = new Map();
	const percent = (progress) => Math.round(progress * 100);

	const tray = new MediaTray(
		{
			upload: (file, options) => uploadMedia(file, options),
			describe: describeMedia,
			preview: (file) => URL.createObjectURL(file),
			revoke: (url) => URL.revokeObjectURL(url),
			poster: (file) => grabPoster(file),
			maxVideoMb: () => Number(root.dataset.maxVideoMb) || 50,
			hasPoll: () => root.dataset.poll === 'true'
		},
		(item) => (item ? paint(item) : sync())
	);

	function build(item) {
		const row = clone(root, 'item');
		const media = el(item.kind === 'video' ? 'video' : 'img');
		if (item.kind === 'video') {
			media.setAttribute('controls', '');
			media.muted = true;
			media.setAttribute('playsinline', '');
			media.preload = 'metadata';
		} else {
			media.alt = '';
		}
		media.src = item.preview;
		row.querySelector('.frame').prepend(media);
		row.querySelector('.remove button').addEventListener('click', () => tray.remove(item.key));
		const alt = row.querySelector('input.alt');
		alt.placeholder = item.kind === 'video' ? table.composer_alt_video : table.composer_alt_image;
		alt.addEventListener('input', () => tray.setAlt(item.key, alt.value));
		row.querySelector('.remove button').setAttribute('aria-label', item.kind === 'video' ? table.composer_remove_video : table.composer_remove_image);
		return row;
	}

	/** Brings the rows in line with the list: new ones, removed ones, order and labels that name the index. */
	function sync() {
		const keys = new Set(tray.items.map((i) => i.key));
		for (const [key, row] of rows) {
			if (!keys.has(key)) {
				row.remove();
				rows.delete(key);
			}
		}
		tray.items.forEach((item, index) => {
			let row = rows.get(item.key);
			if (!row) {
				row = build(item);
				rows.set(item.key, row);
			}
			row.querySelector('input.alt').setAttribute('aria-label', fill(table.composer_alt_label, { index: index + 1 }));
			paint(item);
		});
		root.append(...tray.items.map((item) => rows.get(item.key)));
		root.classList.toggle('single', tray.items.length === 1);
		root.hidden = tray.items.length === 0;
		announce();
	}


	function paint(item) {
		const row = rows.get(item.key);
		if (!row) return;
		const status = row.querySelector('.status');
		status.dataset.state = item.status;
		if (item.status === 'done') {
			status.hidden = true;
			status.replaceChildren();
		} else if (item.status === 'uploading') {
			status.hidden = false;
			status.classList.remove('failed');
			status.removeAttribute('role');
			let bar = status.querySelector('[role="progressbar"]');
			if (!bar) {
				status.replaceChildren();
				bar = clone(root, 'progress');
				status.append(bar);
				if (item.kind === 'video') status.append(el('span', 'note'));
			}
			const value = percent(item.progress);
			if (item.progress > 0) {
				bar.setAttribute('aria-valuenow', String(value));
				bar.querySelector('.comet')?.remove();
				let fillBar = bar.querySelector('.fill');
				if (!fillBar) {
					fillBar = el('i', 'fill', { 'aria-hidden': 'true' });
					bar.append(fillBar);
				}
				fillBar.style.width = `${value}%`;
			}
			const note = status.querySelector('.note');
			if (note) note.textContent = fill(table.composer_uploading_video, { percent: value });
		} else {
			status.hidden = false;
			status.classList.add('failed');
			status.setAttribute('role', 'alert');
			const note = el('span', 'note');
			note.textContent = item.error;
			const retry = clone(root, 'retry');
			retry.addEventListener('click', () => tray.retry(item.key));
			status.replaceChildren(note, retry);
		}
		announce();
	}

	function announce() {
		const detail = { count: tray.items.length, images: tray.images, video: tray.video, busy: tray.busy, failed: tray.failed };
		const event = new CustomEvent('tuita-tray', { bubbles: true, detail });
		event.data = JSON.stringify(detail);
		root.dispatchEvent(event);
	}

	root.tray = tray;
	root.attach = (files) => {
		const refusals = tray.add(files);
		for (const refusal of refusals) say(refusalText(refusal, table));
		return refusals;
	};
	return () => {
		tray.clear();
		delete root.tray;
		delete root.attach;
	};
}

mount('[data-island="media-tray"]', mountTray);

/** Whether `type` is in a `data-accept` list (`image/*`, `video/mp4`; empty accepts anything). */
export function accepts(list, type) {
	const wanted = list.split(',').map((s) => s.trim()).filter(Boolean);
	if (wanted.length === 0) return true;
	return wanted.some((w) => (w.endsWith('/*') ? type.startsWith(w.slice(0, -1)) : w === type));
}

function mountDropzone(zone) {
	const carriesFiles = (event) => event.dataTransfer?.types.includes('Files') ?? false;
	const off = () => zone.dataset.disabled === 'true';
	const clear = () => {
		zone.removeAttribute('data-over');
		zone.removeAttribute('data-rejected');
	};

	const onenter = (event) => {
		if (off() || !carriesFiles(event)) return;
		event.preventDefault();
		zone.setAttribute('data-over', '');
		const items = [...(event.dataTransfer?.items ?? [])].filter((i) => i.kind === 'file');
		const rejected = items.some((i) => !accepts(zone.dataset.accept ?? '', i.type));
		if (rejected) zone.setAttribute('data-rejected', '');
		else zone.removeAttribute('data-rejected');
	};
	const onleave = (event) => {
		const next = event.relatedTarget;
		if (next instanceof Node && zone.contains(next)) return;
		clear();
	};
	const ondrop = (event) => {
		if (off() || !carriesFiles(event)) return;
		event.preventDefault();
		clear();
		const files = [...(event.dataTransfer?.files ?? [])];
		const out = new CustomEvent('tuita-files', { bubbles: true, detail: { files } });
		zone.dispatchEvent(out);
	};

	zone.addEventListener('dragenter', onenter);
	zone.addEventListener('dragover', onenter);
	zone.addEventListener('dragleave', onleave);
	zone.addEventListener('drop', ondrop);
	return () => {
		zone.removeEventListener('dragenter', onenter);
		zone.removeEventListener('dragover', onenter);
		zone.removeEventListener('dragleave', onleave);
		zone.removeEventListener('drop', ondrop);
	};
}

mount('[data-island="dropzone"]', mountDropzone);

function mountMultipart(form) {
	const onsubmit = (event) => {
		event.preventDefault();
		if (form.getAttribute('aria-busy') === 'true') return;
		const fallback = form.dataset.failure || failureText();
		const xhr = new XMLHttpRequest();
		const buttons = [...form.querySelectorAll('button')];
		const bar = form.querySelector('[data-role="progress"]');
		const finish = () => {
			form.removeAttribute('aria-busy');
			for (const button of buttons) button.disabled = false;
		};
		xhr.open(form.dataset.method || 'POST', form.dataset.action ?? '');
		xhr.setRequestHeader('accept', 'application/json');
		xhr.upload.onprogress = (e) => {
			if (bar && e.lengthComputable && e.total > 0 && 'value' in bar) bar.value = Math.round((e.loaded / e.total) * 100);
		};
		xhr.onload = () => {
			if (xhr.status >= 200 && xhr.status < 300) return location.reload();
			finish();
			say(problem(xhr.status, xhr.responseText, fallback).message);
		};
		xhr.onerror = () => {
			finish();
			say(fallback);
		};
		form.setAttribute('aria-busy', 'true');
		for (const button of buttons) button.disabled = true;
		xhr.send(new FormData(form));
	};
	form.addEventListener('submit', onsubmit);
	return () => form.removeEventListener('submit', onsubmit);
}

mount('[data-island="multipart-form"]', mountMultipart);

function mountCropUpload(root) {
	const purpose = root.dataset.purpose ?? 'avatar';
	const pick = root.querySelector('input[data-role="pick"]');
	const dialog = root.querySelector('dialog');
	const cropper = dialog?.querySelector('[data-island="cropper"]');
	if (!pick || !dialog || !cropper) return;
	const table = strings();
	let file = null;
	let sending = false;

	const onpick = () => {
		const chosen = pick.files?.[0];
		pick.value = '';
		if (!chosen) return;
		if (kindOf(chosen) !== 'image') return say(table.image_field_not_image);
		if (chosen.size > MAX_IMAGE_BYTES) return say(table.image_field_too_large);
		file = chosen;
		cropper.dispatchEvent(new CustomEvent('tuita-crop-load', { detail: { file: chosen } }));
		dialog.showModal();
	};
	const oncrop = async (event) => {
		if (!file || sending) return;
		sending = true;
		cropper.setAttribute('data-busy', '');
		try {
			const media = await uploadMedia(file, { purpose, crop: JSON.parse(event.data) });
			const done = new CustomEvent('tuita-uploaded', { bubbles: true, detail: media });
			done.data = JSON.stringify(media);
			root.dispatchEvent(done);
			// The reload would drop a toast, so the message rides in a cookie the next page shows.
			document.cookie = `tuita_flash=image_field_saved; path=/; SameSite=Lax`;
			location.reload();
		} catch (error) {
			say(error instanceof Error ? error.message : failureText());
			sending = false;
			cropper.removeAttribute('data-busy');
		}
	};
	const onpickbutton = (event) => {
		if (event.target instanceof Element && event.target.closest('[data-act="pick"]')) pick.click();
	};
	const oncancel = () => dialog.close();
	pick.addEventListener('change', onpick);
	root.addEventListener('tuita-crop', oncrop);
	root.addEventListener('tuita-crop-cancel', oncancel);
	root.addEventListener('click', onpickbutton);
	return () => {
		pick.removeEventListener('change', onpick);
		root.removeEventListener('tuita-crop', oncrop);
		root.removeEventListener('tuita-crop-cancel', oncancel);
		root.removeEventListener('click', onpickbutton);
	};
}

mount('[data-island="crop-upload"]', mountCropUpload);
