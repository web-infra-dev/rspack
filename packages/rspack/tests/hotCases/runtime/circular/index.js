import a from "./a";

it("should not throw on circular dependencies", (done) => {
	expect(a).toBe(1);
	module.hot.accept("./a", () => {
		// should be a
		expect(require('./a').default).toBe(2);
		done();
	});
	NEXT(require("../../update")(done));
});
