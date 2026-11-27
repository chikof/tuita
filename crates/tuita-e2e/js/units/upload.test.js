// Known-answer vectors for assets/js/upload.js, including the `attachBlock` cases.
const eq = (actual, expected, what) => {
	if (JSON.stringify(actual) !== JSON.stringify(expected)) {
		throw new Error(`${what}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
	}
};

class FakeXhr {
	status = 0;
	responseText = '';
	upload = { onprogress: null };
	onload = null;
	onerror = null;
	onabort = null;
	opened;
	sent;
	open = (method, url) => void (this.opened = [method, url]);
	setRequestHeader = () => {};
	send = (body) => void (this.sent = body);
	abort = () => this.onabort?.({});
}

const png = () => new File([new Uint8Array(4)], 'a.png', { type: 'image/png' });
const dto = { id: 'm1', kind: 'image', url: '/media/m1.webp', posterUrl: null, alt: '', width: 3, height: 2 };

const rejection = async (promise) => {
	try {
		await promise;
	} catch (error) {
		return error;
	}
	throw new Error('expected a rejection');
};

export async function run(mod) {
	// attachBlock
	const none = { images: 0, video: false, poll: false };
	eq(mod.attachBlock({ ...none, images: 3 }, 'image'), null, 'the fourth image fits');
	eq(mod.attachBlock({ ...none, images: 4 }, 'image'), 'max-images', 'the fifth image does not');
	eq(mod.attachBlock({ ...none, images: 1 }, 'video'), 'exclusive', 'a video excludes images');
	eq(mod.attachBlock({ ...none, video: true }, 'image'), 'exclusive', 'images exclude a video');
	eq(mod.attachBlock({ ...none, video: true }, 'video'), 'exclusive', 'one video only');
	eq(mod.attachBlock({ ...none, poll: true }, 'image'), 'exclusive', 'a poll excludes media');
	eq(mod.attachBlock(none, 'video'), null, 'a first video fits');

	// kindOf
	eq(mod.kindOf({ type: 'image/webp' }), 'image', 'webp is an image');
	eq(mod.kindOf({ type: 'video/quicktime' }), 'video', 'mov is a video');
	eq(mod.kindOf({ type: 'application/pdf' }), null, 'pdf is neither');

	// accepts (the dropzone's `data-accept`)
	eq(mod.accepts('', 'application/pdf'), true, 'an empty list accepts anything');
	eq(mod.accepts('image/*,video/*', 'image/png'), true, 'a wildcard');
	eq(mod.accepts('image/*,video/*', 'application/pdf'), false, 'outside the wildcards');
	eq(mod.accepts('image/png, video/mp4', 'video/mp4'), true, 'an exact type');
	eq(mod.accepts('image/png', 'image/gif'), false, 'another exact type');

	// uploadMedia: multipart, progress, the MediaDto
	{
		const xhr = new FakeXhr();
		const seen = [];
		const result = mod.uploadMedia(png(), { alt: 'a cat', onprogress: (f) => seen.push(f) }, undefined, () => xhr);
		xhr.upload.onprogress?.({ lengthComputable: true, loaded: 1, total: 4 });
		xhr.status = 200;
		xhr.responseText = JSON.stringify(dto);
		xhr.onload?.({});
		eq(await result, dto, 'resolves to the MediaDto');
		eq(seen, [0.25], 'progress is a fraction');
		eq(xhr.opened, ['POST', '/api/v1/media'], 'the endpoint');
		eq([...xhr.sent.keys()], ['file', 'purpose', 'alt'], 'the form fields');
		eq(xhr.sent.get('purpose'), 'post', 'the default purpose');
	}

	// the crop travels as JSON
	{
		const xhr = new FakeXhr();
		void mod
			.uploadMedia(png(), { purpose: 'avatar', crop: { x: 1, y: 2, w: 3, h: 3, rotate: 90 } }, undefined, () => xhr)
			.catch(() => {});
		eq(xhr.sent.get('crop'), '{"x":1,"y":2,"w":3,"h":3,"rotate":90}', 'crop as JSON');
	}

	// the server's message
	{
		const xhr = new FakeXhr();
		const result = mod.uploadMedia(png(), {}, undefined, () => xhr);
		xhr.status = 413;
		xhr.responseText = '{"error":"Image is too large (max 10 MB)."}';
		xhr.onload?.({});
		const error = await rejection(result);
		eq([error.status, error.message], [413, 'Image is too large (max 10 MB).'], 'the API error');
	}

	// a network error uses the fallback, an abort is an AbortError
	{
		const down = new FakeXhr();
		const failed = mod.uploadMedia(png(), {}, 'Nope', () => down);
		down.onerror?.({});
		const error = await rejection(failed);
		eq([error.status, error.message], [0, 'Nope'], 'network failure');

		const xhr = new FakeXhr();
		const control = new AbortController();
		const stopped = mod.uploadMedia(png(), { signal: control.signal }, undefined, () => xhr);
		control.abort();
		eq((await rejection(stopped)).name, 'AbortError', 'abort');
	}
}
