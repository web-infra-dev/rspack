it("should be able to recover from json error", function (done) {
	expect(() => require("./data.json")).toThrowError();
	module.hot.accept("./data.json", function () {
		expect(require("./data.json")).toBe(42);
		done();
	});
	NEXT(require("@rspack/test-tools/helper/legacy/update")(done));
});
