it("module and its loader-referencing module should update in right order", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	expect(require("./loader.js!./a")).toBe(2);
	NEXT(
		require("@rspack/test-tools/helper/legacy/update")(done, true, () => {
			expect(require("./loader.js!./a")).toBe(3);
			done();
		})
	);
}));
module.hot.accept("./loader.js!./a");
