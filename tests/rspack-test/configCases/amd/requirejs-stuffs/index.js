it('`require.config()` and `requirejs.config()` call should be replaced with undefined', function () {
	expect(require.config()).toBeUndefined();
	expect(requirejs.config()).toBeUndefined();
});
it('require.version should be "0.0.0"', function () {
	expect(require.version).toBe('0.0.0');
});

const amdOptions = { jQuery: true };

it('`define.amd` should be equal to `options.amd`', function () {
	expect(define.amd).toStrictEqual(amdOptions);
	expect(typeof define.amd).toBe('object');
});

it('`require.amd` should be equal to `options.amd`', function () {
	expect(require.amd).toStrictEqual(amdOptions);
	expect(typeof require.amd).toBe('object');
});

it('define can be renamed, but calling renamed define will throw an error', function () {
	const def = define;
	expect(def).toThrow();
})

it('`typeof define` and `typeof require` should be function', function () {
	expect(typeof define).toBe('function');
	expect(typeof require).toBe('function');
});
