it("should regenerate contenthash", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	const value1 = new URL("./file.text", import.meta.url);
	expect(/file\.[\da-f]{16}\.text/.test(value1.toString())).toBe(true);
	module.hot.accept("./file.text", function() {
		const value2 = new URL("./file.text", import.meta.url);
		expect(/file\.[\da-f]{16}\.text/.test(value2.toString())).toBe(true);
		expect(value1.toString()).not.toBe(value2.toString());
		done();
	});
	NEXT(require("../../update")(done));
}));
