'use strict';
(function () {
	var d = document.documentElement;
	var p = {};
	try {
		p = JSON.parse(localStorage.getItem('tuita:prefs')) || {};
	} catch (e) {}
	var dark = matchMedia('(prefers-color-scheme: dark)');
	var calm = matchMedia('(prefers-reduced-motion: reduce)');
	function apply() {
		var explicit = p.mode === 'light' || p.mode === 'dark';
		d.dataset.theme = explicit ? p.mode : dark.matches ? 'dark' : 'light';
		d.dataset.ink = p.ink || 'bone';
		d.dataset.contrast = p.contrast === 'more' ? 'more' : 'normal';
		if (p.motion === 'reduce' || calm.matches) d.dataset.motion = 'reduce';
		else delete d.dataset.motion;
		d.style.setProperty('--text-scale', String(p.textScale || 1));
	}
	apply();
	dark.addEventListener('change', apply);
	calm.addEventListener('change', apply);
})();
