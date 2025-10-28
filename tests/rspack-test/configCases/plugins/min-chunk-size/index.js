it("should combine two chunk if too small", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	// b should not yet available
	var bf = __webpack_modules__[require.resolveWeak("./b")];
	expect((typeof bf)).toBe("undefined");

	// load a
	import("./a").then(a => {
		expect(a.default).toBe("a");
		// check if b is available too
		var bf = __webpack_modules__[require.resolveWeak("./b")];
		expect((typeof bf)).toBe("function");

		// load b (just to check if it's ok)
		import("./b").then(b => {
			expect(b.default).toBe("b");
			done();
		}).catch(done);
	}).catch(done);
}));
