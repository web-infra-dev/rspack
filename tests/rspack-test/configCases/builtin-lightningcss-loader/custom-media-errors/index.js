it('should report circular custom media from the loader', () => {
	for (const load of [
		() => require('./cyclic.css'),
		() => require('./missing-before-cyclic.css'),
		() => require('./cyclic-before-missing.css'),
	]) {
		try {
			load();
		} catch {
			// The module build errors are asserted by errors.mjs.
		}
	}
});
