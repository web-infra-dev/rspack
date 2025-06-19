import value from "./a";

it("should make clean isolated module works", done => {
	expect(value).toBe("cba");
	module.hot.accept("./a", () => {
		expect(value).toBe("a");
		done();
	});
	NEXT(require("../../update")(done));
});
