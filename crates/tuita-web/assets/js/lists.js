// Lists: the create/edit dialog, deleting a list and removing a member. Pages are server-rendered
// (src/lists/*.rs); this island does what needs the browser.
//
// Hooks (document-wide, so a button inside a fetched dialog works too):
//   [data-act="list-new"]            open the editor for a new list; `data-acct="<acct>"` (the list picker's
//                                    button) adds that account to the list once it exists and reopens the picker
//   [data-act="list-edit"][data-list-id]   open the editor of that list
//   [data-act="list-delete-yes"][data-list-id]   delete the list, then go to /lists
//   [data-act="list-remove-member"][data-list-id][data-acct]   remove a member from `[data-list-members]`
// The editor is `dialog#list-editor` from `/_tuita/dialog/list-editor[/<id>]`, put into `#post-dialogs`:
// `form[data-list-form]` (`data-list-id` when editing), `[data-list-private]` (a switch), `.failure`, and the
// save button `[data-act="list-save"]`.
//
// Client strings: lists_members (plural, with {compact}).

const enc = encodeURIComponent;

const toast = (detail) => document.dispatchEvent(new CustomEvent('tuita-toast', { detail }));

function strings() {
	try {
		return JSON.parse(document.getElementById('tuita-l10n')?.textContent ?? '{}');
	} catch {
		return {};
	}
}

const lang = () => document.documentElement.lang || 'en';

/** Fills `{name}` slots of a client string. */
export function fill(template, values) {
	return String(template ?? '').replace(/\{(\w+)\}/g, (hit, name) => (name in values ? String(values[name]) : hit));
}

/** A plural client string for `count`: the sample whose plural category matches the count's. */
export function pluralText(entry, count, language = lang()) {
	if (typeof entry === 'string') return fill(entry, { count });
	const rules = new Intl.PluralRules(language);
	const wanted = rules.select(count);
	const sample = Object.keys(entry).find((key) => rules.select(Number(key)) === wanted) ?? Object.keys(entry)[0];
	return fill(entry[sample], { count });
}

const compact = (n, language = lang()) =>
	new Intl.NumberFormat(language, { notation: 'compact', maximumFractionDigits: 1 }).format(n);

/** "12 members" for a list of `n`, as the server writes it. */
export const memberCount = (n, language = lang()) =>
	fill(pluralText(strings().lists_members, n, language), { compact: compact(n, language) });

/** The bar's sub line: `@owner · 12 members`. */
export const subLine = (owner, n, language = lang()) => `@${owner} · ${memberCount(n, language)}`;

/** The body of `POST /lists` and `PATCH /lists/{id}` for a filled editor. */
export const listBody = (name, description, hidden) => ({
	name: name.trim(),
	description: description.trim(),
	private: hidden
});

async function api(method, path, body) {
	const response = await fetch(`/api/v1${path}`, {
		method,
		credentials: 'same-origin',
		headers: body === undefined ? { accept: 'application/json' } : { accept: 'application/json', 'content-type': 'application/json' },
		body: body === undefined ? undefined : JSON.stringify(body)
	});
	let data = null;
	try {
		data = await response.json();
	} catch {
		data = null;
	}
	if (!response.ok) throw new Error(data?.error ?? response.statusText ?? String(response.status));
	return data;
}

const messageOf = (error) => (error instanceof Error ? error.message : String(error));

/** Fetches a dialog into `#post-dialogs` and opens it modally; it removes itself on close. */
async function openDialog(url) {
	try {
		const response = await fetch(url, { credentials: 'same-origin', headers: { accept: 'text/html' } });
		if (!response.ok) throw new Error(String(response.status));
		const holder = document.createElement('template');
		holder.innerHTML = await response.text();
		const dialog = holder.content.querySelector('dialog');
		let host = document.getElementById('post-dialogs');
		if (!host) {
			host = document.createElement('div');
			host.id = 'post-dialogs';
			document.body.append(host);
		}
		if (!dialog) return null;
		host.replaceChildren(dialog);
		dialog.addEventListener('close', () => dialog.remove(), { once: true });
		dialog.showModal();
		return dialog;
	} catch (error) {
		toast(messageOf(error));
		return null;
	}
}

