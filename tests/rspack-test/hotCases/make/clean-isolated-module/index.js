import value from "./a";

it("should make clean isolated module works", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	expect(value).toBe("cba");
	module.hot.accept("./a", () => {
		expect(value).toBe("a");
		done();
	});
	NEXT(require("@rspack/test-tools/helper/legacy/update")(done));
}));
