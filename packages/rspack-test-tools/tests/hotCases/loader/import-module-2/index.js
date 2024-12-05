it("module and its loader-referencing module should update in right order", done => {
	expect(require("./loader.js!./a")).toBe(2);
	NEXT(
		require("../../update")(done, true, () => {
			expect(require("./loader.js!./a")).toBe(3);
			done();
		})
	);
});
module.hot.accept("./loader.js!./a");