async function openEditor(button) {
	const id = button.dataset.listId;
	const dialog = await openDialog(id ? `/_tuita/dialog/list-editor/${enc(id)}` : '/_tuita/dialog/list-editor');
	if (dialog && button.dataset.acct) dialog.dataset.acct = button.dataset.acct;
}

const field = (form, name) => form.querySelector(`[name="${name}"]`);

async function save(form) {
	const dialog = form.closest('dialog');
	const button = document.querySelector('[data-act="list-save"]');
	const failure = form.querySelector('.failure');
	const name = field(form, 'name').value;
	if (!name.trim() || form.dataset.busy) return;
	const body = listBody(
		name,
		field(form, 'description').value,
		form.querySelector('[data-list-private]')?.getAttribute('aria-checked') === 'true'
	);
	form.dataset.busy = '';
	button?.setAttribute('aria-busy', 'true');
	if (failure) failure.hidden = true;
	try {
		const id = form.dataset.listId;
		const saved = id ? await api('PATCH', `/lists/${enc(id)}`, body) : await api('POST', '/lists', body);
		const acct = dialog?.dataset.acct;
		dialog?.close();
		if (id) {
			location.reload();
		} else if (acct) {
			await api('POST', `/lists/${enc(saved.id)}/members/${enc(acct)}`);
			await openDialog(`/_tuita/dialog/lists/${enc(acct)}`);
		} else {
			location.assign(`/lists/${enc(saved.id)}`);
		}
	} catch (error) {
		if (failure) {
			failure.textContent = messageOf(error);
			failure.hidden = false;
		}
	} finally {
		delete form.dataset.busy;
		button?.removeAttribute('aria-busy');
	}
}

async function removeMember(button) {
	const { listId, acct } = button.dataset;
	try {
		await api('DELETE', `/lists/${enc(listId)}/members/${enc(acct)}`);
	} catch (error) {
		toast(messageOf(error));
		return;
	}
	const list = document.querySelector('[data-list-members]');
	(button.closest('[role="listitem"]') ?? button).remove();
	if (!list) return;
	const left = Math.max(0, Number(list.dataset.count) - 1);
	list.dataset.count = String(left);
	const sub = document.querySelector('[data-c="page-bar"] .sub');
	if (sub) sub.textContent = subLine(list.dataset.owner, left);
	if (!list.querySelector('[role="listitem"]')) location.reload();
}

async function removeList(button) {
	try {
		await api('DELETE', `/lists/${enc(button.dataset.listId)}`);
		location.assign('/lists');
	} catch (error) {
		toast(messageOf(error));
	}
}

if (typeof document !== 'undefined' && !document.documentElement.dataset.listsBound) {
	document.documentElement.dataset.listsBound = '';

	document.addEventListener('click', (event) => {
		const target = event.target instanceof Element ? event.target : null;
		const act = target?.closest('[data-act^="list-"]');
		if (!act || act.disabled) return;
		switch (act.dataset.act) {
			case 'list-new':
			case 'list-edit':
				void openEditor(act);
				break;
			case 'list-delete-yes':
				void removeList(act);
				break;
			case 'list-remove-member':
				void removeMember(act);
				break;
		}
	});

	// The private switch is not a `[data-act]`, so it has its own listener.
	document.addEventListener('click', (event) => {
		const toggle = event.target instanceof Element ? event.target.closest('[data-list-private]') : null;
		if (!toggle) return;
		toggle.dataset.touched = '';
		toggle.setAttribute('aria-checked', toggle.getAttribute('aria-checked') === 'true' ? 'false' : 'true');
	});

	document.addEventListener('input', (event) => {
		const form = event.target instanceof Element ? event.target.closest('form[data-list-form]') : null;
		const button = document.querySelector('[data-act="list-save"]');
		if (form && button) button.disabled = !field(form, 'name').value.trim();
	});

	document.addEventListener('submit', (event) => {
		const form = event.target instanceof Element ? event.target.closest('form[data-list-form]') : null;
		if (!form) return;
		event.preventDefault();
		void save(form);
	});
}
