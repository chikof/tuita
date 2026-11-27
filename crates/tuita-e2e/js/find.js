// Locator engine of the e2e harness, installed with Page.addScriptToEvaluateOnNewDocument.
// Rust sends locators as JSON step lists; see `src/loc.rs`. Everything here runs in the page.
(() => {
  'use strict';
  if (window.__e2e) return;

  const MARKS = /[\u2068\u2069]/g;
  const norm = (s) => String(s ?? '').replace(MARKS, '').replace(/\s+/g, ' ').trim();
  const SKIP = new Set(['script', 'style', 'template', 'noscript', 'head', 'meta', 'link', 'title']);
  // Text as a reader sees it: scripts, styles and templates do not count (an inline JSON data block is
  // not content), so a client string never makes `<body>` match a text locator.
  const textOf = (el) => {
    let out = '';
    for (const n of el.childNodes) {
      if (n.nodeType === 3) out += n.data;
      else if (n.nodeType === 1 && !SKIP.has(n.localName)) out += textOf(n);
    }
    return out;
  };

  // A Name is {t: 'exact' | 'sub' | 'ci' | 're', v}. Exact is a whole-string, case-sensitive
  // match; sub is a case-insensitive substring; re is a RegExp source; ci a case-insensitive one.
  function matcher(n, raw = false) {
    const prep = raw ? (s) => String(s) : norm;
    if (n.t === 'exact') {
      const v = prep(n.v);
      return (s) => s === v;
    }
    if (n.t === 'sub') {
      const v = prep(n.v).toLowerCase();
      return (s) => s.toLowerCase().includes(v);
    }
    const re = new RegExp(n.v, n.t === 'ci' ? 'i' : '');
    return (s) => re.test(s);
  }

  function visible(el) {
    if (!el.isConnected) return false;
    const box = el.getBoundingClientRect();
    if (box.width === 0 || box.height === 0) return false;
    if (typeof el.checkVisibility === 'function' && !el.checkVisibility()) return false;
    return getComputedStyle(el).visibility === 'visible';
  }

  function implicitRole(el) {
    switch (el.localName) {
      case 'button':
        return 'button';
      case 'a':
      case 'area':
        return el.hasAttribute('href') ? 'link' : null;
      case 'input': {
        const type = (el.getAttribute('type') || 'text').toLowerCase();
        switch (type) {
          case 'button':
          case 'submit':
          case 'reset':
          case 'image':
            return 'button';
          case 'checkbox':
            return 'checkbox';
          case 'radio':
            return 'radio';
          case 'range':
            return 'slider';
          case 'number':
            return 'spinbutton';
          case 'search':
            return el.hasAttribute('list') ? 'combobox' : 'searchbox';
          case 'email':
          case 'tel':
          case 'text':
          case 'url':
            return el.hasAttribute('list') ? 'combobox' : 'textbox';
          default:
            return null;
        }
      }
      case 'textarea':
        return 'textbox';
      case 'select':
        return el.multiple || el.size > 1 ? 'listbox' : 'combobox';
      case 'h1':
      case 'h2':
      case 'h3':
      case 'h4':
      case 'h5':
      case 'h6':
        return 'heading';
      case 'ul':
      case 'ol':
      case 'menu':
        return 'list';
      case 'li':
        return 'listitem';
      case 'dialog':
        return 'dialog';
      case 'nav':
        return 'navigation';
      case 'main':
        return 'main';
      case 'img':
        return el.getAttribute('alt') === '' ? 'presentation' : 'img';
      case 'article':
        return 'article';
      case 'aside':
        return 'complementary';
      case 'section':
        return el.hasAttribute('aria-label') || el.hasAttribute('aria-labelledby') ? 'region' : null;
      case 'form':
        return el.hasAttribute('aria-label') || el.hasAttribute('aria-labelledby') ? 'form' : null;
      case 'header':
        return el.closest('article, aside, main, nav, section') ? null : 'banner';
      case 'footer':
        return el.closest('article, aside, main, nav, section') ? null : 'contentinfo';
      case 'table':
        return 'table';
      case 'thead':
      case 'tbody':
      case 'tfoot':
        return 'rowgroup';
      case 'tr':
        return 'row';
      case 'th':
        return 'columnheader';
      case 'td':
        return 'cell';
      case 'progress':
        return 'progressbar';
      case 'output':
        return 'status';
      case 'hr':
        return 'separator';
      case 'fieldset':
        return 'group';
      case 'option':
        return 'option';
      default:
        return null;
    }
  }

  function roleOf(el) {
    const explicit = (el.getAttribute('role') || '').trim().split(/\s+/)[0];
    return explicit || implicitRole(el);
  }

  // Text a name is built from: text nodes, alt of images, no hidden or aria-hidden subtrees, no
  // values of form controls. `skip` is a control whose own subtree a <label> must not contribute.
  function content(node, skip, top) {
    if (node.nodeType === 3) return node.data;
    if (node.nodeType !== 1 || node === skip) return '';
    const tag = node.localName;
    if (SKIP.has(tag)) return '';
    if (!top) {
      if (node.getAttribute('aria-hidden') === 'true' || node.hidden) return '';
      if (getComputedStyle(node).display === 'none') return '';
    }
    if (tag === 'img') return node.getAttribute('alt') || '';
    if (tag === 'input' || tag === 'textarea' || tag === 'select') return '';
    let out = '';
    for (const child of node.childNodes) out += content(child, skip, false);
    if (!top && !getComputedStyle(node).display.startsWith('inline')) out = ` ${out} `;
    return out;
  }

  function labelledby(el) {
    const ids = el.getAttribute('aria-labelledby');
    if (!ids) return '';
    return ids
      .split(/\s+/)
      .map((id) => document.getElementById(id))
      .filter(Boolean)
      .map((e) => content(e, null, true))
      .join(' ');
  }

  function labelTexts(el) {
    return el.labels ? [...el.labels].map((l) => norm(content(l, el, true))) : [];
  }

  // aria-label, else aria-labelledby, else <label>, else alt, else content (buttons, links,
  // headings, ...; form controls have none), else title, else value of button inputs, else placeholder.
  function accName(el) {
    let t = norm(el.getAttribute('aria-label'));
    if (t) return t;
    t = norm(labelledby(el));
    if (t) return t;
    t = labelTexts(el).filter(Boolean).join(' ');
    if (t) return t;
    t = norm(el.getAttribute('alt'));
    if (t) return t;
    t = norm(content(el, null, true));
    if (t) return t;
    t = norm(el.getAttribute('title'));
    if (t) return t;
    if (el.localName === 'input' && /^(button|submit|reset)$/i.test(el.type)) {
      t = norm(el.value);
      if (t) return t;
    }
    return norm(el.getAttribute('placeholder'));
  }

  const hiddenFromRoles = (el) => !!el.closest('[aria-hidden="true"]');
  const all = (scope) => [...scope.querySelectorAll('*')].filter((e) => !SKIP.has(e.localName));

  function search(scope, step) {
    switch (step.k) {
      case 'css':
        return [...scope.querySelectorAll(step.sel)];
      case 'role': {
        const m = step.name ? matcher(step.name) : null;
        return all(scope).filter(
          (el) =>
            roleOf(el) === step.role &&
            visible(el) &&
            !hiddenFromRoles(el) &&
            (!m || m(accName(el)))
        );
      }
      case 'label': {
        const m = matcher(step.name);
        const found = scope.querySelectorAll(
          '[aria-label], [aria-labelledby], input, textarea, select, button, meter, output, progress'
        );
        return [...found].filter((el) => {
          if (!visible(el)) return false;
          const own = [norm(el.getAttribute('aria-label')), norm(labelledby(el))];
          return [...own, ...labelTexts(el)].some((t) => t && m(t));
        });
      }
      case 'text': {
        const m = matcher(step.name);
        const cache = new Map();
        const hit = (el) => {
          if (!cache.has(el)) cache.set(el, m(norm(textOf(el))));
          return cache.get(el);
        };
        return all(scope).filter(
          (el) => hit(el) && ![...el.children].some((c) => !SKIP.has(c.localName) && hit(c)) && visible(el)
        );
      }
      default:
        throw new Error(`unknown locator step ${step.k}`);
    }
  }

  function find(steps, root = document) {
    let cur = [root];
    for (const step of steps) {
      if (step.k === 'nth') {
        const el = cur.at(step.n);
        cur = el ? [el] : [];
      } else if (step.k === 'has_text') {
        const m = matcher(step.name);
        cur = cur.filter((el) => m(norm(textOf(el))));
      } else if (step.k === 'has') {
        cur = cur.filter((el) => find(step.steps, el).length > 0);
      } else {
        const out = new Set();
        for (const scope of cur) for (const el of search(scope, step)) out.add(el);
        cur = [...out].sort((a, b) =>
          a === b ? 0 : a.compareDocumentPosition(b) & Node.DOCUMENT_POSITION_FOLLOWING ? -1 : 1
        );
      }
    }
    return cur;
  }

  const enabled = (el) => !el.matches(':disabled') && el.getAttribute('aria-disabled') !== 'true';

  function describeHit(el) {
    if (!el) return 'nothing';
    const cls = typeof el.className === 'string' && el.className ? `.${el.className.split(/\s+/)[0]}` : '';
    return `<${el.localName}${cls}>`;
  }

  // Whether a click at (x, y) lands on `el`: the hit element is el, inside it, or a label of it.
  function receives(el, x, y) {
    const hit = document.elementFromPoint(x, y);
    if (!hit) return { ok: false, hit };
    if (el.contains(hit) || hit.contains(el)) return { ok: true, hit };
    const label = hit.closest('label');
    if (label && el.labels && [...el.labels].includes(label)) return { ok: true, hit };
    return { ok: false, hit };
  }

  const ALIGN = ['center', 'start', 'end', 'nearest'];

  function single(steps) {
    const els = find(steps);
    if (els.length === 0) return { fail: 'no element matches' };
    if (els.length > 1) return { fail: `strict mode violation: ${els.length} elements match` };
    return { el: els[0] };
  }

  // Scrolls `el` into view and returns where to click. Fails until its box holds still for two
  // polls; a covered click point moves on to the next scroll alignment (sticky bars).
  const point = (el, mustEnable) => {
    if (!visible(el)) return { ok: false, reason: 'element is not visible' };
    if (mustEnable && !enabled(el)) return { ok: false, reason: 'element is disabled' };
    const block = ALIGN[(el.__e2eCover ?? 0) % ALIGN.length];
    el.scrollIntoView({ block, inline: block, behavior: 'instant' });
    const box = el.getBoundingClientRect();
    const key = `${box.x},${box.y},${box.width},${box.height}`;
    if (el.__e2eBox !== key) {
      el.__e2eBox = key;
      return { ok: false, reason: 'element is not stable' };
    }
    const x = box.x + box.width / 2;
    const y = box.y + box.height / 2;
    const { ok, hit } = receives(el, x, y);
    if (!ok) {
      el.__e2eCover = (el.__e2eCover ?? 0) + 1;
      el.__e2eBox = null;
      return { ok: false, reason: `click point is covered by ${describeHit(hit)}` };
    }
    return { ok: true, x, y };
  };

  const checkedOf = (el) =>
    typeof el.checked === 'boolean' ? el.checked : el.getAttribute('aria-checked') === 'true';

  const setValue = (el, value) => {
    const proto = el instanceof HTMLTextAreaElement ? HTMLTextAreaElement.prototype : HTMLInputElement.prototype;
    Object.getOwnPropertyDescriptor(proto, 'value').set.call(el, value);
    el.dispatchEvent(new Event('input', { bubbles: true }));
    el.dispatchEvent(new Event('change', { bubbles: true }));
  };

  // One entry point: Rust evaluates `__e2e.act(steps, op, arg)` and reads the JSON answer.
  function act(steps, op, arg) {
    if (op === 'count') return { ok: true, value: find(steps).length };
    if (op === 'texts') return { ok: true, value: find(steps).map((el) => norm(textOf(el))) };
    if (op === 'info') {
      const els = find(steps);
      return { ok: true, count: els.length, visible: els.map(visible) };
    }
    const s = single(steps);
    if (s.fail) return { ok: false, reason: s.fail };
    const el = s.el;
    switch (op) {
      case 'text':
        return { ok: true, value: norm(textOf(el)) };
      case 'match': {
        const f = arg.field;
        let actual = null;
        if (f === 'text') actual = norm(textOf(el));
        else if (f === 'value') actual = String(el.value ?? '');
        else if (f.startsWith('attr:')) actual = el.getAttribute(f.slice(5));
        else if (f.startsWith('css:')) actual = getComputedStyle(el).getPropertyValue(f.slice(4)).trim();
        if (actual === null) return { ok: true, match: false, actual };
        return { ok: true, match: matcher(arg.name, f !== 'text')(String(actual)), actual };
      }
      case 'inner_text':
        return { ok: true, value: String(el.innerText ?? '').replace(MARKS, '') };
      case 'html':
        return { ok: true, value: el.innerHTML };
      case 'value':
        return { ok: true, value: String(el.value ?? '') };
      case 'checked':
        return { ok: true, value: checkedOf(el) };
      case 'enabled':
        return { ok: true, value: enabled(el) };
      case 'attr':
        return { ok: true, value: el.getAttribute(arg) };
      case 'css':
        return { ok: true, value: getComputedStyle(el).getPropertyValue(arg).trim() };
      case 'visible':
        return { ok: true, value: visible(el) };
      case 'click':
        return point(el, true);
      case 'hover':
        return point(el, false);
      case 'focus':
        el.focus();
        return document.activeElement === el || el.contains(document.activeElement)
          ? { ok: true }
          : { ok: false, reason: 'element did not take focus' };
      case 'box': {
        if (!visible(el)) return { ok: false, reason: 'element is not visible' };
        el.scrollIntoView({ block: 'nearest', inline: 'nearest', behavior: 'instant' });
        const b = el.getBoundingClientRect();
        return {
          ok: true,
          x: b.x + scrollX,
          y: b.y + scrollY,
          width: b.width,
          height: b.height
        };
      }
      case 'fill': {
        if (!visible(el)) return { ok: false, reason: 'element is not visible' };
        if (!enabled(el)) return { ok: false, reason: 'element is disabled' };
        if (el.readOnly) return { ok: false, reason: 'element is read-only' };
        const editable = el.isContentEditable;
        const isText = el instanceof HTMLTextAreaElement || el instanceof HTMLInputElement;
        if (!isText && !editable) return { ok: false, reason: 'element is not fillable' };
        const direct =
          el instanceof HTMLInputElement &&
          /^(date|datetime-local|month|time|week|color|range)$/i.test(el.type);
        if (direct || arg === '') {
          el.focus();
          if (isText) setValue(el, arg);
          else el.textContent = '';
          return { ok: true, insert: false };
        }
        el.focus();
        if (isText) {
          try {
            el.select();
          } catch {
            // types without a selection: insertText appends
          }
        } else document.getSelection().selectAllChildren(el);
        return { ok: true, insert: true };
      }
      case 'clear_files':
        if (!(el instanceof HTMLInputElement) || el.type !== 'file') return { ok: false, reason: 'element is not a file input' };
        el.value = '';
        el.dispatchEvent(new Event('input', { bubbles: true }));
        el.dispatchEvent(new Event('change', { bubbles: true }));
        return { ok: true };
      case 'select': {
        if (!(el instanceof HTMLSelectElement)) return { ok: false, reason: 'element is not a <select>' };
        if (!visible(el)) return { ok: false, reason: 'element is not visible' };
        if (!enabled(el)) return { ok: false, reason: 'element is disabled' };
        const options = [...el.options];
        const hit =
          options.find((o) => o.value === arg) || options.find((o) => norm(o.textContent) === norm(arg));
        if (!hit) return { ok: false, reason: `no option ${JSON.stringify(arg)}` };
        el.value = hit.value;
        el.dispatchEvent(new Event('input', { bubbles: true }));
        el.dispatchEvent(new Event('change', { bubbles: true }));
        return { ok: true };
      }
      default:
        return { ok: false, reason: `unknown op ${op}` };
    }
  }

  // The one element a locator resolves to, for Runtime.callFunctionOn (set_files).
  function element(steps) {
    const s = single(steps);
    if (s.fail) throw new Error(s.fail);
    return s.el;
  }

  window.__e2e = {
    match: (name, actual, raw) => ({ ok: matcher(name, raw)(String(actual)), actual }),
    act, find, element, norm, visible, roleOf, accName };
})();
