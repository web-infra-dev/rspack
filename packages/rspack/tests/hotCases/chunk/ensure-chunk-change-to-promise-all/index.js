it("should still works when ensure chunk causes the parent chunk change", function (done) {
	import("./file").then(({ React }) => {
		expect(React).toBe(42);
		debugger
		NEXT(require("../../update")(done));
		import.meta.webpackHot.accept("./file", () => {
			import("./file").then(({ Vue }) => {
				debugger
				expect(Vue).toBe(43);
				done()
			})
		});
	});
});
