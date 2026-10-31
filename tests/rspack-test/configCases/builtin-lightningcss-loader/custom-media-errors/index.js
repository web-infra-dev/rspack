it('should report circular custom media from the loader', () => {
	try {
		require('./cyclic.css');
	} catch {
		// The module build error is asserted by errors.mjs.
	}
});
