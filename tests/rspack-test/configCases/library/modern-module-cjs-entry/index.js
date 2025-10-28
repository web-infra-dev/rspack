it('should contains default export for json module', async () => {
	const jsonModule = await import(/*webpackIgnore: true*/'./bundle.mjs');
	expect(jsonModule.default.foo()).toEqual(42);
})

exports.foo = () => 42
