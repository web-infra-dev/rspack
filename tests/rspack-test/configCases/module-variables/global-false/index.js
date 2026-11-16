it("global false", function () {
	global;
	const scope = typeof __rspack_context === 'undefined' ? __webpack_require__ : __rspack_context;
	// A static read of the global helper would request it and invalidate the test.
	expect('g' in scope).toBe(false);
});
