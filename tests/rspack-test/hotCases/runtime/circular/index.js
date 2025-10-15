import a from "./a";

it("should not throw on circular dependencies", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	expect(a).toBe(1);
	module.hot.accept("./a", () => {
		expect(a).toBe(2);
		done();
	});
	NEXT(require("@rspack/test-tools/helper/legacy/update")(done));
}));
