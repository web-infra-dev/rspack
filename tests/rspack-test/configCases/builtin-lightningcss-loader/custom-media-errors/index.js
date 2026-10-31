it('should report custom media errors from the loader', () => {
	for (const load of [
		() => require('./definitions.css'),
		() => require('./component.css'),
		() => require('./conditional.css'),
		() => require('./cyclic.css'),
	]) {
		try {
			load();
		} catch {
			// The module build errors are asserted by errors.mjs.
		}
	}
});
