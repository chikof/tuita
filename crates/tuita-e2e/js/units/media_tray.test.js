// Known-answer vectors for assets/js/upload.js.
const eq = (actual, expected, what) => {
	if (JSON.stringify(actual) !== JSON.stringify(expected)) {
		throw new Error(`${what}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
	}
};

const media = (id) => ({ id, kind: 'image', url: `/media/${id}.webp`, posterUrl: null, alt: '', width: 10, height: 10 });
const file = (name, type = 'image/png', size = 4) => new File([new Uint8Array(size)], name, { type });
const settle = () => new Promise((resolve) => setTimeout(resolve, 0));

export async function run(mod) {
	function tray(over = {}) {
		let n = 0;
		const calls = { upload: 0, describe: [] };
		const deps = {
			upload: async () => {
				calls.upload++;
				return media(`m${++n}`);
			},
			describe: async (id, alt) => {
				calls.describe.push([id, alt]);
				return { ...media(id), alt };
			},
			preview: (f) => `blob:${f.name}`,
			revoke: () => {},
			poster: async () => undefined,
			maxVideoMb: () => 50,
			hasPoll: () => false,
			...over
		};
		return { tray: new mod.MediaTray(deps), calls };
	}

	// four images upload, the fifth is refused, alt text is saved in order
	{
		const { tray: t, calls } = tray();
		const refused = t.add(['a', 'b', 'c', 'd', 'e'].map((n) => file(`${n}.png`)));
		eq(refused, [{ code: 'max-images' }], 'the fifth image');
		eq(t.images, 4, 'four images');
		eq(t.busy, true, 'uploading');
		await settle();
		eq(calls.upload, 4, 'four uploads');
		eq(t.busy, false, 'settled');
		t.setAlt(t.items[1].key, 'a cat');
		eq(await t.finalize(), ['m1', 'm2', 'm3', 'm4'], 'ids in order');
		eq(calls.describe, [['m2', 'a cat']], 'only the changed alt text is saved');
	}

	// refusals
	{
		const { tray: t } = tray();
		eq(t.add([file('x.pdf', 'application/pdf')]), [{ code: 'type' }], 'wrong type');
		eq(t.add([file('big.png', 'image/png', 10 * 1024 * 1024 + 1)]), [{ code: 'image-size' }], 'big image');
		eq(t.add([file('v.mp4', 'video/mp4', 51 * 1024 * 1024)]), [{ code: 'video-size', mb: 50 }], 'big video');
		t.add([file('ok.png')]);
		eq(t.add([file('v.mp4', 'video/mp4')]), [{ code: 'exclusive' }], 'images exclude a video');
		eq(tray({ hasPoll: () => true }).tray.add([file('a.png')]), [{ code: 'exclusive' }], 'a poll excludes media');
	}

	// a failed upload stays for a retry, a removed one goes
	{
		let fail = true;
		const { tray: t } = tray({
			upload: async () => {
				if (fail) throw new Error('Image is too large (max 10 MB).');
				return media('ok');
			}
		});
		t.add([file('a.png')]);
		await settle();
		eq(t.failed, true, 'failed');
		eq(t.items[0].error, 'Image is too large (max 10 MB).', 'the message');
		fail = false;
		t.retry(t.items[0].key);
		await settle();
		eq(t.ids, ['ok'], 'retried');
		t.remove(t.items[0].key);
		eq(t.items, [], 'removed');
	}

	// the owner hears about every change
	{
		const seen = [];
		let n = 0;
		const t = new mod.MediaTray(
			{
				upload: async (_f, options) => {
					options.onprogress(0.5);
					return media(`m${++n}`);
				},
				describe: async (id, alt) => ({ ...media(id), alt }),
				preview: () => 'blob:x',
				revoke: () => {},
				poster: async () => undefined,
				maxVideoMb: () => 50,
				hasPoll: () => false
			},
			(item) => seen.push(item ? item.status + ':' + item.progress : 'list')
		);
		t.add([file('a.png')]);
		await settle();
		eq(seen, ['list', 'uploading:0', 'uploading:0.5', 'done:1'], 'change notifications');
		t.clear();
		eq(seen.at(-1), 'list', 'clearing changes the list');
	}
}
